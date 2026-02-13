use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl ModelVertex {
    pub fn new(position: [f32; 3], uv: [f32; 2], normal: [f32; 3]) -> Self {
        Self {
            position,
            uv,
            normal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub vertices: Vec<ModelVertex>,
}

impl Model {
    pub fn new(vertices: Vec<ModelVertex>) -> Self {
        Self { vertices }
    }

    /// Creates a standard cube model with 24 vertices (4 per face)
    pub fn cube() -> Self {
        let vertices = vec![
            // Front face (+Z)
            ModelVertex::new([1.0, 1.0, 1.0], [1.0, 0.0], [0.0, 0.0, 1.0]),
            ModelVertex::new([0.0, 1.0, 1.0], [0.0, 0.0], [0.0, 0.0, 1.0]),
            ModelVertex::new([0.0, 0.0, 1.0], [0.0, 1.0], [0.0, 0.0, 1.0]),
            ModelVertex::new([1.0, 0.0, 1.0], [1.0, 1.0], [0.0, 0.0, 1.0]),
            // Back face (-Z)
            ModelVertex::new([0.0, 1.0, 0.0], [1.0, 0.0], [0.0, 0.0, -1.0]),
            ModelVertex::new([1.0, 1.0, 0.0], [0.0, 0.0], [0.0, 0.0, -1.0]),
            ModelVertex::new([1.0, 0.0, 0.0], [0.0, 1.0], [0.0, 0.0, -1.0]),
            ModelVertex::new([0.0, 0.0, 0.0], [1.0, 1.0], [0.0, 0.0, -1.0]),
            // Left face (-X)
            ModelVertex::new([0.0, 1.0, 1.0], [1.0, 0.0], [-1.0, 0.0, 0.0]),
            ModelVertex::new([0.0, 1.0, 0.0], [0.0, 0.0], [-1.0, 0.0, 0.0]),
            ModelVertex::new([0.0, 0.0, 0.0], [0.0, 1.0], [-1.0, 0.0, 0.0]),
            ModelVertex::new([0.0, 0.0, 1.0], [1.0, 1.0], [-1.0, 0.0, 0.0]),
            // Right face (+X)
            ModelVertex::new([1.0, 1.0, 0.0], [1.0, 0.0], [1.0, 0.0, 0.0]),
            ModelVertex::new([1.0, 1.0, 1.0], [0.0, 0.0], [1.0, 0.0, 0.0]),
            ModelVertex::new([1.0, 0.0, 1.0], [0.0, 1.0], [1.0, 0.0, 0.0]),
            ModelVertex::new([1.0, 0.0, 0.0], [1.0, 1.0], [1.0, 0.0, 0.0]),
            // Top face (+Y)
            ModelVertex::new([1.0, 1.0, 1.0], [1.0, 1.0], [0.0, 1.0, 0.0]),
            ModelVertex::new([1.0, 1.0, 0.0], [1.0, 0.0], [0.0, 1.0, 0.0]),
            ModelVertex::new([0.0, 1.0, 0.0], [0.0, 0.0], [0.0, 1.0, 0.0]),
            ModelVertex::new([0.0, 1.0, 1.0], [0.0, 1.0], [0.0, 1.0, 0.0]),
            // Bottom face (-Y)
            ModelVertex::new([1.0, 0.0, 0.0], [1.0, 1.0], [0.0, -1.0, 0.0]),
            ModelVertex::new([1.0, 0.0, 1.0], [1.0, 0.0], [0.0, -1.0, 0.0]),
            ModelVertex::new([0.0, 0.0, 1.0], [0.0, 0.0], [0.0, -1.0, 0.0]),
            ModelVertex::new([0.0, 0.0, 0.0], [0.0, 1.0], [0.0, -1.0, 0.0]),
        ];

        Self::new(vertices)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelId(pub u8);

#[derive(Debug, Default)]
pub struct ModelRegistry {
    models: HashMap<String, ModelId>,
    model_data: Vec<Model>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_model(&mut self, name: String, model: Model) -> ModelId {
        let id = ModelId(self.model_data.len() as u8);
        self.models.insert(name, id);
        self.model_data.push(model);
        id
    }

    pub fn get_model_id(&self, name: &str) -> Option<ModelId> {
        self.models.get(name).copied()
    }

    pub fn get_model(&self, id: ModelId) -> Option<&Model> {
        self.model_data.get(id.0 as usize)
    }

    /// Flattens all model vertices into a single buffer for GPU upload
    pub fn flatten_to_buffer(&self) -> Vec<f32> {
        let mut buffer = Vec::new();

        for model in &self.model_data {
            for vertex in &model.vertices {
                // Pack: position (3), uv (2), normal (3) = 8 floats per vertex
                buffer.extend_from_slice(&vertex.position);
                buffer.extend_from_slice(&vertex.uv);
                buffer.extend_from_slice(&vertex.normal);
            }
        }

        buffer
    }

    /// Returns the starting vertex index for a given model in the flattened buffer
    pub fn get_model_offset(&self, id: ModelId) -> u32 {
        let mut offset = 0;
        for i in 0..(id.0 as usize) {
            offset += self.model_data[i].vertices.len();
        }
        offset as u32
    }
}
