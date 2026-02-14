use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Collision {
    pub time: f32,
    pub normal: Vec3,
}

pub fn swept_aabb(a: Vec3, ah: Vec3, b: Vec3, bh: Vec3, delta: Vec3) -> Collision {
    let m = b - (a + ah);
    let mh = ah + bh;

    let mut time = 1.0;
    let mut normal = Vec3::ZERO;

    let s = line_to_plane(Vec3::ZERO, delta, m, Vec3::NEG_X);

    if s >= 0.0
        && delta.x > 0.0
        && s < time
        && between(s * delta.y, m.y, m.y + mh.y)
        && between(s * delta.z, m.z, m.z + mh.z)
    {
        time = s;
        normal = Vec3::NEG_X;
    }

    let s = line_to_plane(Vec3::ZERO, delta, m + mh * Vec3::X, Vec3::X);

    if s >= 0.0
        && delta.x < 0.0
        && s < time
        && between(s * delta.y, m.y, m.y + mh.y)
        && between(s * delta.z, m.z, m.z + mh.z)
    {
        time = s;
        normal = Vec3::X;
    }

    let s = line_to_plane(Vec3::ZERO, delta, m, Vec3::NEG_Y);

    if s >= 0.0
        && delta.y > 0.0
        && s < time
        && between(s * delta.x, m.x, m.x + mh.x)
        && between(s * delta.z, m.z, m.z + mh.z)
    {
        time = s;
        normal = Vec3::NEG_Y;
    }

    let s = line_to_plane(Vec3::ZERO, delta, m + mh * Vec3::Y, Vec3::Y);

    if s >= 0.0
        && delta.y < 0.0
        && s < time
        && between(s * delta.x, m.x, m.x + mh.x)
        && between(s * delta.z, m.z, m.z + mh.z)
    {
        time = s;
        normal = Vec3::Y;
    }

    let s = line_to_plane(Vec3::ZERO, delta, m, Vec3::NEG_Z);

    if s >= 0.0
        && delta.z > 0.0
        && s < time
        && between(s * delta.x, m.x, m.x + mh.x)
        && between(s * delta.y, m.y, m.y + mh.y)
    {
        time = s;
        normal = Vec3::NEG_Z;
    }

    let s = line_to_plane(Vec3::ZERO, delta, m + mh * Vec3::Z, Vec3::Z);

    if s >= 0.0
        && delta.z < 0.0
        && s < time
        && between(s * delta.x, m.x, m.x + mh.x)
        && between(s * delta.y, m.y, m.y + mh.y)
    {
        time = s;
        normal = Vec3::Z;
    }

    Collision { time, normal }
}

fn line_to_plane(p: Vec3, u: Vec3, v: Vec3, n: Vec3) -> f32 {
    let n_dot_u = n.dot(u);

    if n_dot_u == 0.0 {
        return f32::INFINITY;
    }

    n.dot(v - p) / n_dot_u
}

fn between(value: f32, min: f32, max: f32) -> bool {
    value >= min && value <= max
}
