use glam::Vec3;

use crate::{Camera, Input, Player, Projection, World, WorldGenerator};

pub struct GameState {
    pub player: Player,
    pub camera: Camera,
    pub projection: Projection,
    pub world: World,
}

impl GameState {
    pub fn new() -> Self {
        let player = Player::new(Vec3::new(0.0, 0.0, 280.0), Vec3::new(0.8, 1.8, 0.8), 1.4);
        let camera = Camera::new(
            player.camera_position(),
            player.yaw_degrees,
            player.pitch_degrees,
        );
        let projection = Projection::new(0, 0, 60.0, 0.1, 1000.0);
        let world = World::new(WorldGenerator::new());

        Self {
            player,
            camera,
            projection,
            world,
        }
    }

    pub fn tick(&mut self, input: &mut Input, delta: f32) {
        self.player.update(input, delta);
        self.camera.position = self.player.camera_position();
        self.camera.yaw_degrees = self.player.yaw_degrees;
        self.camera.pitch_degrees = self.player.pitch_degrees;
    }
}
