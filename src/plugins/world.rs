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
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, Slider},
};
use indexmap::IndexMap;
use strum::IntoEnumIterator;

use crate::{
    Block, BlockKind, CHUNK_SIZE, ChunkData, ChunkMaterial, Player, RegionManager, Registry,
    RelevantChunks, TextureArrayBuilder, WorldGenerator, generate_mesh,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default())
            .add_plugins(EguiPlugin::default())
            .insert_resource(World {
                center_pos: IVec3::ZERO,
                region_manager: Arc::new(RegionManager::new(PathBuf::from("regions"))),
                generator: WorldGenerator::new(),
                generation_radius: 12,
                generation_tasks: IndexMap::new(),
                mesh_tasks: IndexMap::new(),
                chunks: IndexMap::new(),
                chunks_to_save: HashMap::new(),
                registry: Registry::new(),
                last_save_time: Instant::now(),
                save_task: None,
            })
            .add_systems(Startup, setup_registry)
            .add_systems(Update, (update_world, save_chunks).chain_ignore_deferred())
            .add_systems(EguiPrimaryContextPass, debug_ui);
    }
}

#[derive(Resource)]
pub struct BlockTextureArray {
    pub handle: Handle<Image>,
    pub material: Handle<ChunkMaterial>,
}

#[derive(Resource)]
pub struct World {
    center_pos: IVec3,
    region_manager: Arc<RegionManager>,
    generator: WorldGenerator,
    generation_radius: i32,
    generation_tasks: IndexMap<IVec3, Task<GenerationResult>>,
    mesh_tasks: IndexMap<IVec3, Task<Option<Mesh>>>,
    chunks: IndexMap<IVec3, Chunk>,
    chunks_to_save: HashMap<IVec3, ChunkData>,
    save_task: Option<Task<()>>,
    last_save_time: Instant,
    registry: Registry,
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

        chunk.data.set_block(local_pos, block);
        self.chunks_to_save.insert(chunk_pos, chunk.data.clone());

        self.force_remesh(chunk_pos);

        for neighbor_pos in get_neighbors(chunk_pos) {
            self.force_remesh(neighbor_pos);

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
    mesh_status: MeshStatus,
    entity: Entity,
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

fn setup_registry(
    mut commands: Commands,
    mut world: ResMut<World>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
) {
    let mut builder = TextureArrayBuilder::new(16, 16);

    let atlas = image::load_from_memory(include_bytes!("../../textures/textures.png")).unwrap();

    for block_kind in BlockKind::iter() {
        block_kind.register_textures(&mut builder, &mut world.registry, &atlas);
    }

    let textures = builder.into_textures();

    log::info!("Generating an array of {} textures", textures.len());

    // Convert Vec<DynamicImage> into a texture array
    let texture_size = 16u32;
    let array_layers = textures.len() as u32;

    let mut texture_array_data = Vec::new();

    for texture in textures {
        let rgba = texture.to_rgba8();
        texture_array_data.extend_from_slice(&rgba);
    }

    let texture_array = Image::new(
        Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: array_layers,
        },
        TextureDimension::D2,
        texture_array_data,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    );

    let handle = images.add(texture_array);

    let material = materials.add(ChunkMaterial {
        array_texture: handle.clone(),
    });

    commands.insert_resource(BlockTextureArray { handle, material });
}

fn update_world(
    mut commands: Commands,
    mut world: ResMut<World>,
    player: Query<&GlobalTransform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    texture_array: Res<BlockTextureArray>,
) {
    let player = player.single().unwrap();

    world.update_center_pos(player.translation());

    unload_far_chunks(&mut commands, &mut world);
    load_near_chunks(&mut commands, &mut world, texture_array.material.clone());
    regenerate_meshes(&mut commands, &mut world, &mut meshes);
}

fn unload_far_chunks(commands: &mut Commands, world: &mut World) {
    let chunks_to_unload = world.chunks_to_unload();

    for chunk_pos in chunks_to_unload {
        if let Some(chunk) = world.chunks.swap_remove(&chunk_pos) {
            commands.entity(chunk.entity).despawn();
        }

        world.mesh_tasks.swap_remove(&chunk_pos);
        world.generation_tasks.swap_remove(&chunk_pos);

        for neighbor_pos in get_neighbors(chunk_pos) {
            world.force_remesh(neighbor_pos);
        }
    }
}

fn load_near_chunks(commands: &mut Commands, world: &mut World, material: Handle<ChunkMaterial>) {
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

        let task =
            task_pool.spawn(async move { generate_chunk(&region_manager, &generator, chunk_pos) });

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
        // Force remesh of neighbors, since their mesh will depend on this chunk
        for neighbor_pos in get_neighbors(chunk_pos) {
            world.force_remesh(neighbor_pos);
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

fn regenerate_meshes(commands: &mut Commands, world: &mut World, meshes: &mut Assets<Mesh>) {
    let task_pool = AsyncComputeTaskPool::get();

    // We need to generate meshes a single time for queued chunks
    let mut chunks_to_mesh = world
        .chunks
        .keys()
        .copied()
        .map(|chunk_pos| (chunk_pos, world.chunks[&chunk_pos].mesh_status))
        .filter(|&(chunk_pos, status)| {
            status != MeshStatus::Complete && !world.mesh_tasks.contains_key(&chunk_pos)
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

        // If there are any unloaded neighbors, we shouldn't waste time generating a mesh for this chunk yet
        let mut should_mesh = true;

        for neighbor_pos in get_neighbors(chunk_pos) {
            should_mesh &=
                world.chunks.contains_key(&neighbor_pos) || !world.is_visible_chunk(neighbor_pos);
        }

        if !should_mesh {
            continue;
        }

        // Obtain a reference to neighboring chunks, since we need to generate the mesh for this chunk based on them
        let relevant_chunks = RelevantChunks::from_world(world, chunk_pos);

        let registry = world.registry.clone();

        let task =
            task_pool.spawn(async move { generate_mesh(chunk_pos, &relevant_chunks, &registry) });

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

    for (chunk_pos, mesh) in results {
        if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
            // if let Some(mesh) = mesh {
            //     commands
            //         .entity(chunk.entity)
            //         .insert(Mesh3d(meshes.add(mesh)));
            // } else {
            //     commands.entity(chunk.entity).remove::<Mesh3d>();
            // }

            if let Some(mesh) = mesh {
                commands
                    .entity(chunk.entity)
                    .insert((Visibility::Visible, Mesh3d(meshes.add(mesh))));
            } else {
                commands.entity(chunk.entity).insert(Visibility::Hidden);
            }

            // Mark the chunk as complete so we don't mesh it again
            chunk.mesh_status = MeshStatus::Complete;
        }
    }
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
    egui::Window::new("Settings").show(contexts.ctx_mut()?, |ui| {
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
) -> GenerationResult {
    if let Some(chunk_data) = region_manager.load_chunk(chunk_pos) {
        return GenerationResult {
            chunk_data,
            needs_saving: false,
        };
    }

    let chunk_data = world_generator.generate_chunk(chunk_pos);

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
