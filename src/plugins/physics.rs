use bevy::prelude::*;

use crate::{Registry, SharedRegistry, World};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, move_entities);
    }
}

#[derive(Debug, Default, Clone, Component)]
pub struct CollisionNormals(pub Vec<Vec3>);

#[derive(Debug, Clone, Copy, Component)]
#[require(CollisionNormals)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    pub fn new(position: Vec3, size: Vec3) -> Self {
        let min = position;
        let max = position + size;
        Self { min, max }
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn translate(&self, translation: Vec3) -> Self {
        Self {
            min: self.min + translation,
            max: self.max + translation,
        }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Velocity(pub Vec3);

fn move_entities(
    time: Res<Time>,
    world: Res<World>,
    registry: Res<SharedRegistry>,
    mut query: Query<(
        &mut Velocity,
        &mut Transform,
        Option<&Aabb>,
        Option<&mut CollisionNormals>,
    )>,
) {
    for (mut velocity, mut transform, aabb, collision_normals) in query.iter_mut() {
        let delta = velocity.0 * time.delta_secs();
        if let Some(aabb) = aabb
            && let Some(mut collision_normals) = collision_normals
        {
            move_with_collision(
                &mut velocity.0,
                aabb.size(),
                &mut transform.translation,
                delta,
                &world,
                &registry.0,
                &mut collision_normals,
            );
        } else {
            transform.translation += delta;
        }
    }
}

fn move_with_collision(
    velocity: &mut Vec3,
    size: Vec3,
    position: &mut Vec3,
    mut delta: Vec3,
    world: &World,
    registry: &Registry,
    collision_normals: &mut CollisionNormals,
) {
    collision_normals.0.clear();

    let mut previous_position = *position;

    loop {
        let source = Aabb::new(previous_position, size);
        let target = source.translate(delta);
        let min = Vec3::min(source.min(), target.min()).floor();
        let max = Vec3::max(source.max(), target.max()).ceil();

        let mut collision = Collision {
            time: 1.0,
            normal: Vec3::ZERO,
        };

        for x in min.x as i32..max.x as i32 {
            for y in min.y as i32..max.y as i32 {
                for z in min.z as i32..max.z as i32 {
                    let block_pos = IVec3::new(x, y, z);
                    if let Some(block) = world.get_block(block_pos)
                        && let Some(block_aabb) = registry
                            .block_type(block.id)
                            .get_aabb(block.data)
                            .map(|aabb| {
                                aabb.translate(Vec3::new(
                                    block_pos.x as f32,
                                    block_pos.y as f32,
                                    block_pos.z as f32,
                                ))
                            })
                    {
                        let c = swept_aabb(
                            previous_position,
                            size,
                            block_aabb.min(),
                            block_aabb.size(),
                            delta,
                        );

                        if c.time < collision.time {
                            collision = c;
                        }
                    }
                }
            }
        }

        let epsilon = 0.001;

        *position = previous_position + collision.time * delta + epsilon * collision.normal;

        if collision.time == 1.0 {
            break;
        }

        collision_normals.0.push(collision.normal);

        let b_dot_b = collision.normal.dot(collision.normal);

        if b_dot_b != 0.0 {
            previous_position = *position;

            let velocity_dot_normal = velocity.dot(collision.normal);
            *velocity -= velocity_dot_normal * collision.normal;

            let remaining = (1.0 - collision.time) * delta;
            let a_dot_b = remaining.dot(collision.normal);
            *position += remaining - (a_dot_b / b_dot_b) * collision.normal;
            delta = remaining - (a_dot_b / b_dot_b) * collision.normal;
        }
    }
}

struct Collision {
    time: f32,
    normal: Vec3,
}

fn swept_aabb(a: Vec3, ah: Vec3, b: Vec3, bh: Vec3, d: Vec3) -> Collision {
    let m = b - (a + ah);
    let mh = ah + bh;

    let mut time = 1.0;
    let mut normal = Vec3::ZERO;

    let s = line_to_plane(Vec3::ZERO, d, m, Vec3::NEG_X);

    if s >= 0.0
        && d.x > 0.0
        && s < time
        && between(s * d.y, m.y, m.y + mh.y)
        && between(s * d.z, m.z, m.z + mh.z)
    {
        time = s;
        normal = Vec3::NEG_X;
    }

    let s = line_to_plane(Vec3::ZERO, d, m + mh * Vec3::X, Vec3::X);

    if s >= 0.0
        && d.x < 0.0
        && s < time
        && between(s * d.y, m.y, m.y + mh.y)
        && between(s * d.z, m.z, m.z + mh.z)
    {
        time = s;
        normal = Vec3::X;
    }

    let s = line_to_plane(Vec3::ZERO, d, m, Vec3::NEG_Y);

    if s >= 0.0
        && d.y > 0.0
        && s < time
        && between(s * d.x, m.x, m.x + mh.x)
        && between(s * d.z, m.z, m.z + mh.z)
    {
        time = s;
        normal = Vec3::NEG_Y;
    }

    let s = line_to_plane(Vec3::ZERO, d, m + mh * Vec3::Y, Vec3::Y);

    if s >= 0.0
        && d.y < 0.0
        && s < time
        && between(s * d.x, m.x, m.x + mh.x)
        && between(s * d.z, m.z, m.z + mh.z)
    {
        time = s;
        normal = Vec3::Y;
    }

    let s = line_to_plane(Vec3::ZERO, d, m, Vec3::NEG_Z);

    if s >= 0.0
        && d.z > 0.0
        && s < time
        && between(s * d.x, m.x, m.x + mh.x)
        && between(s * d.y, m.y, m.y + mh.y)
    {
        time = s;
        normal = Vec3::NEG_Z;
    }

    let s = line_to_plane(Vec3::ZERO, d, m + mh * Vec3::Z, Vec3::Z);

    if s >= 0.0
        && d.z < 0.0
        && s < time
        && between(s * d.x, m.x, m.x + mh.x)
        && between(s * d.y, m.y, m.y + mh.y)
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
