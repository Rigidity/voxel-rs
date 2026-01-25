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

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x + EPSILON
            && self.max.x > other.min.x - EPSILON
            && self.min.y < other.max.y + EPSILON
            && self.max.y > other.min.y - EPSILON
            && self.min.z < other.max.z + EPSILON
            && self.max.z > other.min.z - EPSILON
    }
}
