use glam::Vec3;
use winit::keyboard::KeyCode;

use crate::{Aabb, Input};

#[derive(Debug, Clone)]
pub struct Player {
    pub position: Vec3,
    pub size: Vec3,
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
    pub eye_height: f32,
}

impl Player {
    pub fn new(position: Vec3, size: Vec3, eye_height: f32) -> Self {
        Self {
            position,
            size,
            yaw_degrees: -90.0,
            pitch_degrees: 0.0,
            eye_height,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.position, self.size)
    }

    pub fn camera_position(&self) -> Vec3 {
        self.position + Vec3::new(self.size.x / 2.0, self.eye_height, self.size.z / 2.0)
    }

    pub fn update(&mut self, input: &mut Input, delta: f32) {
        let speed = 10.0 * delta;
        let rotation_speed = 100.0 * delta;

        let (sin_yaw, cos_yaw) = self.yaw_degrees.to_radians().sin_cos();
        let forward = Vec3::new(cos_yaw, 0.0, sin_yaw).normalize();
        let right = Vec3::new(-sin_yaw, 0.0, cos_yaw).normalize();

        if input.is_key_pressed(KeyCode::KeyA) {
            self.position -= right * speed;
        }

        if input.is_key_pressed(KeyCode::KeyD) {
            self.position += right * speed;
        }

        if input.is_key_pressed(KeyCode::KeyW) {
            self.position += forward * speed;
        }

        if input.is_key_pressed(KeyCode::KeyS) {
            self.position -= forward * speed;
        }

        if input.is_key_pressed(KeyCode::Space) {
            self.position.y += speed;
        }

        if input.is_key_pressed(KeyCode::ShiftLeft) {
            self.position.y -= speed;
        }

        if input.is_key_pressed(KeyCode::ArrowLeft) {
            self.yaw_degrees -= rotation_speed;
        }

        if input.is_key_pressed(KeyCode::ArrowRight) {
            self.yaw_degrees += rotation_speed;
        }

        if input.is_key_pressed(KeyCode::ArrowUp) {
            self.pitch_degrees += rotation_speed;
        }

        if input.is_key_pressed(KeyCode::ArrowDown) {
            self.pitch_degrees -= rotation_speed;
        }

        if input.is_key_just_pressed(KeyCode::Escape) {
            input.set_mouse_locked(!input.is_mouse_locked());
        }

        if input.is_mouse_locked() {
            let delta = input.mouse_motion();
            let sensitivity = 0.1;

            self.yaw_degrees += delta.x * sensitivity;
            self.pitch_degrees -= delta.y * sensitivity;
        }

        self.pitch_degrees = self.pitch_degrees.clamp(-89.0, 89.0);
    }
}
