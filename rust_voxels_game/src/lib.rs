#![cfg_attr(all(feature = "embedded", not(test)), no_std)]

use crate::{
    fixed::Fix64,
    screen::RgbColor,
    sin_cos::sin_cos_pi,
    vec::Vec3D,
    world::{Block, World},
};
use core::fmt::Write;
#[cfg(feature = "hosted")]
use std::process::exit;

mod console;
mod fixed;
mod screen;
mod sin_cos;
mod take_once;
mod vec;
mod world;
mod shapes {
    pub(crate) mod libre_soc_logo {
        include!(concat!(env!("OUT_DIR"), "/libre-soc-logo.rs"));
    }
}

#[cfg(feature = "embedded")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    use core::sync::atomic::{AtomicBool, Ordering};

    static PANICKED: AtomicBool = AtomicBool::new(false);

    if PANICKED.swap(true, Ordering::Relaxed) {
        loop {}
    }
    let console = unsafe { console::Console::emergency_console() };
    loop {
        let _ = writeln!(console, "{info}");
    }
}

#[cfg(feature = "embedded")]
fn exit(code: i32) -> ! {
    panic!("exited code={code}");
}

#[rustfmt::skip]
const NEW_BLOCK_COLORS: [RgbColor; 10] = [
    RgbColor { r: 0, g: 0, b: 0 },
    RgbColor { r: 0, g: 0, b: 0xFF },
    RgbColor { r: 0, g: 0xFF, b: 0 },
    RgbColor { r: 0, g: 0xFF, b: 0xFF },
    RgbColor { r: 0xFF, g: 0, b: 0 },
    RgbColor { r: 0xFF, g: 0, b: 0xFF },
    RgbColor { r: 0xFF, g: 0xFF, b: 0 },
    RgbColor { r: 0xFF, g: 0xFF, b: 0xFF },
    RgbColor { r: 0x55, g: 0x55, b: 0x55 },
    RgbColor { r: 0xAA, g: 0xAA, b: 0xAA },
];

#[cfg_attr(feature = "embedded", no_mangle)]
pub extern "C" fn main() -> ! {
    let console = console::Console::take();
    console.write_str("starting...\n").unwrap();
    let screen = screen::Screen::take();
    let world = World::take();
    let mut pos = Vec3D {
        x: Fix64::from(0i64),
        y: Fix64::from(0i64),
        z: Fix64::from(20i64),
    };
    let mut theta_over_pi = Fix64::from(1i64);
    let mut phi_over_pi = Fix64::from(0i64);
    let mut blink_counter = 0;
    let blink_period = 6;
    loop {
        blink_counter = (blink_counter + 1) % blink_period;
        let (sin_theta, cos_theta) = sin_cos_pi(theta_over_pi);
        let (sin_phi, cos_phi) = sin_cos_pi(phi_over_pi);
        let forward0 = Vec3D {
            x: Fix64::from(sin_theta),
            y: Fix64::from(0i64),
            z: Fix64::from(cos_theta),
        };
        let right = Vec3D {
            x: Fix64::from(cos_theta),
            y: Fix64::from(0i64),
            z: Fix64::from(-sin_theta),
        };
        let down0 = Vec3D {
            x: Fix64::from(0i64),
            y: Fix64::from(-1i64),
            z: Fix64::from(0i64),
        };
        let forward = forward0 * cos_phi - down0 * sin_phi;
        let down = forward0 * sin_phi + down0 * cos_phi;
        let mut restore_cursor = None;
        let (_prev_pos, hit_pos) = world.get_hit_pos(pos, forward);
        if blink_counter * 3 < blink_period * 2 {
            restore_cursor = hit_pos.map(|hit_pos| {
                let block = world.get_mut(hit_pos).unwrap();
                let old = *block;
                block.color = block.color.map(|_| {
                    if blink_counter * 3 < blink_period {
                        RgbColor::black().to_packed()
                    } else {
                        RgbColor::white().to_packed()
                    }
                });
                move |world: &mut World| *world.get_mut(hit_pos).unwrap() = old
            });
        }
        world.render(screen, pos, forward, right, down);
        restore_cursor.map(|f| f(world));
        screen.display(console);
        writeln!(console, "Press WASD to move, IJKL to change look dir, F to move down, R to move up").unwrap();
        writeln!(console, "0-9 to place a block, - to delete a block, ESC to exit.").unwrap();
        loop {
            let (prev_pos, hit_pos) = world.get_hit_pos(pos, forward);
            let mut new_pos = pos;
            let Some(b) = console.try_read() else {
                break;
            };
            match b {
                b'w' | b'W' => new_pos = pos + forward0 * Fix64::from_rat(1, 4),
                b's' | b'S' => new_pos = pos - forward0 * Fix64::from_rat(1, 4),
                b'f' | b'F' => new_pos = pos + down0 * Fix64::from_rat(1, 4),
                b' ' | b'r' | b'R' => new_pos = pos - down0 * Fix64::from_rat(1, 4),
                b'd' | b'D' => new_pos = pos + right * Fix64::from_rat(1, 4),
                b'a' | b'A' => new_pos = pos - right * Fix64::from_rat(1, 4),
                b'i' | b'I' => phi_over_pi += Fix64::from_rat(1, 32),
                b'k' | b'K' => phi_over_pi -= Fix64::from_rat(1, 32),
                b'l' | b'L' => theta_over_pi += Fix64::from_rat(1, 32),
                b'j' | b'J' => theta_over_pi -= Fix64::from_rat(1, 32),
                b'0'..=b'9' => {
                    if let Some(prev_pos) = prev_pos {
                        if prev_pos != pos.map(Fix64::floor) {
                            world.get_mut(prev_pos).unwrap().color =
                                Some(NEW_BLOCK_COLORS[(b - b'0') as usize].to_packed());
                        }
                    }
                }
                b'\x08' | b'-' => {
                    if let Some(hit_pos) = hit_pos {
                        *world.get_mut(hit_pos).unwrap() = Block::default();
                    }
                }
                b'\x1B' => {
                    writeln!(console).unwrap();
                    exit(0);
                }
                _ => {}
            }
            theta_over_pi %= Fix64::from(2i64);
            phi_over_pi = phi_over_pi.clamp(Fix64::from_rat(-1, 2), Fix64::from_rat(1, 2));
            if world.get(new_pos.map(Fix64::floor)) == Some(&Block::default()) {
                pos = new_pos;
            }
        }
    }
}
