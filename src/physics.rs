use bevy::prelude::*;

mod aabb;
mod move_and_slide;
mod swept_aabb;

pub use aabb::*;
pub use move_and_slide::*;
pub use swept_aabb::*;

use crate::{SharedRegistry, World};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, move_entities);
    }
}

#[derive(Debug, Default, Clone, Component)]
pub struct CollisionNormals(pub Vec<Vec3>);

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
            move_and_slide(
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
