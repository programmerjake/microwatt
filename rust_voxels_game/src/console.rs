use crate::take_once::{AlreadyTaken, TakeOnce};
use core::{ffi::c_int, fmt};

#[cfg(feature = "embedded")]
extern "C" {
    fn console_init();
    fn getchar() -> c_int;
    fn console_havechar() -> bool;
    fn putchar(c: c_int) -> c_int;
}

/// Safety: must only be called once
#[cfg(feature = "hosted")]
unsafe fn console_init() {
    use std::sync::Mutex;

    static ORIG_TIOS: Mutex<Option<termios::Termios>> = Mutex::new(None);

    extern "C" fn handle_exit() {
        let Some(tios) = *ORIG_TIOS.lock().unwrap() else {
            return;
        };
        let _ = termios::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &tios);
    }

    extern "C" fn handle_signal(sig: c_int) {
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
            handle_exit();
            libc::raise(sig);
        }
    }

    if let Ok(mut tios) = termios::Termios::from_fd(libc::STDIN_FILENO) {
        *ORIG_TIOS.lock().unwrap() = Some(tios);
        termios::cfmakeraw(&mut tios);
        tios.c_lflag |= termios::ISIG;
        termios::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &tios).unwrap();
        libc::atexit(handle_exit);
        if libc::signal(libc::SIGINT, handle_signal as libc::sighandler_t) == libc::SIG_IGN {
            libc::signal(libc::SIGINT, libc::SIG_IGN);
        }
        if libc::signal(libc::SIGTERM, handle_signal as libc::sighandler_t) == libc::SIG_IGN {
            libc::signal(libc::SIGTERM, libc::SIG_IGN);
        }
    }
    let flags = libc::fcntl(libc::STDIN_FILENO, libc::F_GETFL);
    assert!(flags >= 0);
    assert!(libc::fcntl(libc::STDIN_FILENO, libc::F_SETFL, flags | libc::O_NONBLOCK) >= 0);
}

#[cfg(feature = "embedded")]
fn console_try_read() -> Option<u8> {
    unsafe {
        if console_havechar() {
            Some(getchar() as u8)
        } else {
            None
        }
    }
}

#[cfg(feature = "hosted")]
fn console_try_read() -> Option<u8> {
    use std::io::Read;

    let mut retval = [0u8];
    match std::io::stdin().read(&mut retval) {
        Ok(1) => Some(retval[0]),
        _ => None,
    }
}

#[cfg(feature = "embedded")]
fn console_write(b: u8) {
    unsafe {
        putchar(b as c_int);
    }
}

#[cfg(all(feature = "hosted", not(feature = "hosted_full_speed")))]
fn console_write(b: u8) {
    use core::{
        sync::atomic::{AtomicU32, Ordering},
        time::Duration,
    };
    use std::{io::Write, sync::Mutex, thread::sleep, time::Instant};

    const CHECK_PERIOD: u32 = 1024;
    const SIMULATED_BYTES_PER_SEC: f64 = 1000000.0 / 8.0;

    static SLEEP_COUNTER: AtomicU32 = AtomicU32::new(CHECK_PERIOD);

    if SLEEP_COUNTER.fetch_add(1, Ordering::Relaxed) >= CHECK_PERIOD {
        struct SleepState {
            last_sleep: Option<Instant>,
        }
        static SLEEP_STATE: Mutex<SleepState> = Mutex::new(SleepState { last_sleep: None });
        let mut state = SLEEP_STATE.lock().unwrap();
        let sleep_counter = SLEEP_COUNTER.load(Ordering::Relaxed);
        let now = Instant::now();
        let last_sleep = state.last_sleep.get_or_insert(now);
        let target = last_sleep
            .checked_add(Duration::from_secs(sleep_counter as u64).div_f64(SIMULATED_BYTES_PER_SEC))
            .unwrap();
        if let Some(sleep_duration) = target.checked_duration_since(now) {
            sleep(sleep_duration);
        }
        *last_sleep = target;
        SLEEP_COUNTER.fetch_sub(sleep_counter, Ordering::Relaxed);
    }

    let _ = std::io::stdout().write_all(&[b]);
}

#[cfg(feature = "hosted_full_speed")]
fn console_write(b: u8) {
    use std::io::Write;

    let _ = std::io::stdout().write_all(&[b]);
}

pub struct Console(());

impl Console {
    fn try_take() -> Result<&'static mut Console, AlreadyTaken> {
        static CONSOLE: TakeOnce<Console> = TakeOnce::new(Console(()));
        let retval = CONSOLE.take()?;
        unsafe {
            console_init();
        }
        Ok(retval)
    }

    pub fn take() -> &'static mut Console {
        Self::try_take().expect("console already taken")
    }

    #[cfg(feature = "embedded")]
    pub(crate) unsafe fn emergency_console() -> &'static mut Console {
        use core::cell::UnsafeCell;

        struct EmergencyConsole(UnsafeCell<Console>);

        unsafe impl Sync for EmergencyConsole {}
        static EMERGENCY_CONSOLE: EmergencyConsole = EmergencyConsole(UnsafeCell::new(Console(())));
        Self::try_take().unwrap_or_else(|_| unsafe { &mut *EMERGENCY_CONSOLE.0.get() })
    }

    pub fn try_read(&mut self) -> Option<u8> {
        console_try_read()
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                console_write(b'\r');
            }
            console_write(b);
        }
        Ok(())
    }
}
