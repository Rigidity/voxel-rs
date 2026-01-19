use std::collections::HashSet;

use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct Input {
    pressed_keys: HashSet<KeyCode>,
}

impl Input {
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn set_key_pressed(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.pressed_keys.insert(key);
        } else {
            self.pressed_keys.remove(&key);
        }
    }
}
