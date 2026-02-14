use std::{
    cmp::Reverse,
    collections::HashMap,
    mem,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use bevy::{
    math::USizeVec3,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, Slider},
};
use indexmap::IndexMap;

use crate::{
    Block, BlockTextureArray, CHUNK_SIZE, ChunkData, ChunkMaterial, LightData, LightDataInner,
    Player, RegionManager, Registry, RelevantChunks, RelevantLights, SharedRegistry,
    WorldGenerator, generate_mesh, propagate_block_light, propagate_skylight,
};

#[derive(Resource)]
pub struct DayNightCycle {
    pub time_of_day: f32,
    pub day_length_seconds: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time_of_day: 2.0,
            day_length_seconds: 180.0,
        }
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .insert_resource(World {
                center_pos: IVec3::ZERO,
                region_manager: Arc::new(RegionManager::new(PathBuf::from("regions"))),
                generator: WorldGenerator::new(),
                generation_radius: 12,
                generation_tasks: IndexMap::new(),
                light_tasks: IndexMap::new(),
                mesh_tasks: IndexMap::new(),
                pending_mesh_results: HashMap::new(),
                chunks: IndexMap::new(),
                chunks_to_save: HashMap::new(),
                last_save_time: Instant::now(),
                save_task: None,
            })
            .insert_resource(DayNightCycle::default())
            .add_systems(
                Update,
                (update_day_night_cycle, update_world, save_chunks).chain_ignore_deferred(),
            )
            .add_systems(EguiPrimaryContextPass, debug_ui);
    }
}

#[derive(Resource)]
pub struct World {
    center_pos: IVec3,
    region_manager: Arc<RegionManager>,
    generator: WorldGenerator,
    generation_radius: i32,
    generation_tasks: IndexMap<IVec3, Task<GenerationResult>>,
    light_tasks: IndexMap<IVec3, Task<LightData>>,
    mesh_tasks: IndexMap<IVec3, Task<Option<Mesh>>>,
    pending_mesh_results: HashMap<IVec3, Option<Mesh>>,
    chunks: IndexMap<IVec3, Chunk>,
    chunks_to_save: HashMap<IVec3, ChunkData>,
    save_task: Option<Task<()>>,
    last_save_time: Instant,
}

impl World {
    pub fn is_visible_chunk(&self, chunk_pos: IVec3) -> bool {
        let horizontal_distance = Vec2::new(chunk_pos.x as f32, chunk_pos.z as f32).distance(
            Vec2::new(self.center_pos.x as f32, self.center_pos.z as f32),
        );

        if horizontal_distance > self.generation_radius as f32 {
            return false;
        }

        let vertical_distance = (chunk_pos.y as f32 - self.center_pos.y as f32).abs();

        if vertical_distance > self.generation_radius as f32 {
            return false;
        }

        true
    }

    pub fn get_chunk_data(&self, chunk_pos: IVec3) -> Option<ChunkData> {
        self.chunks.get(&chunk_pos).map(|chunk| chunk.data.clone())
    }

    pub fn get_light_data(&self, chunk_pos: IVec3) -> Option<LightData> {
        self.chunks.get(&chunk_pos).map(|chunk| chunk.light.clone())
    }

    pub fn get_block(&self, world_pos: IVec3) -> Option<Block> {
        let chunk_pos = Self::chunk_pos(world_pos);
        let local_pos = Self::local_pos(world_pos);
        let chunk = self.chunks.get(&chunk_pos)?;
        chunk.data.get_block(local_pos)
    }

    pub fn set_block(&mut self, world_pos: IVec3, block: Option<Block>) {
        let chunk_pos = Self::chunk_pos(world_pos);
        let local_pos = Self::local_pos(world_pos);

        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return;
        };

        Arc::make_mut(&mut chunk.data).set_block(local_pos, block);
        self.chunks_to_save.insert(chunk_pos, chunk.data.clone());

        // Block change invalidates light for this chunk and neighbors
        self.invalidate_light(chunk_pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.mesh_status = MeshStatus::Urgent;
        }

        for neighbor_pos in get_neighbors(chunk_pos) {
            self.invalidate_light(neighbor_pos);

            if let Some(neighbor) = self.chunks.get_mut(&neighbor_pos) {
                neighbor.mesh_status = MeshStatus::Urgent;
            }
        }
    }

    pub fn chunk_sort_key(&self, chunk_pos: IVec3) -> (i32, i32) {
        (
            IVec3::new(chunk_pos.x, self.center_pos.y, chunk_pos.z)
                .distance_squared(self.center_pos),
            (chunk_pos.y - self.center_pos.y).abs(),
        )
    }

    pub fn chunk_pos(world_pos: IVec3) -> IVec3 {
        world_pos.div_euclid(IVec3::splat(CHUNK_SIZE as i32))
    }

    pub fn local_pos(world_pos: IVec3) -> USizeVec3 {
        let local_pos = world_pos.rem_euclid(IVec3::splat(CHUNK_SIZE as i32));
        USizeVec3::new(
            local_pos.x as usize,
            local_pos.y as usize,
            local_pos.z as usize,
        )
    }

    fn update_center_pos(&mut self, position: Vec3) {
        let player_world_pos = IVec3::new(position.x as i32, position.y as i32, position.z as i32);
        self.center_pos = World::chunk_pos(player_world_pos);
    }

    fn chunks_to_unload(&self) -> Vec<IVec3> {
        self.chunks
            .keys()
            .copied()
            .filter(|chunk_pos| !self.is_visible_chunk(*chunk_pos))
            .collect()
    }

    /// Invalidate a chunk's light, cancelling any in-progress light task and
    /// marking it for recomputation. Also queues it for remeshing.
    fn invalidate_light(&mut self, chunk_pos: IVec3) {
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.light_status = LightStatus::Pending;
            if chunk.mesh_status != MeshStatus::Urgent {
                chunk.mesh_status = MeshStatus::Queued;
            }
        }
        self.light_tasks.swap_remove(&chunk_pos);
        self.mesh_tasks.swap_remove(&chunk_pos);
        self.pending_mesh_results.remove(&chunk_pos);
    }

    fn force_remesh(&mut self, chunk_pos: IVec3) {
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos)
            && chunk.mesh_status != MeshStatus::Urgent
        {
            chunk.mesh_status = MeshStatus::Queued;
        }

        self.mesh_tasks.swap_remove(&chunk_pos);
    }
}

struct Chunk {
    data: ChunkData,
    light: LightData,
    light_status: LightStatus,
    mesh_status: MeshStatus,
    entity: Entity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LightStatus {
    /// Light needs to be (re)computed.
    Pending,
    /// Light has been computed and is up to date.
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MeshStatus {
    /// The chunk already has a mesh and hasn't been modified.
    Complete,

    /// The chunk is visible and should be meshed based on distance from the player.
    Queued,

    /// The chunk should be remeshed first.
    Urgent,
}

fn update_day_night_cycle(
    time: Res<Time>,
    mut cycle: ResMut<DayNightCycle>,
    texture_array: Res<BlockTextureArray>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
) {
    cycle.time_of_day += time.delta_secs() / cycle.day_length_seconds;
    cycle.time_of_day %= 1.0;

    let angle = cycle.time_of_day * std::f32::consts::TAU;

    // Sun rotates around the X axis: at time 0.25 (noon) sun is overhead
    let sun_direction = Vec3::new(0.0, angle.cos(), angle.sin()).normalize();

    // Sun strength based on how high the sun is above the horizon
    let sun_height = sun_direction.y;
    let sun_strength = sun_height.max(0.0) * 0.6;

    // Sky brightness: full at noon, dim at night
    let sky_brightness = (sun_height * 2.0).clamp(0.05, 1.0);

    if let Some(material) = materials.get_mut(&texture_array.material) {
        material.sun_direction = sun_direction;
        material.sun_strength = sun_strength;
        material.sky_brightness = sky_brightness;
    }
}

fn update_world(
    mut commands: Commands,
    mut world: ResMut<World>,
    player: Query<&GlobalTransform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    texture_array: Res<BlockTextureArray>,
    shared_registry: Res<SharedRegistry>,
) {
    let player = player.single().unwrap();

    world.update_center_pos(player.translation());

    unload_far_chunks(&mut commands, &mut world);
    load_near_chunks(
        &mut commands,
        &mut world,
        texture_array.material.clone(),
        &shared_registry.0,
    );
    propagate_lights(&mut world, shared_registry.0.clone());
    regenerate_meshes(
        &mut commands,
        &mut world,
        &mut meshes,
        shared_registry.0.clone(),
    );
}

fn unload_far_chunks(commands: &mut Commands, world: &mut World) {
    let chunks_to_unload = world.chunks_to_unload();

    for chunk_pos in chunks_to_unload {
        if let Some(chunk) = world.chunks.swap_remove(&chunk_pos) {
            commands.entity(chunk.entity).despawn();
        }

        world.light_tasks.swap_remove(&chunk_pos);
        world.mesh_tasks.swap_remove(&chunk_pos);
        world.generation_tasks.swap_remove(&chunk_pos);

        for neighbor_pos in get_neighbors(chunk_pos) {
            world.invalidate_light(neighbor_pos);
        }
    }
}

fn load_near_chunks(
    commands: &mut Commands,
    world: &mut World,
    material: Handle<ChunkMaterial>,
    registry: &Arc<Registry>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    let mut chunks_to_generate = Vec::new();

    // Look for visible chunks in a cube around the player (not all of these are visible)
    for x in -world.generation_radius..=world.generation_radius {
        for y in -world.generation_radius..=world.generation_radius {
            for z in -world.generation_radius..=world.generation_radius {
                let chunk_pos = world.center_pos + IVec3::new(x, y, z);

                // Don't load chunks multiple times
                if world.is_visible_chunk(chunk_pos)
                    && !world.chunks.contains_key(&chunk_pos)
                    && !world.generation_tasks.contains_key(&chunk_pos)
                {
                    chunks_to_generate.push(chunk_pos);
                }
            }
        }
    }

    // Sort chunks by distance from the player
    chunks_to_generate.sort_by_key(|chunk_pos| world.chunk_sort_key(*chunk_pos));

    let mut exceeded = false;

    for chunk_pos in chunks_to_generate {
        if world.generation_tasks.len() >= 16 {
            exceeded = true;
            break;
        }

        let region_manager = world.region_manager.clone();
        let generator = world.generator.clone();
        let registry = registry.clone();

        let task = task_pool.spawn(async move {
            generate_chunk(&region_manager, &generator, chunk_pos, &registry)
        });

        world.generation_tasks.insert(chunk_pos, task);
    }

    let has_generations = !world.generation_tasks.is_empty();

    let mut results = HashMap::new();

    // Collect generated chunks, and remove the tasks from the map
    world
        .generation_tasks
        .retain(|chunk_pos, task| match check_ready(task) {
            Some(data) => {
                results.insert(*chunk_pos, data);

                false
            }
            None => true,
        });

    for (chunk_pos, result) in results {
        // New chunk affects light and meshes of neighbors
        for neighbor_pos in get_neighbors(chunk_pos) {
            world.invalidate_light(neighbor_pos);
        }

        // Add the entity to the world
        let entity = commands
            .spawn((
                Transform::from_xyz(
                    chunk_pos.x as f32 * CHUNK_SIZE as f32,
                    chunk_pos.y as f32 * CHUNK_SIZE as f32,
                    chunk_pos.z as f32 * CHUNK_SIZE as f32,
                ),
                Visibility::Visible,
                MeshMaterial3d(material.clone()),
            ))
            .id();

        // Insert the data and queue it for meshing
        if result.needs_saving {
            world
                .chunks_to_save
                .insert(chunk_pos, result.chunk_data.clone());
        }

        world.chunks.insert(
            chunk_pos,
            Chunk {
                data: result.chunk_data,
                light: Arc::new(LightDataInner::new()),
                light_status: LightStatus::Pending,
                mesh_status: MeshStatus::Queued,
                entity,
            },
        );
    }

    if has_generations && world.generation_tasks.is_empty() && !exceeded {
        log::info!(
            "Finished generating chunks, currently have {} loaded chunks",
            world.chunks.len()
        );
    }
}

/// Separate light propagation pass. Runs before meshing so that light data
/// is always up-to-date before any mesh is built.
fn propagate_lights(world: &mut World, registry: Arc<Registry>) {
    let task_pool = AsyncComputeTaskPool::get();

    // Find chunks that need light computation
    let mut chunks_to_light: Vec<IVec3> = world
        .chunks
        .iter()
        .filter_map(|(pos, chunk)| {
            if chunk.light_status == LightStatus::Pending && !world.light_tasks.contains_key(pos) {
                Some(*pos)
            } else {
                None
            }
        })
        .collect();

    chunks_to_light.sort_by_key(|pos| world.chunk_sort_key(*pos));

    for chunk_pos in chunks_to_light {
        if world.light_tasks.len() >= 16 {
            break;
        }

        // Need all neighbors loaded before we can compute light (for border seeding)
        let can_light = get_neighbors(chunk_pos)
            .iter()
            .all(|n| world.chunks.contains_key(n) || !world.is_visible_chunk(*n));
        if !can_light {
            continue;
        }

        let relevant_chunks = RelevantChunks::from_world(world, chunk_pos);
        let registry = registry.clone();

        // Build neighbor lights for block light border seeding
        let mut neighbor_lights =
            RelevantLights::new(chunk_pos, Arc::new(LightDataInner::new()));
        for neighbor_pos in get_neighbors(chunk_pos) {
            if let Some(light) = world.get_light_data(neighbor_pos) {
                neighbor_lights.add_neighbor(neighbor_pos, light);
            }
        }

        let task = task_pool.spawn(async move {
            let mut light = propagate_skylight(chunk_pos, &relevant_chunks, &registry);
            propagate_block_light(
                Arc::make_mut(&mut light),
                chunk_pos,
                &relevant_chunks,
                &neighbor_lights,
                &registry,
            );
            light
        });

        world.light_tasks.insert(chunk_pos, task);
    }

    // Collect completed light tasks
    let mut results = HashMap::new();

    world
        .light_tasks
        .retain(|chunk_pos, task| match check_ready(task) {
            Some(light) => {
                results.insert(*chunk_pos, light);
                false
            }
            None => true,
        });

    for (chunk_pos, light) in results {
        // Compare border block light with previous values. Only invalidate
        // face-adjacent neighbors if the border changed, preventing infinite loops.
        let border_changed = world
            .chunks
            .get(&chunk_pos)
            .is_some_and(|chunk| border_block_light_changed(&chunk.light, &light));

        if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
            chunk.light = light;
            chunk.light_status = LightStatus::Complete;

            // Force remesh so mesh picks up the new light data
            if chunk.mesh_status != MeshStatus::Urgent {
                chunk.mesh_status = MeshStatus::Queued;
            }
        }

        // Neighbors need remeshing (smooth lighting reads neighbor light)
        for neighbor_pos in get_neighbors(chunk_pos) {
            world.force_remesh(neighbor_pos);
        }

        // If border block light changed, face-adjacent neighbors must re-propagate
        if border_changed {
            for neighbor_pos in get_face_neighbors(chunk_pos) {
                if let Some(neighbor) = world.chunks.get(&neighbor_pos) {
                    if neighbor.light_status == LightStatus::Complete {
                        world.invalidate_light(neighbor_pos);
                    }
                }
            }
        }
    }
}

fn regenerate_meshes(
    commands: &mut Commands,
    world: &mut World,
    meshes: &mut Assets<Mesh>,
    registry: Arc<Registry>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    // We need to generate meshes a single time for queued chunks
    let mut chunks_to_mesh = world
        .chunks
        .keys()
        .copied()
        .map(|chunk_pos| (chunk_pos, world.chunks[&chunk_pos].mesh_status))
        .filter(|&(chunk_pos, status)| {
            status != MeshStatus::Complete
                && !world.mesh_tasks.contains_key(&chunk_pos)
                && !world.pending_mesh_results.contains_key(&chunk_pos)
        })
        .collect::<Vec<_>>();

    // Sort chunks by priority and distance from the player
    chunks_to_mesh
        .sort_by_key(|&(chunk_pos, status)| (Reverse(status), world.chunk_sort_key(chunk_pos)));

    // Check if any chunks are urgent
    let has_urgent = chunks_to_mesh
        .iter()
        .any(|&(_chunk_pos, status)| status == MeshStatus::Urgent);

    // If any chunks are urgent, we need to regenerate their meshes first, so let's remove all other tasks
    if has_urgent {
        let chunk_statuses: HashMap<IVec3, MeshStatus> = world
            .chunks
            .iter()
            .map(|(chunk_pos, chunk)| (*chunk_pos, chunk.mesh_status))
            .collect();

        world
            .mesh_tasks
            .retain(|chunk_pos, _| chunk_statuses[chunk_pos] == MeshStatus::Urgent);
    }

    for (chunk_pos, status) in chunks_to_mesh {
        if world.mesh_tasks.len() >= 16 || (has_urgent && status != MeshStatus::Urgent) {
            break;
        }

        // Don't mesh until light is computed for this chunk and all neighbors
        if world.chunks[&chunk_pos].light_status != LightStatus::Complete {
            continue;
        }

        let mut should_mesh = true;
        for neighbor_pos in get_neighbors(chunk_pos) {
            if let Some(neighbor) = world.chunks.get(&neighbor_pos) {
                // Neighbor exists but light not ready — wait
                if neighbor.light_status != LightStatus::Complete {
                    should_mesh = false;
                    break;
                }
            } else if world.is_visible_chunk(neighbor_pos) {
                // Visible neighbor not loaded yet — wait
                should_mesh = false;
                break;
            }
        }

        if !should_mesh {
            continue;
        }

        let relevant_chunks = RelevantChunks::from_world(world, chunk_pos);
        let registry = registry.clone();

        // Build RelevantLights from pre-computed light data
        let center_light = world.get_light_data(chunk_pos).unwrap();
        let mut relevant_lights = RelevantLights::new(chunk_pos, center_light);
        for neighbor_pos in get_neighbors(chunk_pos) {
            if let Some(light) = world.get_light_data(neighbor_pos) {
                relevant_lights.add_neighbor(neighbor_pos, light);
            }
        }

        let task = task_pool.spawn(async move {
            generate_mesh(chunk_pos, &relevant_chunks, &relevant_lights, &registry)
        });

        world.mesh_tasks.insert(chunk_pos, task);
    }

    let mut results = HashMap::new();

    // Collect generated meshes, and remove the tasks from the map
    world
        .mesh_tasks
        .retain(|chunk_pos, task| match check_ready(task) {
            Some(mesh) => {
                results.insert(*chunk_pos, mesh);

                false
            }
            None => true,
        });

    // Buffer urgent results; apply non-urgent immediately
    for (chunk_pos, mesh) in results {
        let is_urgent = world
            .chunks
            .get(&chunk_pos)
            .is_some_and(|c| c.mesh_status == MeshStatus::Urgent);

        if is_urgent {
            world.pending_mesh_results.insert(chunk_pos, mesh);
        } else {
            apply_mesh(commands, world, meshes, chunk_pos, mesh);
        }
    }

    // Check if all urgent chunks now have results buffered
    let all_urgent_ready = !world.chunks.iter().any(|(pos, chunk)| {
        chunk.mesh_status == MeshStatus::Urgent && !world.pending_mesh_results.contains_key(pos)
    });

    // Flush the buffer once every urgent chunk is accounted for
    if all_urgent_ready && !world.pending_mesh_results.is_empty() {
        let buffered = mem::take(&mut world.pending_mesh_results);
        for (chunk_pos, mesh) in buffered {
            apply_mesh(commands, world, meshes, chunk_pos, mesh);
        }
    }
}

fn apply_mesh(
    commands: &mut Commands,
    world: &mut World,
    meshes: &mut Assets<Mesh>,
    chunk_pos: IVec3,
    mesh: Option<Mesh>,
) {
    if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
        if let Some(mesh) = mesh {
            commands
                .entity(chunk.entity)
                .insert((Visibility::Visible, Mesh3d(meshes.add(mesh))));
        } else {
            commands.entity(chunk.entity).insert(Visibility::Hidden);
        }

        chunk.mesh_status = MeshStatus::Complete;
    }
}

fn get_face_neighbors(chunk_pos: IVec3) -> [IVec3; 6] {
    [
        chunk_pos + IVec3::X,
        chunk_pos - IVec3::X,
        chunk_pos + IVec3::Y,
        chunk_pos - IVec3::Y,
        chunk_pos + IVec3::Z,
        chunk_pos - IVec3::Z,
    ]
}

fn border_block_light_changed(old: &LightData, new: &LightData) -> bool {
    let cs = CHUNK_SIZE;
    for a in 0..cs {
        for b in 0..cs {
            let positions = [
                USizeVec3::new(0, a, b),
                USizeVec3::new(cs - 1, a, b),
                USizeVec3::new(a, 0, b),
                USizeVec3::new(a, cs - 1, b),
                USizeVec3::new(a, b, 0),
                USizeVec3::new(a, b, cs - 1),
            ];
            for pos in positions {
                if old.get_block_light(pos) != new.get_block_light(pos) {
                    return true;
                }
            }
        }
    }
    false
}

fn get_neighbors(chunk_pos: IVec3) -> Vec<IVec3> {
    let mut neighbors = Vec::new();

    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }

                neighbors.push(chunk_pos + IVec3::new(x, y, z));
            }
        }
    }

    neighbors
}

fn debug_ui(mut contexts: EguiContexts, mut world: ResMut<World>) -> Result {
    egui::Window::new("Settings")
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
        .show(contexts.ctx_mut()?, |ui| {
            ui.add(Slider::new(&mut world.generation_radius, 1..=32).text("Generation Radius"));
        });
    Ok(())
}

struct GenerationResult {
    chunk_data: ChunkData,
    needs_saving: bool,
}

fn generate_chunk(
    region_manager: &RegionManager,
    world_generator: &WorldGenerator,
    chunk_pos: IVec3,
    registry: &Registry,
) -> GenerationResult {
    if let Some(chunk_data) = region_manager.load_chunk(chunk_pos) {
        return GenerationResult {
            chunk_data,
            needs_saving: false,
        };
    }

    let chunk_data = world_generator.generate_chunk(chunk_pos, registry);

    GenerationResult {
        chunk_data,
        needs_saving: true,
    }
}

fn save_chunks(mut world: ResMut<World>) {
    if world.chunks_to_save.is_empty() || world.last_save_time.elapsed() < Duration::from_secs(3) {
        return;
    }

    if let Some(task) = world.save_task.as_mut()
        && check_ready(task).is_none()
    {
        return;
    }

    let task_pool = AsyncComputeTaskPool::get();
    let region_manager = world.region_manager.clone();
    let chunks = mem::take(&mut world.chunks_to_save);

    world.save_task = Some(task_pool.spawn(async move {
        region_manager.save_chunks(&chunks);
    }));

    world.last_save_time = Instant::now();
}
