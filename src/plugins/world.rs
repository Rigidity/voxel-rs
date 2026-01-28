use std::{cmp::Reverse, collections::HashMap};

use bevy::{
    math::USizeVec3,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};
use indexmap::IndexMap;
use strum::IntoEnumIterator;

use crate::{
    Block, BlockKind, ChunkMaterial, Player, Registry, TextureArrayBuilder, WorldGenerator,
    generate_mesh,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(World {
            center_pos: IVec3::ZERO,
            generator: WorldGenerator::new(),
            generation_radius: 4,
            generation_tasks: IndexMap::new(),
            mesh_tasks: IndexMap::new(),
            chunks: IndexMap::new(),
            registry: Registry::new(),
        })
        .add_systems(Startup, setup_registry)
        .add_systems(Update, update_world);
    }
}

#[derive(Resource)]
pub struct World {
    center_pos: IVec3,
    generator: WorldGenerator,
    generation_radius: i32,
    generation_tasks: IndexMap<IVec3, Task<ChunkData>>,
    mesh_tasks: IndexMap<IVec3, Task<Option<Mesh>>>,
    chunks: IndexMap<IVec3, Chunk>,
    registry: Registry,
}

impl World {
    pub fn is_visible_chunk(&self, chunk_pos: IVec3) -> bool {
        let horizontal_distance = Vec3::new(
            chunk_pos.x as f32,
            self.center_pos.y as f32,
            chunk_pos.z as f32,
        )
        .distance(Vec3::new(
            self.center_pos.x as f32,
            self.center_pos.y as f32,
            self.center_pos.z as f32,
        ));

        if horizontal_distance > self.generation_radius as f32 {
            return false;
        }

        let vertical_distance = (chunk_pos.y as f32 - self.center_pos.y as f32).abs();

        if vertical_distance > self.generation_radius as f32 {
            return false;
        }

        true
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

        if chunk.mesh_status == MeshStatus::Complete {
            chunk.mesh_status = MeshStatus::Queued;
        }

        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let neighbor = chunk_pos + IVec3::new(x, y, z);

                    if neighbor == chunk_pos {
                        continue;
                    }

                    if let Some(neighbor) = self.chunks.get_mut(&neighbor) {
                        neighbor.mesh_status = MeshStatus::Urgent;
                    }
                }
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
}

pub const CHUNK_SIZE: usize = 32;

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

#[derive(Debug, Clone)]
pub struct ChunkData {
    blocks: Vec<Option<Block>>,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            blocks: vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl ChunkData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_block(&self, local_pos: USizeVec3) -> Option<Block> {
        self.blocks[self.index(local_pos)]
    }

    pub fn set_block(&mut self, local_pos: USizeVec3, block: Option<Block>) {
        let index = self.index(local_pos);
        self.blocks[index] = block;
    }

    fn index(&self, local_pos: USizeVec3) -> usize {
        assert!(local_pos.x < CHUNK_SIZE);
        assert!(local_pos.y < CHUNK_SIZE);
        assert!(local_pos.z < CHUNK_SIZE);
        local_pos.x + local_pos.y * CHUNK_SIZE + local_pos.z * CHUNK_SIZE * CHUNK_SIZE
    }
}

#[derive(Debug, Clone)]
pub struct RelevantChunks {
    chunks: HashMap<IVec3, ChunkData>,
}

impl RelevantChunks {
    pub fn from_world(world: &World, center_pos: IVec3) -> Self {
        let mut chunks = HashMap::new();

        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let chunk_pos = center_pos + IVec3::new(x, y, z);
                    if let Some(chunk) = world.chunks.get(&chunk_pos) {
                        chunks.insert(chunk_pos, chunk.data.clone());
                    }
                }
            }
        }

        Self { chunks }
    }

    pub fn get_block(&self, world_pos: IVec3) -> Option<Block> {
        let chunk_pos = World::chunk_pos(world_pos);
        let local_pos = World::local_pos(world_pos);
        let chunk = self.chunks.get(&chunk_pos)?;
        chunk.get_block(local_pos)
    }
}

fn setup_registry(mut world: ResMut<World>) {
    let mut builder = TextureArrayBuilder::new(16, 16);

    let atlas = image::load_from_memory(include_bytes!("../../textures/textures.png")).unwrap();

    for block_kind in BlockKind::iter() {
        block_kind.register_textures(&mut builder, &mut world.registry, &atlas);
    }

    let textures = builder.into_textures();

    log::info!("Generating an array of {} textures", textures.len());
}

fn update_world(
    mut commands: Commands,
    mut world: ResMut<World>,
    player: Query<&GlobalTransform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
) {
    let player = player.single().unwrap();
    let player_world_pos = IVec3::new(
        player.translation().x as i32,
        player.translation().y as i32,
        player.translation().z as i32,
    );
    world.center_pos = World::chunk_pos(player_world_pos);

    let task_pool = AsyncComputeTaskPool::get();

    let chunks_to_remove = world
        .chunks
        .keys()
        .copied()
        .filter(|chunk_pos| !world.is_visible_chunk(*chunk_pos))
        .collect::<Vec<_>>();

    for chunk_pos in chunks_to_remove {
        if let Some(chunk) = world.chunks.swap_remove(&chunk_pos) {
            commands.entity(chunk.entity).despawn();
        }
    }

    let mut chunks_to_generate = Vec::new();

    for x in -world.generation_radius..=world.generation_radius {
        for y in -world.generation_radius..=world.generation_radius {
            for z in -world.generation_radius..=world.generation_radius {
                let chunk_pos = world.center_pos + IVec3::new(x, y, z);

                if world.is_visible_chunk(chunk_pos)
                    && !world.chunks.contains_key(&chunk_pos)
                    && !world.generation_tasks.contains_key(&chunk_pos)
                {
                    chunks_to_generate.push(chunk_pos);
                }
            }
        }
    }

    chunks_to_generate.sort_by_key(|chunk_pos| world.chunk_sort_key(*chunk_pos));

    let mut exceeded = false;

    for chunk_pos in chunks_to_generate {
        if world.generation_tasks.len() >= task_pool.thread_num() {
            exceeded = true;
            break;
        }

        let generator = world.generator.clone();

        let task = task_pool.spawn(async move { generator.generate_chunk(chunk_pos) });

        world.generation_tasks.insert(chunk_pos, task);
    }

    let has_generations = !world.generation_tasks.is_empty();

    let mut results = HashMap::new();

    world
        .generation_tasks
        .retain(|chunk_pos, task| match check_ready(task) {
            Some(data) => {
                results.insert(*chunk_pos, data);

                false
            }
            None => true,
        });

    for (chunk_pos, data) in results {
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let neighbor = chunk_pos + IVec3::new(x, y, z);

                    if let Some(neighbor) = world.chunks.get_mut(&neighbor)
                        && neighbor.mesh_status == MeshStatus::Complete
                    {
                        neighbor.mesh_status = MeshStatus::Queued;
                    }
                }
            }
        }

        let entity = commands
            .spawn((
                Transform::from_xyz(
                    chunk_pos.x as f32 * CHUNK_SIZE as f32,
                    chunk_pos.y as f32 * CHUNK_SIZE as f32,
                    chunk_pos.z as f32 * CHUNK_SIZE as f32,
                ),
                Visibility::Visible,
                MeshMaterial3d(materials.add(ChunkMaterial { array_texture })),
            ))
            .id();

        world.chunks.insert(
            chunk_pos,
            Chunk {
                data,
                mesh_status: MeshStatus::Queued,
                entity,
            },
        );
    }

    if has_generations && world.generation_tasks.is_empty() && !exceeded {
        println!(
            "Finished generating chunks, currently have {} loaded chunks",
            world.chunks.len()
        );
    }

    let tasks_to_remove = world
        .mesh_tasks
        .keys()
        .copied()
        .filter(|chunk_pos| !world.chunks.contains_key(chunk_pos))
        .collect::<Vec<_>>();

    for chunk_pos in tasks_to_remove {
        world.mesh_tasks.swap_remove(&chunk_pos);
    }

    let mut chunks_to_mesh = world
        .chunks
        .keys()
        .copied()
        .map(|chunk_pos| (chunk_pos, world.chunks[&chunk_pos].mesh_status))
        .filter(|&(chunk_pos, status)| {
            status != MeshStatus::Complete && !world.mesh_tasks.contains_key(&chunk_pos)
        })
        .collect::<Vec<_>>();

    chunks_to_mesh
        .sort_by_key(|&(chunk_pos, status)| (Reverse(status), world.chunk_sort_key(chunk_pos)));

    let has_modified = chunks_to_mesh
        .iter()
        .any(|&(_chunk_pos, status)| status == MeshStatus::Urgent);

    if has_modified {
        // TODO: Cleanup
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
        if world.mesh_tasks.len() >= task_pool.thread_num()
            || (has_modified && status != MeshStatus::Urgent)
        {
            break;
        }

        let neighbors = [
            chunk_pos - IVec3::X,
            chunk_pos + IVec3::X,
            chunk_pos - IVec3::Y,
            chunk_pos + IVec3::Y,
            chunk_pos - IVec3::Z,
            chunk_pos + IVec3::Z,
        ];

        let mut should_generate = true;

        for neighbor in neighbors {
            should_generate &=
                world.chunks.contains_key(&neighbor) || !world.is_visible_chunk(neighbor);
        }

        if !should_generate {
            continue;
        }

        let relevant_chunks = RelevantChunks::from_world(&world, chunk_pos);

        let registry = world.registry.clone();

        let task =
            task_pool.spawn(async move { generate_mesh(chunk_pos, &relevant_chunks, &registry) });

        world.mesh_tasks.insert(chunk_pos, task);
    }

    let mut results = HashMap::new();

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
            if let Some(mesh) = mesh {
                commands
                    .entity(chunk.entity)
                    .insert(Mesh3d(meshes.add(mesh)));
            }

            chunk.mesh_status = MeshStatus::Complete;
        }
    }
}
