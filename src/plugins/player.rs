use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

use crate::{Aabb, Block, BlockKind, CollisionNormals, Velocity, World};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, update_player);
    }
}

const COYOTE_TIME: f32 = 0.075;

#[derive(Component)]
pub struct Player {
    grounded_timer: f32,
    pitch: f32,
    yaw: f32,
}

#[derive(Component)]
pub struct PlayerCamera;

fn setup_player(mut commands: Commands) {
    commands
        .spawn((
            Player {
                grounded_timer: 0.0,
                pitch: 0.0,
                yaw: 0.0,
            },
            Aabb::new(Vec3::ZERO, Vec3::new(0.6, 1.8, 0.6)),
            Velocity(Vec3::ZERO),
            Transform::from_xyz(0.0, 50.0, 0.0),
            Visibility::Visible,
        ))
        .with_children(|children| {
            children.spawn((
                PlayerCamera,
                Camera3d::default(),
                Projection::Perspective(PerspectiveProjection {
                    fov: 75.0f32.to_radians(),
                    ..Default::default()
                }),
                Transform::from_xyz(0.3, 1.5, 0.3),
            ));
        });
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn update_player(
    time: Res<Time>,
    mut world: ResMut<World>,
    mut player: Query<(&mut Player, &mut Velocity, &CollisionNormals)>,
    mut camera: Query<(&mut Transform, &GlobalTransform), (With<PlayerCamera>, Without<Player>)>,
    input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut cursor_options: Single<&mut CursorOptions>,
    mut mouse_motion_reader: MessageReader<MouseMotion>,
) {
    let delta = time.delta_secs();

    let (mut player, mut velocity, collision_normals) = player.single_mut().unwrap();
    let (mut camera, camera_global) = camera.single_mut().unwrap();

    if collision_normals.0.contains(&Vec3::Y) {
        player.grounded_timer = COYOTE_TIME;
    }

    player.grounded_timer = (player.grounded_timer - delta).max(0.0);

    let walk_speed = 7.0;
    let gravity = -36.0;
    let jump_velocity = 13.0;
    let rotation_speed = 2.0 * delta;

    // Scale acceleration with walk_speed to maintain responsive movement
    let ground_acceleration = walk_speed * 20.0;
    let air_acceleration = walk_speed * 10.0;
    let ground_friction = 0.91;
    let air_friction = 0.98;

    let (sin_yaw, cos_yaw) = player.yaw.sin_cos();
    let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw).normalize();
    let right = Vec3::new(cos_yaw, 0.0, -sin_yaw).normalize();

    let mut movement_dir = Vec3::ZERO;

    if input.pressed(KeyCode::KeyA) {
        movement_dir -= right;
    }

    if input.pressed(KeyCode::KeyD) {
        movement_dir += right;
    }

    if input.pressed(KeyCode::KeyW) {
        movement_dir += forward;
    }

    if input.pressed(KeyCode::KeyS) {
        movement_dir -= forward;
    }

    if movement_dir.length_squared() > 0.0 {
        movement_dir = movement_dir.normalize();
    }

    let target_velocity = movement_dir * walk_speed;

    let is_grounded = player.grounded_timer > 0.0;
    let acceleration = if is_grounded {
        ground_acceleration
    } else {
        air_acceleration
    };
    let friction = if is_grounded {
        ground_friction
    } else {
        air_friction
    };

    // Apply friction first
    velocity.0.x *= friction;
    velocity.0.z *= friction;

    // Then accelerate towards target velocity
    let current_horizontal = Vec3::new(velocity.0.x, 0.0, velocity.0.z);
    let velocity_diff = target_velocity - current_horizontal;
    let max_acceleration = acceleration * delta;
    let acceleration_force = if velocity_diff.length_squared() > 0.0 {
        let diff_length = velocity_diff.length();
        velocity_diff * (diff_length.min(max_acceleration) / diff_length)
    } else {
        Vec3::ZERO
    };

    velocity.0.x += acceleration_force.x;
    velocity.0.z += acceleration_force.z;

    if player.grounded_timer == 0.0 {
        velocity.0.y += gravity * delta;
    }

    velocity.0.y = velocity.0.y.max(-50.0);

    if input.pressed(KeyCode::Space) && player.grounded_timer > 0.0 {
        velocity.0.y = jump_velocity;
        player.grounded_timer = 0.0;
    }

    if input.pressed(KeyCode::ArrowLeft) {
        player.yaw += rotation_speed;
    }

    if input.pressed(KeyCode::ArrowRight) {
        player.yaw -= rotation_speed;
    }

    if input.pressed(KeyCode::ArrowUp) {
        player.pitch += rotation_speed;
    }

    if input.pressed(KeyCode::ArrowDown) {
        player.pitch -= rotation_speed;
    }

    if input.just_pressed(KeyCode::Escape) {
        if cursor_options.grab_mode == CursorGrabMode::Locked {
            cursor_options.grab_mode = CursorGrabMode::None;
            cursor_options.visible = true;
        } else {
            cursor_options.grab_mode = CursorGrabMode::Locked;
            cursor_options.visible = false;
        }
    }

    if cursor_options.grab_mode == CursorGrabMode::Locked {
        for event in mouse_motion_reader.read() {
            let sensitivity = 0.0025;

            player.yaw -= event.delta.x * sensitivity;
            player.pitch -= event.delta.y * sensitivity;
        }
    }

    player.pitch = player
        .pitch
        .clamp((-89.0f32).to_radians(), 89.0f32.to_radians());

    let (sin_yaw, cos_yaw) = player.yaw.sin_cos();
    let (sin_pitch, cos_pitch) = player.pitch.sin_cos();
    let forward_with_pitch =
        Vec3::new(-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch).normalize();

    if cursor_options.grab_mode == CursorGrabMode::Locked {
        if mouse_input.just_pressed(MouseButton::Left)
            && let Some(result) =
                voxel_raycast(camera_global.translation(), forward_with_pitch, 5.0, &world)
        {
            world.set_block(result.hit_position, None);
        } else if mouse_input.just_pressed(MouseButton::Right)
            && let Some(result) =
                voxel_raycast(camera_global.translation(), forward_with_pitch, 5.0, &world)
        {
            world.set_block(
                result.previous_position,
                Some(Block::new(BlockKind::Test, 0)),
            );
        }
    }

    camera.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
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
        if world.get_block(current_voxel).is_some() {
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
