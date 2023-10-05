use crate::{console::Console, fixed::Fix64, take_once::TakeOnce, vec::Vec3D};
use core::{fmt::Write, num::NonZeroU8};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct PackedColor(NonZeroU8);

impl PackedColor {
    pub const R_STEPS: u32 = 7;
    pub const G_STEPS: u32 = 9;
    pub const B_STEPS: u32 = 4;
    pub const R_MAX: u32 = Self::R_STEPS - 1;
    pub const G_MAX: u32 = Self::G_STEPS - 1;
    pub const B_MAX: u32 = Self::B_STEPS - 1;
}

const _: () = {
    assert!(
        PackedColor::R_STEPS * PackedColor::G_STEPS * PackedColor::B_STEPS < u8::MAX as u32,
        "not enough space"
    );
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn as_vec3d(self) -> Vec3D<u8> {
        Vec3D {
            x: self.r,
            y: self.g,
            z: self.b,
        }
    }
    pub const fn from_vec3d(v: Vec3D<u8>) -> Self {
        Self {
            r: v.x,
            g: v.y,
            b: v.z,
        }
    }
    pub const fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }
    pub const fn white() -> Self {
        Self {
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
        }
    }
    pub const fn to_packed(self) -> PackedColor {
        let r = self.r as u32 * PackedColor::R_MAX / u8::MAX as u32;
        let g = self.g as u32 * PackedColor::G_MAX / u8::MAX as u32;
        let b = self.b as u32 * PackedColor::B_MAX / u8::MAX as u32;
        let mut retval = r;
        retval *= PackedColor::G_STEPS;
        retval += g;
        retval *= PackedColor::B_STEPS;
        retval += b;
        let Some(retval) = NonZeroU8::new((1 + retval) as u8) else {
            unreachable!();
        };
        PackedColor(retval)
    }
    pub const fn from_packed(v: PackedColor) -> Self {
        let mut v = v.0.get() as u32;
        v -= 1;
        let b = v % PackedColor::B_STEPS;
        v /= PackedColor::B_STEPS;
        let g = v % PackedColor::G_STEPS;
        v /= PackedColor::G_STEPS;
        let r = v % PackedColor::R_STEPS;
        let r = r * u8::MAX as u32 / PackedColor::R_MAX;
        let g = g * u8::MAX as u32 / PackedColor::G_MAX;
        let b = b * u8::MAX as u32 / PackedColor::B_MAX;
        Self {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        }
    }
}

impl Console {
    pub fn set_background_color(&mut self, color: RgbColor) {
        let RgbColor { r, g, b } = color;
        write!(self, "\x1B[48;2;{r};{g};{b}m").unwrap();
    }
    pub fn set_foreground_color(&mut self, color: RgbColor) {
        let RgbColor { r, g, b } = color;
        write!(self, "\x1B[38;2;{r};{g};{b}m").unwrap();
    }
}

pub struct Screen {
    pub pixels: [[RgbColor; Self::X_SIZE]; Self::Y_SIZE],
}

impl Screen {
    pub const X_SIZE: usize = 100;
    pub const Y_SIZE: usize = 75;
    pub fn pixel_dimensions(&self) -> (Fix64, Fix64) {
        (Fix64::from(1), Fix64::from(1))
    }
    pub fn take() -> &'static mut Screen {
        static SCREEN: TakeOnce<Screen> = TakeOnce::new(Screen {
            pixels: [[RgbColor { r: 0, g: 0, b: 0 }; Screen::X_SIZE]; Screen::Y_SIZE],
        });
        SCREEN.take().expect("screen already taken")
    }
    pub fn display(&self, console: &mut Console) {
        let mut last_bg = RgbColor::black();
        let mut last_fg = RgbColor::white();
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
                    .unwrap_or(RgbColor::black());
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
