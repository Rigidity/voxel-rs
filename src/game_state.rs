use std::sync::Arc;

use glam::{IVec3, Vec3};

use crate::{
    Camera, DIRT, Input, Player, Projection, ROCK, Registry, TEST, TextureArrayBuilder, World,
    WorldGenerator,
};

pub struct GameState {
    pub player: Player,
    pub camera: Camera,
    pub projection: Projection,
    pub world: World,
    pub registry: Arc<Registry>,
    pub teleported_to_safety: bool,
}

impl GameState {
    pub fn new(texture_builder: Option<&mut TextureArrayBuilder>) -> Self {
        let player = Player::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.6, 1.8, 0.6), 1.4);
        let camera = Camera::new(player.camera_position(), player.yaw, player.pitch);
        let projection = Projection::new(0, 0, 75.0, 0.1, 1000.0);
        let world = World::new(WorldGenerator::new());

        let mut registry = Registry::new();
        registry.register_block_type(DIRT);
        registry.register_block_type(ROCK);
        registry.register_block_type(TEST);

        if let Some(builder) = texture_builder {
            for block_id in registry.block_ids().collect::<Vec<_>>() {
                let block_type = registry.block_type(block_id);
                block_type.register_textures(builder, &mut registry);
            }
        }

        Self {
            player,
            camera,
            projection,
            world,
            registry: Arc::new(registry),
            teleported_to_safety: false,
        }
    }

    pub fn tick(&mut self, input: &mut Input, delta: f32) {
        let player_pos = IVec3::new(
            self.player.position.x as i32,
            self.player.position.y as i32,
            self.player.position.z as i32,
        );
        let player_chunk_pos = World::chunk_pos(player_pos);
        self.world.tick(player_chunk_pos, &self.registry);
        self.player
            .update(input, delta, &mut self.world, &self.registry);
        self.camera.position = self.player.camera_position();
        self.camera.yaw = self.player.yaw;
        self.camera.pitch = self.player.pitch;

        if !self.teleported_to_safety {
            let mut y = 0;

            loop {
                let world_pos = IVec3::new(0, y, 0);
                let chunk_pos = World::chunk_pos(world_pos);

                if !self.world.is_visible_chunk(chunk_pos) {
                    self.teleported_to_safety = true;
                    break;
                }

                if !self.world.chunks.contains_key(&chunk_pos) {
                    break;
                }

                if self.world.get_block(world_pos).is_none()
                    && self.world.get_block(world_pos + IVec3::Y).is_none()
                {
                    self.player.position = Vec3::new(0.0, y as f32 + 1.0, 0.0);
                    self.teleported_to_safety = true;
                    break;
                }

                y += 1;
            }
        }
    }
}
