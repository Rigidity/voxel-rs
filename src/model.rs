pub trait Model: 'static + Send + Sync {
    fn unique_name(&self) -> String;

    fn vertices(&self) -> Vec<ModelVertex>;
}

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
