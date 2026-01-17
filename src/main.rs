use anyhow::Result;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod app_state;
mod camera;
mod texture;

pub use app::*;
pub use app_state::*;
pub use camera::*;
pub use texture::*;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
