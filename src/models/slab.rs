use crate::{Model, ModelVertex};

pub struct Slab;

impl Model for Slab {
    fn unique_name(&self) -> String {
        "slab".to_string()
    }

    fn vertices(&self) -> Vec<ModelVertex> {
        vec![
            // Front face (+Z)
            ModelVertex::new([1.0, 0.5, 1.0], [1.0, 0.0], [0.0, 0.0, 1.0]),
            ModelVertex::new([0.0, 0.5, 1.0], [0.0, 0.0], [0.0, 0.0, 1.0]),
            ModelVertex::new([0.0, 0.0, 1.0], [0.0, 0.5], [0.0, 0.0, 1.0]),
            ModelVertex::new([1.0, 0.0, 1.0], [1.0, 0.5], [0.0, 0.0, 1.0]),
            // Back face (-Z)
            ModelVertex::new([0.0, 0.5, 0.0], [1.0, 0.0], [0.0, 0.0, -1.0]),
            ModelVertex::new([1.0, 0.5, 0.0], [0.0, 0.0], [0.0, 0.0, -1.0]),
            ModelVertex::new([1.0, 0.0, 0.0], [0.0, 0.5], [0.0, 0.0, -1.0]),
            ModelVertex::new([0.0, 0.0, 0.0], [1.0, 0.5], [0.0, 0.0, -1.0]),
            // Left face (-X)
            ModelVertex::new([0.0, 0.5, 1.0], [1.0, 0.0], [-1.0, 0.0, 0.0]),
            ModelVertex::new([0.0, 0.5, 0.0], [0.0, 0.0], [-1.0, 0.0, 0.0]),
            ModelVertex::new([0.0, 0.0, 0.0], [0.0, 0.5], [-1.0, 0.0, 0.0]),
            ModelVertex::new([0.0, 0.0, 1.0], [1.0, 0.5], [-1.0, 0.0, 0.0]),
            // Right face (+X)
            ModelVertex::new([1.0, 0.5, 0.0], [1.0, 0.0], [1.0, 0.0, 0.0]),
            ModelVertex::new([1.0, 0.5, 1.0], [0.0, 0.0], [1.0, 0.0, 0.0]),
            ModelVertex::new([1.0, 0.0, 1.0], [0.0, 0.5], [1.0, 0.0, 0.0]),
            ModelVertex::new([1.0, 0.0, 0.0], [1.0, 0.5], [1.0, 0.0, 0.0]),
            // Top face (+Y)
            ModelVertex::new([1.0, 0.5, 1.0], [1.0, 1.0], [0.0, 1.0, 0.0]),
            ModelVertex::new([1.0, 0.5, 0.0], [1.0, 0.0], [0.0, 1.0, 0.0]),
            ModelVertex::new([0.0, 0.5, 0.0], [0.0, 0.0], [0.0, 1.0, 0.0]),
            ModelVertex::new([0.0, 0.5, 1.0], [0.0, 1.0], [0.0, 1.0, 0.0]),
            // Bottom face (-Y)
            ModelVertex::new([1.0, 0.0, 0.0], [1.0, 1.0], [0.0, -1.0, 0.0]),
            ModelVertex::new([1.0, 0.0, 1.0], [1.0, 0.0], [0.0, -1.0, 0.0]),
            ModelVertex::new([0.0, 0.0, 1.0], [0.0, 0.0], [0.0, -1.0, 0.0]),
            ModelVertex::new([0.0, 0.0, 0.0], [0.0, 1.0], [0.0, -1.0, 0.0]),
        ]
    }
}
