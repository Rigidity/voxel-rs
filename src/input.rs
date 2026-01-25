use std::collections::HashSet;

use glam::Vec2;
use winit::keyboard::KeyCode;

pub struct Input {
    mouse_motion: Vec2,
    is_mouse_locked: bool,
    just_pressed_keys: HashSet<KeyCode>,
    pressed_keys: HashSet<KeyCode>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_motion: Vec2::ZERO,
            is_mouse_locked: false,
            just_pressed_keys: HashSet::new(),
            pressed_keys: HashSet::new(),
        }
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

    pub fn finish_tick(&mut self) {
        self.just_pressed_keys.clear();
        self.mouse_motion = Vec2::ZERO;
    }
}
