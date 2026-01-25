use std::collections::HashSet;

use glam::Vec2;
use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Default)]
pub struct Input {
    mouse_motion: Vec2,
    is_mouse_locked: bool,
    just_pressed_keys: HashSet<KeyCode>,
    pressed_keys: HashSet<KeyCode>,
    just_pressed_mouse_buttons: HashSet<MouseButton>,
    pressed_mouse_buttons: HashSet<MouseButton>,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mouse_motion(&self) -> Vec2 {
        self.mouse_motion
    }

    pub fn set_mouse_motion(&mut self, motion: Vec2) {
        self.mouse_motion = motion;
    }

    pub fn is_mouse_locked(&self) -> bool {
        self.is_mouse_locked
    }

    pub fn set_mouse_locked(&mut self, locked: bool) {
        self.is_mouse_locked = locked;
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    pub fn set_key_state(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.pressed_keys.insert(key);
            self.just_pressed_keys.insert(key);
        } else {
            self.pressed_keys.remove(&key);
        }
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed_mouse_buttons.contains(&button)
    }

    pub fn set_mouse_button_state(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            self.pressed_mouse_buttons.insert(button);
            self.just_pressed_mouse_buttons.insert(button);
        } else {
            self.pressed_mouse_buttons.remove(&button);
        }
    }

    pub fn finish_tick(&mut self) {
        self.just_pressed_keys.clear();
        self.just_pressed_mouse_buttons.clear();
        self.mouse_motion = Vec2::ZERO;
    }
}
