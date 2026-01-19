use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::WindowId,
};

use crate::{AppState, Window};

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

        let attributes = winit::window::Window::default_attributes()
            .with_inner_size(LogicalSize::new(1000, 600))
            .with_visible(false);

        let window = Window::new(event_loop.create_window(attributes).unwrap());
        window.center();
        window.set_visible(true);

        self.state = Some(pollster::block_on(AppState::new(window)).unwrap())
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
                        let size = state.window().size();
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
                    state.update_key_state(key, event.state.is_pressed());
                }
            }
            WindowEvent::CursorLeft { device_id: _ } => {
                state.update_mouse_position(None);
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                state.update_mouse_position(Some(position));
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            state.update_relative_mouse_position(delta);
        }
    }
}
