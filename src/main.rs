#![allow(clippy::new_without_default)]

use anyhow::Result;
use winit::event_loop::{ControlFlow, EventLoop};

mod aabb;
mod app;
mod app_state;
mod block;
mod camera;
mod chunk;
mod chunk_data;
mod chunk_mesh;
mod game_state;
mod input;
mod player;
mod projection;
mod registry;
mod relevant_chunks;
mod renderer;
mod texture;
mod texture_array;
mod texture_array_builder;
mod voxel_pipeline;
mod world;
mod world_generator;

pub use aabb::*;
pub use app::*;
pub use app_state::*;
pub use block::*;
pub use camera::*;
pub use chunk::*;
pub use chunk_data::*;
pub use chunk_mesh::*;
pub use game_state::*;
pub use input::*;
pub use player::*;
pub use projection::*;
pub use registry::*;
pub use relevant_chunks::*;
pub use renderer::*;
pub use texture::*;
pub use texture_array::*;
pub use texture_array_builder::*;
pub use voxel_pipeline::*;
pub use world::*;
pub use world_generator::*;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
