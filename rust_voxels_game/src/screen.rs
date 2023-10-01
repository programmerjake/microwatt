use crate::{console::Console, fixed::Fix64, take_once::TakeOnce};
use core::fmt::Write;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color(pub u8);

impl Color {
    pub const fn default() -> Color {
        Color(0)
    }
}

impl Console {
    pub fn set_background_color(&mut self, color: Color) {
        write!(self, "\x1B[48;5;{}m", color.0).unwrap();
    }
    pub fn set_foreground_color(&mut self, color: Color) {
        write!(self, "\x1B[38;5;{}m", color.0).unwrap();
    }
}

pub struct Screen {
    pub pixels: [[Color; Self::X_SIZE]; Self::Y_SIZE],
}

impl Screen {
    pub const X_SIZE: usize = 80;
    pub const Y_SIZE: usize = 50;
    pub fn pixel_dimensions(&self) -> (Fix64, Fix64) {
        (Fix64::from(1), Fix64::from(1))
    }
    pub fn take() -> &'static mut Screen {
        static SCREEN: TakeOnce<Screen> = TakeOnce::new(Screen {
            pixels: [[Color(0); Screen::X_SIZE]; Screen::Y_SIZE],
        });
        SCREEN.take().expect("screen already taken")
    }
    pub fn display(&self, console: &mut Console) {
        let mut last_bg = Color::default();
        let mut last_fg = Color::default();
        write!(console, "\x1B[H").unwrap();
        for y in (0..Self::Y_SIZE).step_by(2) {
            console.set_background_color(last_bg);
            console.set_foreground_color(last_fg);
            for x in 0..Self::X_SIZE {
                let fg = self.pixels[y][x];
                let bg = self
                    .pixels
                    .get(y + 1)
                    .map(|row| row[x])
                    .unwrap_or(Color::default());
                if fg != last_fg {
                    console.set_foreground_color(fg);
                    last_fg = fg;
                }
                if bg != last_bg {
                    console.set_background_color(bg);
                    last_bg = bg;
                }
                write!(console, "\u{2580}").unwrap(); // upper half block
            }
            writeln!(console, "\x1B[m").unwrap();
        }
    }
}
