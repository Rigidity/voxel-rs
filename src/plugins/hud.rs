use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud);
    }
}

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let crosshair: Handle<Image> = asset_server.load("crosshair.png");

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        })
        .with_child(ImageNode {
            image: crosshair,
            ..Default::default()
        });
}
