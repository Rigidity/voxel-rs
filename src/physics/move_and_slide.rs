use bevy::prelude::*;

use crate::{Aabb, Collision, CollisionNormals, Registry, World, swept_aabb};

pub fn move_and_slide(
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
