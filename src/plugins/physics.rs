use bevy::prelude::*;

use crate::World;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_entities);
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
    collision_normals: &mut CollisionNormals,
) {
    collision_normals.0.clear();

    for _ in 0..3 {
        let collision = check_collision(Aabb::new(*position, size), delta, world)
            .into_iter()
            .min_by(|a, b| a.time.total_cmp(&b.time));

        if let Some(Collision { time, normal }) = collision {
            collision_normals.0.push(normal);

            const TOLERANCE: f32 = 1.0 / 4096.0;
            *position += TOLERANCE * normal + delta * time;
            let remaining_time = 1.0 - time;
            delta = delta.reject_from_normalized(normal) * remaining_time;
            *velocity = velocity.reject_from_normalized(normal);
        } else {
            *position += delta;
            return;
        }
    }
}

struct Collision {
    time: f32,
    normal: Vec3,
}

fn check_collision(source: Aabb, delta: Vec3, world: &World) -> Vec<Collision> {
    let mut collisions = Vec::new();

    let target = source.translate(delta);
    let min = Vec3::min(source.min(), target.min()).floor();
    let max = Vec3::max(source.max(), target.max()).ceil();

    for x in min.x as i32..max.x as i32 {
        for y in min.y as i32..max.y as i32 {
            for z in min.z as i32..max.z as i32 {
                let block_pos = IVec3::new(x, y, z);
                if let Some(block) = world.get_block(block_pos)
                    && let Some(block_aabb) = block.kind.get_aabb(block.data).map(|aabb| {
                        aabb.translate(Vec3::new(
                            block_pos.x as f32,
                            block_pos.y as f32,
                            block_pos.z as f32,
                        ))
                    })
                {
                    collisions.extend(swept_aabb(delta, source, block_aabb))
                }
            }
        }
    }

    collisions
}

fn swept_aabb(delta: Vec3, moving: Aabb, still: Aabb) -> Option<Collision> {
    let before = still.min() - moving.max();
    let after = still.max() - moving.min();
    let positive = delta.cmpgt(Vec3::ZERO);
    let zero = delta.cmpeq(Vec3::ZERO);
    let entry = Vec3::select(positive, before, after);
    let entry = Vec3::select(zero, Vec3::NEG_INFINITY, entry / delta);

    let exit = Vec3::select(positive, after, before);
    let exit = Vec3::select(zero, Vec3::INFINITY, exit / delta);

    let time = entry.max_element();

    if !(0.0..=1.0).contains(&time) {
        return None;
    }

    if time > exit.min_element() {
        return None;
    }

    let normal = if time == entry.x && positive.x {
        Vec3::NEG_X
    } else if time == entry.x {
        Vec3::X
    } else if time == entry.y && positive.y {
        Vec3::NEG_Y
    } else if time == entry.y {
        Vec3::Y
    } else if positive.z {
        Vec3::NEG_Z
    } else {
        Vec3::Z
    };

    Some(Collision { time, normal })
}
