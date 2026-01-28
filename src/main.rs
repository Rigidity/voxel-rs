use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, prelude::*, window::PresentMode};
use voxel::{PlayerPlugin, WorldPlugin};

fn main() -> Result<()> {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::Immediate,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        )
        .add_plugins((PlayerPlugin, WorldPlugin))
        .add_plugins(FpsOverlayPlugin::default())
        .insert_resource(ClearColor(Color::linear_rgb(0.1, 0.4, 0.7)))
        .run();

    Ok(())
}
