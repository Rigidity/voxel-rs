use glam::Vec3;

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
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }
}
