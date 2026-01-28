use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player);
    }
}

#[derive(Component)]
pub struct Player;

fn setup_player(mut commands: Commands) {
    commands.spawn((Player, Camera3d::default()));
}
