use crate::{
    fixed::Fix64,
    screen::{Color, Screen},
    take_once::TakeOnce,
    vec::Vec3D,
};
use core::ops::ControlFlow;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Block {
    pub color: Color,
}

impl Block {
    pub const fn is_empty(&self) -> bool {
        self.color.0 == Color::default().0
    }
    pub const fn default() -> Self {
        Block {
            color: Color::default(),
        }
    }
}

pub struct World {
    pub blocks: [[[Block; Self::SIZE]; Self::SIZE]; Self::SIZE],
}

struct RayCastDimension {
    next_pos: i64,
    next_t: Fix64,
    t_step: Fix64,
    pos_step: i64,
}

impl RayCastDimension {
    fn new(start: Fix64, dir: Fix64) -> Option<Self> {
        let pos_step = dir.signum();
        if pos_step == 0 {
            return None;
        }
        let inv_dir = Fix64::from(1) / dir;
        let next_pos = start.floor() + pos_step;
        let target = if pos_step > 0 {
            Fix64::from(next_pos)
        } else {
            Fix64::from(next_pos) + Fix64::from(1)
        };
        let next_t = (target - start) * inv_dir;

        let retval = RayCastDimension {
            next_pos,
            next_t,
            t_step: inv_dir.abs(),
            pos_step,
        };

        Some(retval)
    }
    fn step(&mut self) {
        self.next_t += self.t_step;
        self.next_pos += self.pos_step;
    }
}

impl World {
    pub const SIZE: usize = 40;
    pub const ARRAY_AXIS_ORIGIN: i64 = Self::SIZE as i64 / -2;
    pub const ARRAY_ORIGIN: Vec3D<i64> = Vec3D {
        x: Self::ARRAY_AXIS_ORIGIN,
        y: Self::ARRAY_AXIS_ORIGIN,
        z: Self::ARRAY_AXIS_ORIGIN,
    };
    const fn init_block(pos: Vec3D<i64>) -> Block {
        let mut block = Block {
            color: Color((pos.x * 157 + pos.y * 246 + pos.z * 43 + 123) as u8),
        };
        const SPHERES: &[(Vec3D<i64>, i64, Color)] = &[
            (Vec3D { x: 0, y: 0, z: 0 }, 10 * 10, Color::default()),
            (
                Vec3D {
                    x: -5,
                    y: -5,
                    z: -5,
                },
                3 * 3,
                Color(3),
            ),
            (
                Vec3D {
                    x: -5,
                    y: 5,
                    z: 5,
                },
                3 * 3,
                Color(6),
            ),
            (
                Vec3D {
                    x: 5,
                    y: 5,
                    z: -5,
                },
                3 * 3,
                Color(5),
            ),
            (
                Vec3D {
                    x: 5,
                    y: -5,
                    z: 5,
                },
                3 * 3,
                Color(7),
            ),
        ];
        let mut sphere_idx = 0;
        while sphere_idx < SPHERES.len() {
            let (sphere_pos, r_sq, sphere_color) = SPHERES[sphere_idx];
            if pos.sub_const(sphere_pos).abs_sq_const() <= r_sq {
                block.color = sphere_color;
            }
            sphere_idx += 1;
        }
        block
    }
    const fn new() -> World {
        let mut retval = Self {
            blocks: [[[Block::default(); Self::SIZE]; Self::SIZE]; Self::SIZE],
        };
        let mut array_pos = Vec3D { x: 0, y: 0, z: 0 };
        while array_pos.x < Self::SIZE {
            array_pos.y = 0;
            while array_pos.y < Self::SIZE {
                array_pos.z = 0;
                while array_pos.z < Self::SIZE {
                    let pos = Self::from_array_pos(array_pos);
                    retval.blocks[array_pos.z][array_pos.y][array_pos.x] = Self::init_block(pos);
                    array_pos.z += 1;
                }
                array_pos.y += 1;
            }
            array_pos.x += 1;
        }
        retval
    }
    pub fn take() -> &'static mut World {
        #[allow(long_running_const_eval)]
        static WORLD: TakeOnce<World> = TakeOnce::new(World::new());
        WORLD.take().expect("world already taken")
    }
    /// out-of-range inputs produce wrapping outputs
    pub const fn from_array_pos(array_pos: Vec3D<usize>) -> Vec3D<i64> {
        Vec3D {
            x: (array_pos.x as i64).wrapping_add(Self::ARRAY_ORIGIN.x),
            y: (array_pos.y as i64).wrapping_add(Self::ARRAY_ORIGIN.y),
            z: (array_pos.z as i64).wrapping_add(Self::ARRAY_ORIGIN.z),
        }
    }
    /// out-of-range inputs produce wrapping outputs
    pub fn array_pos(pos: Vec3D<i64>) -> Vec3D<usize> {
        pos.zip(Self::ARRAY_ORIGIN)
            .map(|(pos, ao)| pos.wrapping_sub(ao) as usize)
    }
    pub fn get_array_mut(&mut self, array_pos: Vec3D<usize>) -> Option<&mut Block> {
        self.blocks
            .get_mut(array_pos.z)?
            .get_mut(array_pos.y)?
            .get_mut(array_pos.x)
    }
    pub fn get_array(&self, array_pos: Vec3D<usize>) -> Option<&Block> {
        self.blocks
            .get(array_pos.z)?
            .get(array_pos.y)?
            .get(array_pos.x)
    }
    pub fn get_mut(&mut self, pos: Vec3D<i64>) -> Option<&mut Block> {
        let array_pos = Self::array_pos(pos);
        self.get_array_mut(array_pos)
    }
    pub fn get(&self, pos: Vec3D<i64>) -> Option<&Block> {
        let array_pos = Self::array_pos(pos);
        self.get_array(array_pos)
    }
    pub fn array_positions() -> impl Iterator<Item = Vec3D<usize>> {
        (0..Self::SIZE).flat_map(|x| {
            (0..Self::SIZE).flat_map(move |y| (0..Self::SIZE).map(move |z| Vec3D { x, y, z }))
        })
    }
    pub fn positions() -> impl Iterator<Item = Vec3D<i64>> {
        Self::array_positions().map(Self::from_array_pos)
    }
    fn cast_ray_impl(
        &self,
        start: Vec3D<Fix64>,
        dir: Vec3D<Fix64>,
        mut f: impl FnMut(Vec3D<i64>, &Block) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        let mut f = move |pos| {
            let Some(block) = self.get(pos) else {
                return ControlFlow::Break(());
            };
            f(pos, block)
        };
        let mut pos = start.map(Fix64::floor).into_array();
        let mut ray_casters = start
            .zip(dir)
            .map(|(start, dir)| RayCastDimension::new(start, dir))
            .into_array();
        loop {
            f(Vec3D::from_array(pos))?;
            let mut min_index = None;
            let mut min_t = Fix64::from_bits(i64::MAX);
            for (index, ray_caster) in ray_casters.iter().enumerate() {
                let Some(ray_caster) = ray_caster else {
                    continue;
                };
                if ray_caster.next_t < min_t {
                    min_t = ray_caster.next_t;
                    min_index = Some(index);
                }
            }
            let Some(min_index) = min_index else {
                return ControlFlow::Break(());
            };
            let ray_caster = ray_casters[min_index].as_mut().unwrap();
            pos[min_index] = ray_caster.next_pos;
            ray_caster.step();
        }
    }
    pub fn cast_ray(
        &self,
        start: Vec3D<Fix64>,
        dir: Vec3D<Fix64>,
        f: impl FnMut(Vec3D<i64>, &Block) -> ControlFlow<()>,
    ) {
        let _ = self.cast_ray_impl(start, dir, f);
    }
    pub fn get_hit_pos(
        &self,
        start: Vec3D<Fix64>,
        forward: Vec3D<Fix64>,
    ) -> (Option<Vec3D<i64>>, Option<Vec3D<i64>>) {
        let mut prev_pos = None;
        let mut hit_pos = None;
        self.cast_ray(start, forward, |pos, block| {
            if block.is_empty() {
                prev_pos = Some(pos);
                ControlFlow::Continue(())
            } else {
                hit_pos = Some(pos);
                ControlFlow::Break(())
            }
        });
        (prev_pos, hit_pos)
    }
    pub fn render(
        &self,
        screen: &mut Screen,
        start: Vec3D<Fix64>,
        forward: Vec3D<Fix64>,
        right: Vec3D<Fix64>,
        down: Vec3D<Fix64>,
    ) {
        let (pixel_x_dim, pixel_y_dim) = screen.pixel_dimensions();
        let screen_x_size = Fix64::from(Screen::X_SIZE as i64);
        let screen_y_size = Fix64::from(Screen::Y_SIZE as i64);
        let screen_x_center = screen_x_size / Fix64::from(2i64);
        let screen_y_center = screen_y_size / Fix64::from(2i64);
        let screen_x_dim = pixel_x_dim * screen_x_size;
        let screen_y_dim = pixel_y_dim * screen_y_size;
        let screen_min_dim = screen_x_dim.min(screen_y_dim);
        let screen_x_factor = screen_x_dim / screen_min_dim;
        let screen_y_factor = screen_y_dim / screen_min_dim;
        let right_factor_inc = Fix64::from(2) * screen_x_factor / screen_x_size;
        let down_factor_inc = Fix64::from(2) * screen_y_factor / screen_y_size;
        for (y, row) in screen.pixels.iter_mut().enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                let right_factor = (Fix64::from(x as i64) - screen_x_center) * right_factor_inc;
                let down_factor = (Fix64::from(y as i64) - screen_y_center) * down_factor_inc;
                let dir = forward + right * right_factor + down * down_factor;
                let mut color = Color::default();
                self.cast_ray(start, dir, |_pos, block| {
                    if block.is_empty() {
                        ControlFlow::Continue(())
                    } else {
                        color = block.color;
                        ControlFlow::Break(())
                    }
                });
                *pixel = color;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_cast() {
        let world = World::new();
        let valid_steps = &[
            Vec3D { x: -1, y: 0, z: 0 },
            Vec3D { x: 1, y: 0, z: 0 },
            Vec3D { x: 0, y: -1, z: 0 },
            Vec3D { x: 0, y: 1, z: 0 },
            Vec3D { x: 0, y: 0, z: -1 },
            Vec3D { x: 0, y: 0, z: 1 },
        ];
        let check_cast_ray = |dir, expected_visited: &[_]| {
            let mut visited = Vec::new();
            world.cast_ray(
                Vec3D {
                    x: Fix64::from(0.0),
                    y: Fix64::from(0.0),
                    z: Fix64::from(0.0),
                },
                dir,
                |pos, _block| {
                    visited.push(pos);
                    ControlFlow::Continue(())
                },
            );
            assert_eq!(expected_visited, &*visited, "dir={dir:?}");
            for i in visited.windows(2) {
                let diff = i[0] - i[1];
                assert!(valid_steps.contains(&diff), "diff={diff:?} dir={dir:?}");
            }
        };
        check_cast_ray(
            Vec3D {
                x: Fix64::from(-1.0 / 8.0),
                y: Fix64::from(0.0),
                z: Fix64::from(1.0),
            },
            &[
                Vec3D { x: 0, y: 0, z: 0 },
                Vec3D { x: -1, y: 0, z: 0 },
                Vec3D { x: -1, y: 0, z: 1 },
                Vec3D { x: -1, y: 0, z: 2 },
                Vec3D { x: -1, y: 0, z: 3 },
                Vec3D { x: -1, y: 0, z: 4 },
                Vec3D { x: -1, y: 0, z: 5 },
                Vec3D { x: -1, y: 0, z: 6 },
                Vec3D { x: -1, y: 0, z: 7 },
                Vec3D { x: -2, y: 0, z: 7 },
                Vec3D { x: -2, y: 0, z: 8 },
                Vec3D { x: -2, y: 0, z: 9 },
                Vec3D { x: -2, y: 0, z: 10 },
                Vec3D { x: -2, y: 0, z: 11 },
                Vec3D { x: -2, y: 0, z: 12 },
                Vec3D { x: -2, y: 0, z: 13 },
                Vec3D { x: -2, y: 0, z: 14 },
                Vec3D { x: -2, y: 0, z: 15 },
                Vec3D { x: -3, y: 0, z: 15 },
                Vec3D { x: -3, y: 0, z: 16 },
                Vec3D { x: -3, y: 0, z: 17 },
                Vec3D { x: -3, y: 0, z: 18 },
                Vec3D { x: -3, y: 0, z: 19 },
            ],
        );
        check_cast_ray(
            Vec3D {
                x: Fix64::from(1.0 / 8.0),
                y: Fix64::from(0.0),
                z: Fix64::from(1.0),
            },
            &[
                Vec3D { x: 0, y: 0, z: 0 },
                Vec3D { x: 0, y: 0, z: 1 },
                Vec3D { x: 0, y: 0, z: 2 },
                Vec3D { x: 0, y: 0, z: 3 },
                Vec3D { x: 0, y: 0, z: 4 },
                Vec3D { x: 0, y: 0, z: 5 },
                Vec3D { x: 0, y: 0, z: 6 },
                Vec3D { x: 0, y: 0, z: 7 },
                Vec3D { x: 1, y: 0, z: 7 },
                Vec3D { x: 1, y: 0, z: 8 },
                Vec3D { x: 1, y: 0, z: 9 },
                Vec3D { x: 1, y: 0, z: 10 },
                Vec3D { x: 1, y: 0, z: 11 },
                Vec3D { x: 1, y: 0, z: 12 },
                Vec3D { x: 1, y: 0, z: 13 },
                Vec3D { x: 1, y: 0, z: 14 },
                Vec3D { x: 1, y: 0, z: 15 },
                Vec3D { x: 2, y: 0, z: 15 },
                Vec3D { x: 2, y: 0, z: 16 },
                Vec3D { x: 2, y: 0, z: 17 },
                Vec3D { x: 2, y: 0, z: 18 },
                Vec3D { x: 2, y: 0, z: 19 },
            ],
        );
    }
}
