use nalgebra::Vector3;

#[derive(Debug)]
pub struct AABB {
    min: Vector3<f32>,
    max: Vector3<f32>,
}

pub struct AABBIterator {
    min: Vector3<i32>,
    max: Vector3<i32>,
    curr: Vector3<i32>,
}

impl AABB {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> AABB {
        AABB {
            min: min,
            max: max,
        }
    }

    pub fn translate(&self, diff: Vector3<f32>) -> AABB {
        AABB::new(self.min + diff, self.max + diff)
    }

    pub fn translate3(&self, x: f32, y: f32, z: f32) -> AABB {
        self.translate(Vector3::new(x, y, z))
    }

    pub fn blocks(&self) -> AABBIterator {
        let min = Vector3::new(self.min.x.floor() as i32, self.min.y.floor() as i32, self.min.z.floor() as i32);
        let max = Vector3::new(self.max.x.ceil() as i32, self.max.y.ceil() as i32, self.max.z.ceil() as i32);

        AABBIterator {
            min: min,
            max: max,
            curr: min,
        }
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
