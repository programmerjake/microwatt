use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

macro_rules! impl_assign_op {
    ($AssignOp:ident::$assign_fn:ident => $Op:ident::$op_fn:ident) => {
        impl<L, R> $AssignOp<R> for Vec3D<L>
        where
            Self: $Op<R, Output = Self> + Clone,
        {
            fn $assign_fn(&mut self, rhs: R) {
                *self = self.clone().$op_fn(rhs);
            }
        }
    };
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Vec3D<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3D<T> {
    pub fn map<R, F: FnMut(T) -> R>(self, mut f: F) -> Vec3D<R> {
        Vec3D {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
        }
    }
    pub fn as_ref(&self) -> Vec3D<&T> {
        let Vec3D { x, y, z } = self;
        Vec3D { x, y, z }
    }
    pub fn zip<R>(self, rhs: Vec3D<R>) -> Vec3D<(T, R)> {
        Vec3D {
            x: (self.x, rhs.x),
            y: (self.y, rhs.y),
            z: (self.z, rhs.z),
        }
    }
    pub fn into_array(self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
    pub fn from_array(v: [T; 3]) -> Self {
        let [x, y, z] = v;
        Self { x, y, z }
    }
    pub fn dot<Rhs, R>(self, rhs: Vec3D<Rhs>) -> R
    where
        R: Add<Output = R>,
        T: Mul<Rhs, Output = R>,
    {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    pub fn abs_sq<R>(self) -> R
    where
        R: Add<Output = R>,
        T: Mul<T, Output = R> + Clone,
    {
        let rhs = self.clone();
        self.dot(rhs)
    }
}

impl Vec3D<i64> {
    pub const fn sub_const(self, r: Self) -> Self {
        Vec3D {
            x: self.x - r.x,
            y: self.y - r.y,
            z: self.z - r.z,
        }
    }
    pub const fn dot_const(self, rhs: Vec3D<i64>) -> i64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    pub const fn abs_sq_const(self) -> i64 {
        self.dot_const(self)
    }
}

impl<T: Neg> Neg for Vec3D<T> {
    type Output = Vec3D<T::Output>;

    fn neg(self) -> Self::Output {
        Vec3D {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<L, R> Add<Vec3D<R>> for Vec3D<L>
where
    L: Add<R>,
{
    type Output = Vec3D<L::Output>;

    fn add(self, r: Vec3D<R>) -> Self::Output {
        Vec3D {
            x: self.x + r.x,
            y: self.y + r.y,
            z: self.z + r.z,
        }
    }
}

impl_assign_op!(AddAssign::add_assign => Add::add);

impl<L, R> Sub<Vec3D<R>> for Vec3D<L>
where
    L: Sub<R>,
{
    type Output = Vec3D<L::Output>;

    fn sub(self, r: Vec3D<R>) -> Self::Output {
        Vec3D {
            x: self.x - r.x,
            y: self.y - r.y,
            z: self.z - r.z,
        }
    }
}

impl_assign_op!(SubAssign::sub_assign => Sub::sub);

impl<L, R> Mul<R> for Vec3D<L>
where
    L: Mul<R>,
    R: Clone,
{
    type Output = Vec3D<L::Output>;

    fn mul(self, r: R) -> Self::Output {
        Vec3D {
            x: self.x * r.clone(),
            y: self.y * r.clone(),
            z: self.z * r,
        }
    }
}

impl_assign_op!(MulAssign::mul_assign => Mul::mul);

impl<L, R> Div<R> for Vec3D<L>
where
    L: Div<R>,
    R: Clone,
{
    type Output = Vec3D<L::Output>;

    fn div(self, r: R) -> Self::Output {
        Vec3D {
            x: self.x / r.clone(),
            y: self.y / r.clone(),
            z: self.z / r,
        }
    }
}

impl_assign_op!(DivAssign::div_assign => Div::div);
