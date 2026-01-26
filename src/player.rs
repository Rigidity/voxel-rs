use glam::{IVec3, Vec3};
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{Aabb, Block, Input, World};

const COYOTE_TIME: f32 = 0.075;

#[derive(Debug, Clone)]
pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub size: Vec3,
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
    pub eye_height: f32,
    pub grounded_timer: f32,
}

impl Player {
    pub fn new(position: Vec3, size: Vec3, eye_height: f32) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            size,
            yaw_degrees: -90.0,
            pitch_degrees: 0.0,
            eye_height,
            grounded_timer: 0.0,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.position, self.size)
    }

    pub fn camera_position(&self) -> Vec3 {
        self.position + Vec3::new(self.size.x / 2.0, self.eye_height, self.size.z / 2.0)
    }

    pub fn update(&mut self, input: &mut Input, delta: f32, world: &mut World) {
        self.grounded_timer = (self.grounded_timer - delta).max(0.0);

        let walk_speed = 6.0;
        let gravity = -32.0;
        let jump_velocity = 10.0;
        let rotation_speed = 100.0 * delta;

        let (sin_yaw, cos_yaw) = self.yaw_degrees.to_radians().sin_cos();
        let forward = Vec3::new(cos_yaw, 0.0, sin_yaw).normalize();
        let right = Vec3::new(-sin_yaw, 0.0, cos_yaw).normalize();

        let mut movement_dir = Vec3::ZERO;

        if input.is_key_pressed(KeyCode::KeyA) {
            movement_dir -= right;
        }

        if input.is_key_pressed(KeyCode::KeyD) {
            movement_dir += right;
        }

        if input.is_key_pressed(KeyCode::KeyW) {
            movement_dir += forward;
        }

        if input.is_key_pressed(KeyCode::KeyS) {
            movement_dir -= forward;
        }

        if movement_dir.length_squared() > 0.0 {
            movement_dir = movement_dir.normalize();
        }

        self.velocity.x = movement_dir.x * walk_speed;
        self.velocity.z = movement_dir.z * walk_speed;

        if self.grounded_timer == 0.0 {
            self.velocity.y += gravity * delta;
        }

        self.velocity.y = self.velocity.y.max(-50.0);

        if input.is_key_pressed(KeyCode::Space) && self.grounded_timer > 0.0 {
            self.velocity.y = jump_velocity;
            self.grounded_timer = 0.0;
        }

        self.move_with_collision(self.velocity * delta, world);

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

        let (sin_yaw, cos_yaw) = self.yaw_degrees.to_radians().sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch_degrees.to_radians().sin_cos();
        let forward_with_pitch =
            Vec3::new(cos_yaw * cos_pitch, sin_pitch, sin_yaw * cos_pitch).normalize();

        if input.is_mouse_locked() {
            if input.is_mouse_button_just_pressed(MouseButton::Left)
                && let Some(result) =
                    voxel_raycast(self.camera_position(), forward_with_pitch, 5.0, world)
            {
                world.set_block(result.hit_position, Block::Air);
            } else if input.is_mouse_button_just_pressed(MouseButton::Right)
                && let Some(result) =
                    voxel_raycast(self.camera_position(), forward_with_pitch, 5.0, world)
            {
                world.set_block(result.previous_position, Block::Rock);
            }
        }
    }

    fn move_with_collision(&mut self, mut delta: Vec3, world: &World) {
        for _ in 0..3 {
            let collision = self
                .check_collision(delta, world)
                .into_iter()
                .min_by(|a, b| a.time.total_cmp(&b.time));

            if let Some(Collision { time, normal }) = collision {
                if normal.y == 1.0 {
                    self.grounded_timer = COYOTE_TIME;
                }

                const TOLERANCE: f32 = 1.0 / 4096.0;
                self.position += TOLERANCE * normal + delta * time;
                let remaining_time = 1.0 - time;
                delta = delta.reject_from_normalized(normal) * remaining_time;
                self.velocity = self.velocity.reject_from_normalized(normal);
            } else {
                self.position += delta;
                return;
            }
        }
    }

    fn check_collision(&self, delta: Vec3, world: &World) -> Vec<Collision> {
        let mut collisions = Vec::new();

        let source = self.aabb();

        let target = source.translate(delta);
        let min = Vec3::min(source.min(), target.min()).floor();
        let max = Vec3::max(source.max(), target.max()).ceil();

        for x in min.x as i32..max.x as i32 {
            for y in min.y as i32..max.y as i32 {
                for z in min.z as i32..max.z as i32 {
                    let block_pos = IVec3::new(x, y, z);
                    if let Some(block) = world.get_block(block_pos)
                        && let Some(block_aabb) = block.aabb(block_pos)
                    {
                        collisions.extend(swept_aabb(delta, source, block_aabb))
                    }
                }
            }
        }

        collisions
    }
}

struct Collision {
    time: f32,
    normal: Vec3,
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

struct VoxelRaycastResult {
    hit_position: IVec3,
    previous_position: IVec3,
}

fn voxel_raycast(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    world: &World,
) -> Option<VoxelRaycastResult> {
    let direction = direction.normalize();

    let mut current_voxel = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );

    let step_dir = IVec3::new(
        direction.x.signum() as i32,
        direction.y.signum() as i32,
        direction.z.signum() as i32,
    );

    let t_delta = Vec3::new(
        (1.0 / direction.x).abs(),
        (1.0 / direction.y).abs(),
        (1.0 / direction.z).abs(),
    );

    let mut t_max = Vec3::new(
        if direction.x != 0.0 {
            (current_voxel.x as f32 + if step_dir.x > 0 { 1.0 } else { 0.0 } - origin.x)
                / direction.x
        } else {
            f32::MAX
        },
        if direction.y != 0.0 {
            (current_voxel.y as f32 + if step_dir.y > 0 { 1.0 } else { 0.0 } - origin.y)
                / direction.y
        } else {
            f32::MAX
        },
        if direction.z != 0.0 {
            (current_voxel.z as f32 + if step_dir.z > 0 { 1.0 } else { 0.0 } - origin.z)
                / direction.z
        } else {
            f32::MAX
        },
    );

    let mut distance = 0.0;
    let mut previous_voxel = current_voxel;

    while distance < max_distance {
        if world
            .get_block(current_voxel)
            .is_some_and(|block| block.is_solid())
        {
            return Some(VoxelRaycastResult {
                hit_position: current_voxel,
                previous_position: previous_voxel,
            });
        }

        previous_voxel = current_voxel;

        if t_max.x < t_max.y && t_max.x < t_max.z {
            current_voxel.x += step_dir.x;
            distance = t_max.x;
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.z {
            current_voxel.y += step_dir.y;
            distance = t_max.y;
            t_max.y += t_delta.y;
        } else {
            current_voxel.z += step_dir.z;
            distance = t_max.z;
            t_max.z += t_delta.z;
        }
    }

    None
}
