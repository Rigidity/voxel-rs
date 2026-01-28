use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, prelude::*, window::WindowResolution};
use voxel::{PlayerPlugin, WorldPlugin};

fn main() -> Result<()> {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Defaria".to_string(),
                        resolution: WindowResolution::new(1200, 675),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
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
