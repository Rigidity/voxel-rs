use glam::Vec3;

pub const EPSILON: f32 = 0.00001;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    pub fn new(position: Vec3, size: Vec3) -> Self {
        let min = position;
        let max = position + size;
        Self { min, max }
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn translate(&self, translation: Vec3) -> Self {
        Self {
            min: self.min + translation,
            max: self.max + translation,
        }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x + EPSILON
            && self.max.x > other.min.x - EPSILON
            && self.min.y < other.max.y + EPSILON
            && self.max.y > other.min.y - EPSILON
            && self.min.z < other.max.z + EPSILON
            && self.max.z > other.min.z - EPSILON
    }
}
