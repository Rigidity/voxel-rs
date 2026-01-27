use std::{collections::HashMap, sync::LazyLock};

use crate::{BlockType, Dirt};

#[derive(Default)]
pub struct Registry {
    block_types: Vec<Box<dyn BlockType>>,
    block_ids: HashMap<&'static str, u16>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block_id(&self, name: &str) -> u16 {
        self.block_ids[name]
    }

    pub fn block_type(&self, id: u16) -> &dyn BlockType {
        &*self.block_types[id as usize]
    }

    fn register(&mut self, block_type: Box<dyn BlockType>) {
        let name = block_type.base_name();
        let id = self.block_types.len() as u16;
        self.block_types.push(block_type);
        self.block_ids.insert(name, id);
    }
}

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
    let mut registry = Registry::new();
    registry.register(Box::new(Dirt));
    registry
});
