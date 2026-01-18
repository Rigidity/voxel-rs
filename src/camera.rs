use glam::{Mat4, Vec3, Vec4};

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
}

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.to_radians(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn projection_matrix(&self) -> Mat4 {
        OPENGL_TO_WGPU_MATRIX
            * Mat4::perspective_rh_gl(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);
