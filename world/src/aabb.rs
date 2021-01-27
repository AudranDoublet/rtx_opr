use nalgebra::{Vector3, Vector4, Matrix4};

use crate::{BlockFace, World};

#[derive(Debug)]
pub struct AABB {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

pub struct AABBIterator {
    min: Vector3<i32>,
    max: Vector3<i32>,
    curr: Vector3<i32>,
}

impl AABB {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> AABB {
        AABB {
            min,
            max,
        }
    }

    pub fn new_from_coords(positions: &[Vector3<f32>]) -> AABB {
        let mut min_x = std::f32::INFINITY;
        let mut min_y = std::f32::INFINITY;
        let mut min_z = std::f32::INFINITY;
        let mut max_x = std::f32::NEG_INFINITY;
        let mut max_y = std::f32::NEG_INFINITY;
        let mut max_z = std::f32::NEG_INFINITY;

        for pos in positions {
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            min_z = min_z.min(pos.z);

            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
            max_z = max_z.max(pos.z);
        }

        AABB::new(Vector3::new(min_x, min_y, min_z), Vector3::new(max_x, max_y, max_z))
    }

    pub fn rotate(&self, transformation: Matrix4<f32>) -> AABB {
        let transform = |v: Vector3<f32>| {
            let v = Vector4::new(v.x, v.y, v.z, 1.0);
            let t = transformation * v;

            Vector3::new(t.x / t.w, t.y / t.w, t.z / t.w)
        };

        let mut points = vec![
            transform(self.min),
        ];

        let diff = self.max - self.min;

        for i in 0..=1 {
            for j in 0..=1 {
                for k in 0..=1 {
                    points.push(
                        transform(self.min + diff.component_mul(&Vector3::new(i as f32, j as f32, k as f32)))
                    );
                }
            }
        }

        Self::new_from_coords(&points)
    }

    pub fn translate(&self, diff: Vector3<f32>) -> AABB {
        AABB::new(self.min + diff, self.max + diff)
    }

    pub fn translate3(&self, x: f32, y: f32, z: f32) -> AABB {
        self.translate(Vector3::new(x, y, z))
    }

    pub fn blocks(&self) -> AABBIterator {
        let min = Vector3::new(self.min.x.floor() as i32, self.min.y.floor() as i32, self.min.z.floor() as i32);
        let max = Vector3::new(self.max.x.floor() as i32, self.max.y.floor() as i32, self.max.z.floor() as i32);

        AABBIterator {
            min: min,
            max: max,
            curr: min,
        }
    }

    pub fn has_blocks(&self, world: &World) -> bool {
        self.blocks()
            .filter_map(|p| world.block_at(p))
            .filter_map(|b| b.aabb(Vector3::zeros()))
            .count() > 0
    }

    pub fn augment(&self, diff: Vector3<f32>) -> AABB {
        let mut min = self.min;
        let mut max = self.max;

        match diff.x {
            x if x < 0.0 => min.x += x,
            x            => max.x += x,
        }

        match diff.y {
            y if y < 0.0 => min.y += y,
            y            => max.y += y,
        }

        match diff.z {
            z if z < 0.0 => min.z += z,
            z            => max.z += z,
        }

        AABB::new(min, max)
    }

    pub fn augment3(&self, x: f32, y: f32, z: f32) -> AABB {
        self.augment(Vector3::new(x, y, z))
    }

    pub fn intersects_coord(&self, other: &AABB, coord: usize) -> bool {
        other.max[coord] > self.min[coord] && other.min[coord] < self.max[coord]
    }

    pub fn offset(&self, other: &AABB, coord: usize, offset: f32) -> f32 {
        let c2 = (coord + 1) % 3;
        let c3 = (coord + 2) % 3;

        if self.intersects_coord(other, c2) && self.intersects_coord(other, c3) {

            if offset > 0.0 && other.max[coord] <= self.min[coord] {
                let diff = self.min[coord] - other.max[coord];

                if diff < offset {
                    return diff;
                }
            }

            if offset < 0.0 && other.min[coord] >= self.max[coord] {
                let diff = self.max[coord] - other.min[coord];

                if diff > offset {
                    return diff;
                }
            }
        }

        offset
    }

    pub fn extend(&self, diff: Vector3<f32>) -> AABB {
        AABB::new(self.min - diff, self.max + diff)
    }

    pub fn box_intersects(&self, other: &AABB) -> bool {
        for i in 0..3 {
            if !(self.min[i] < other.max[i] && self.max[i] > other.min[i]) {
                return false;
            }
        }

        return true;
    }

    pub fn ray_intersects(&self, origin: Vector3<f32>, inv_direction: Vector3<f32>) -> Option<(f32, BlockFace)> {
        let a = (self.min - origin).component_mul(&inv_direction);
        let b = (self.max - origin).component_mul(&inv_direction);

        let min = Vector3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
        let max = Vector3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

        let mut face = BlockFace::Up;
        let mut t1 = std::f32::NEG_INFINITY;

        for i in 0..3 {
            if min[i] > t1 {
                t1 = min[i];
                face = BlockFace::coord(i);

                if inv_direction[i] > 0.0 {
                    face = face.opposite();
                }
            }
        }

        let t2 = max.x.min(max.y).min(max.z);

        if t1 > 0.0 && t1 < t2 {
            Some((t1, face))
        } else {
            None
        }
    }
}

impl Iterator for AABBIterator {
    type Item = Vector3<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.z > self.max.z {
            None
        } else {
            let res = self.curr;

            self.curr.x += 1;

            if self.curr.x > self.max.x {
                self.curr.x = self.min.x;
                self.curr.y += 1;

                if self.curr.y > self.max.y {
                    self.curr.y = self.min.y;
                    self.curr.z += 1;
                }
            }

            Some(res)
        }
    }
}
