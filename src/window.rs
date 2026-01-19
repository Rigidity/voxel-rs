use std::{collections::HashSet, sync::Arc};

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    keyboard::KeyCode,
    window::CursorGrabMode,
};

pub struct Window {
    window: Arc<winit::window::Window>,
    just_pressed_keys: HashSet<KeyCode>,
    pressed_keys: HashSet<KeyCode>,
    is_mouse_locked: bool,
    mouse_position: Option<PhysicalPosition<f64>>,
    mouse_delta: (f64, f64),
}

impl From<&Window> for wgpu::SurfaceTarget<'_> {
    fn from(value: &Window) -> Self {
        value.window.clone().into()
    }
}

impl Window {
    pub fn new(window: winit::window::Window) -> Self {
        Self {
            window: Arc::new(window),
            just_pressed_keys: HashSet::new(),
            pressed_keys: HashSet::new(),
            is_mouse_locked: false,
            mouse_position: None,
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn center(&self) {
        if let Some(monitor) = self.window.current_monitor() {
            let screen_size = monitor.size();
            let window_size = self.window.outer_size();

            self.window.set_outer_position(PhysicalPosition {
                x: screen_size.width.saturating_sub(window_size.width) as f64 / 2.
                    + monitor.position().x as f64,
                y: screen_size.height.saturating_sub(window_size.height) as f64 / 2.
                    + monitor.position().y as f64,
            });
        }
    }

    pub fn get_center_position(&self) -> PhysicalPosition<f64> {
        let size = self.size();

        PhysicalPosition::new(size.width as f64 / 2., size.height as f64 / 2.)
    }

    pub fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible);
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    pub fn update_mouse_position(&mut self, position: Option<PhysicalPosition<f64>>) {
        self.mouse_position = position;
    }

    pub fn update_relative_mouse_position(&mut self, delta: (f64, f64)) {
        self.mouse_delta.0 += delta.0;
        self.mouse_delta.1 += delta.1;
    }

    pub fn update_key_state(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.pressed_keys.insert(key);
            self.just_pressed_keys.insert(key);
        } else {
            self.pressed_keys.remove(&key);
        }
    }

    pub fn clear_input(&mut self) {
        self.just_pressed_keys.clear();
        self.mouse_delta = (0.0, 0.0);
    }

    pub fn is_mouse_locked(&self) -> bool {
        self.is_mouse_locked
    }

    pub fn set_mouse_locked(&mut self, locked: bool) {
        self.is_mouse_locked = locked;

        if locked {
            self.window.set_cursor_visible(false);

            let center = self.get_center_position();

            if let Err(error) = self.window.set_cursor_position(center) {
                log::warn!("Failed to set cursor position: {error}");
            }

            if let Err(error) = self.window.set_cursor_grab(CursorGrabMode::Locked) {
                log::warn!("Failed to lock mouse: {error}");
            }
        } else {
            if let Err(error) = self.window.set_cursor_grab(CursorGrabMode::None) {
                log::warn!("Failed to unlock mouse: {error}");
            }

            self.window.set_cursor_visible(true);
        }
    }
}
