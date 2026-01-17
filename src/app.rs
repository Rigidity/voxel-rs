use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use crate::AppState;

#[derive(Default)]
pub struct App {
    state: Option<AppState>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(1000, 600))
            .with_visible(false);

        let window = event_loop.create_window(attributes).unwrap();

        if let Some(monitor) = window.current_monitor() {
            let screen_size = monitor.size();
            let window_size = window.outer_size();

            window.set_outer_position(PhysicalPosition {
                x: screen_size.width.saturating_sub(window_size.width) as f64 / 2.
                    + monitor.position().x as f64,
                y: screen_size.height.saturating_sub(window_size.height) as f64 / 2.
                    + monitor.position().y as f64,
            });
        }

        window.set_visible(true);

        self.state = Some(pollster::block_on(AppState::new(Arc::new(window))).unwrap())
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                state.update();

                if let Err(error) = state.render() {
                    if matches!(
                        error,
                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated
                    ) {
                        let size = state.window().inner_size();
                        state.resize(size.width, size.height);
                    } else {
                        log::error!("An error occurred while rendering: {error}");
                    }
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    state.set_key_pressed(key, event.state.is_pressed());
                }
            }
            _ => (),
        }
    }
}
