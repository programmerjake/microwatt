#[cfg(feature = "hosted")]
use core::fmt;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Shl, ShlAssign, Shr,
    ShrAssign, Sub, SubAssign,
};

macro_rules! impl_assign_op {
    ($AssignOp:ident::$assign_fn:ident => $Op:ident::$op_fn:ident) => {
        impl<Rhs> $AssignOp<Rhs> for Fix64
        where
            Self: $Op<Rhs, Output = Self>,
        {
            fn $assign_fn(&mut self, rhs: Rhs) {
                *self = self.$op_fn(rhs);
            }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Fix64(i64);

#[cfg(feature = "hosted")]
impl fmt::Display for Fix64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let frac_digits = (Fix64::FRAC_BITS + 3) / 4;
        let v = self.0.unsigned_abs();
        if self.0 < 0 {
            write!(f, "-")?;
        }
        let trunc = v >> Self::FRAC_BITS;
        let fract = v as u128 & Self::FRAC_MASK as u128;
        let fract = (fract << 4 * frac_digits) >> Fix64::FRAC_BITS;
        write!(
            f,
            "0x{trunc:x}.{fract:0digits$x}",
            digits = frac_digits as usize,
        )
    }
}

#[cfg(feature = "hosted")]
impl fmt::Debug for Fix64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Fix64 {
    pub const FRAC_BITS: u32 = 24;
    pub const INT_MASK: i64 = (!0i64) << Self::FRAC_BITS;
    pub const FRAC_MASK: i64 = !Self::INT_MASK;
    pub const fn from_bits(v: i64) -> Self {
        Self(v)
    }
    pub const fn as_bits(self) -> i64 {
        self.0
    }
    pub const fn from_int(v: i64) -> Self {
        Self(v << Self::FRAC_BITS)
    }
    pub const fn from_rat(num: i64, denom: i64) -> Self {
        Self((((num as i128) << Self::FRAC_BITS) / denom as i128) as i64)
    }
    #[cfg(feature = "hosted")]
    pub fn from_f32(v: f32) -> Self {
        Self((v * (1u64 << Self::FRAC_BITS) as f32) as i64)
    }
    #[cfg(feature = "hosted")]
    pub fn to_f32(self) -> f32 {
        self.0 as f32 * (1.0 / (1u64 << Self::FRAC_BITS) as f32)
    }
    #[cfg(feature = "hosted")]
    pub fn from_f64(v: f64) -> Self {
        Self((v * (1u64 << Self::FRAC_BITS) as f64) as i64)
    }
    #[cfg(feature = "hosted")]
    pub fn to_f64(self) -> f64 {
        self.0 as f64 * (1.0 / (1u64 << Self::FRAC_BITS) as f64)
    }
    pub const fn floor_fract(self) -> Self {
        Self(self.0 & Self::FRAC_MASK)
    }
    pub const fn floor(self) -> i64 {
        self.0 >> Self::FRAC_BITS
    }
    pub const fn round(self) -> i64 {
        Self(self.0 + Self::from_rat(1, 2).0).floor()
    }
    pub const fn ceil(self) -> i64 {
        (self.0 + Self::FRAC_MASK) >> Self::FRAC_BITS
    }
    pub const fn trunc(self) -> i64 {
        self.0 / Self::from_int(1).0
    }
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }
    pub const fn is_positive(self) -> bool {
        self.0 > 0
    }
    pub const fn signum(self) -> i64 {
        self.0.signum()
    }
    /// Computes `(self * a) + b)` rounding once at the end
    pub const fn mul_add(self, a: Self, b: Self) -> Self {
        let prod = self.0 as i128 * a.0 as i128;
        let sum = prod + ((b.0 as i128) << Self::FRAC_BITS);
        Self((sum >> Self::FRAC_BITS) as i64)
    }
}

#[cfg(feature = "hosted")]
impl From<Fix64> for f32 {
    fn from(value: Fix64) -> Self {
        value.to_f32()
    }
}

#[cfg(feature = "hosted")]
impl From<Fix64> for f64 {
    fn from(value: Fix64) -> Self {
        value.to_f64()
    }
}

#[cfg(feature = "hosted")]
impl From<f32> for Fix64 {
    fn from(value: f32) -> Self {
        Self::from_f32(value)
    }
}

#[cfg(feature = "hosted")]
impl From<f64> for Fix64 {
    fn from(value: f64) -> Self {
        Self::from_f64(value)
    }
}

impl From<i64> for Fix64 {
    fn from(value: i64) -> Self {
        Self::from_int(value)
    }
}

impl Add for Fix64 {
    type Output = Self;

    fn add(self, rhs: Fix64) -> Self::Output {
        Fix64(self.0 + rhs.0)
    }
}

impl_assign_op!(AddAssign::add_assign => Add::add);

impl Sub for Fix64 {
    type Output = Self;

    fn sub(self, rhs: Fix64) -> Self::Output {
        Fix64(self.0 - rhs.0)
    }
}

impl_assign_op!(SubAssign::sub_assign => Sub::sub);

impl Mul for Fix64 {
    type Output = Self;

    fn mul(self, rhs: Fix64) -> Self::Output {
        Fix64((self.0 as i128 * rhs.0 as i128 >> Self::FRAC_BITS) as i64)
    }
}

impl_assign_op!(MulAssign::mul_assign => Mul::mul);

impl Div for Fix64 {
    type Output = Self;

    fn div(self, rhs: Fix64) -> Self::Output {
        Fix64((((self.0 as i128) << Self::FRAC_BITS) / rhs.0 as i128) as i64)
    }
}

impl_assign_op!(DivAssign::div_assign => Div::div);

impl Rem for Fix64 {
    type Output = Self;

    fn rem(self, rhs: Fix64) -> Self::Output {
        Fix64(self.0 % rhs.0)
    }
}

impl_assign_op!(RemAssign::rem_assign => Rem::rem);

impl<Rhs> Shl<Rhs> for Fix64
where
    i64: Shl<Rhs, Output = i64>,
{
    type Output = Self;

    fn shl(self, rhs: Rhs) -> Self::Output {
        Fix64(self.0 << rhs)
    }
}

impl_assign_op!(ShlAssign::shl_assign => Shl::shl);

impl<Rhs> Shr<Rhs> for Fix64
where
    i64: Shr<Rhs, Output = i64>,
{
    type Output = Self;

    fn shr(self, rhs: Rhs) -> Self::Output {
        Fix64(self.0 >> rhs)
    }
}

impl_assign_op!(ShrAssign::shr_assign => Shr::shr);

impl Neg for Fix64 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Fix64(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix64_display() {
        assert_eq!(
            Fix64::from_bits(0x123456789abcdef).to_string(),
            "0x123456789.abcdef"
        );
        assert_eq!(
            Fix64::from_bits(-0x123456789abcdef).to_string(),
            "-0x123456789.abcdef"
        );
        assert_eq!(Fix64::from_bits(-0x3C00001).to_string(), "-0x3.c00001");
    }
}
