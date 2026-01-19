use glam::{Mat4, Vec3};
use winit::keyboard::KeyCode;

use crate::Window;

pub struct Camera {
    pub position: Vec3,
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
}

impl Camera {
    pub fn new(position: Vec3, yaw_degrees: f32, pitch_degrees: f32) -> Self {
        Self {
            position,
            yaw_degrees,
            pitch_degrees,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch_degrees.to_radians().sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw_degrees.to_radians().sin_cos();

        Mat4::look_to_rh(
            self.position,
            Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::Y,
        )
    }

    pub fn update(&mut self, window: &mut Window, delta_time: f32) {
        let speed = 10.0 * delta_time;
        let rotation_speed = 100.0 * delta_time;

        let (sin_yaw, cos_yaw) = self.yaw_degrees.to_radians().sin_cos();
        let forward = Vec3::new(cos_yaw, 0.0, sin_yaw).normalize();
        let right = Vec3::new(-sin_yaw, 0.0, cos_yaw).normalize();

        if window.is_key_pressed(KeyCode::KeyA) {
            self.position -= right * speed;
        }

        if window.is_key_pressed(KeyCode::KeyD) {
            self.position += right * speed;
        }

        if window.is_key_pressed(KeyCode::KeyW) {
            self.position += forward * speed;
        }

        if window.is_key_pressed(KeyCode::KeyS) {
            self.position -= forward * speed;
        }

        if window.is_key_pressed(KeyCode::Space) {
            self.position.y += speed;
        }

        if window.is_key_pressed(KeyCode::ShiftLeft) {
            self.position.y -= speed;
        }

        if window.is_key_pressed(KeyCode::ArrowLeft) {
            self.yaw_degrees -= rotation_speed;
        }

        if window.is_key_pressed(KeyCode::ArrowRight) {
            self.yaw_degrees += rotation_speed;
        }

        if window.is_key_pressed(KeyCode::ArrowUp) {
            self.pitch_degrees += rotation_speed;
        }

        if window.is_key_pressed(KeyCode::ArrowDown) {
            self.pitch_degrees -= rotation_speed;
        }

        if window.is_key_just_pressed(KeyCode::Escape) {
            window.set_mouse_locked(!window.is_mouse_locked());
        }

        if window.is_mouse_locked() {
            let delta = window.mouse_delta();
            let sensitivity = 0.1;

            self.yaw_degrees += delta.0 as f32 * sensitivity;
            self.pitch_degrees -= delta.1 as f32 * sensitivity;
        }

        self.pitch_degrees = self.pitch_degrees.clamp(-89.0, 89.0);
    }
}
