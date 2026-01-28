use bevy::prelude::*;
use voxel::{PlayerPlugin, WorldPlugin};

fn main() -> Result<()> {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((PlayerPlugin, WorldPlugin))
        .run();

    Ok(())
}
