//! Warren Game — standalone Bevy voxel editor binary.

use std::path::PathBuf;
use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use warren_game::WarrenGamePlugin;

fn main() {
    // Bevy 0.20 resolves file_path relative to the binary's directory (target/debug/),
    // NOT CWD or CARGO_MANIFEST_DIR. Use an absolute path so it always resolves correctly
    // regardless of where the binary is launched from.
    let asset_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .to_string_lossy()
        .to_string();

    let asset_plugin = AssetPlugin {
        file_path: asset_dir.clone(),
        processed_file_path: asset_dir,
        ..Default::default()
    };

    App::new()
        .add_plugins(DefaultPlugins.set(asset_plugin))
        .add_plugins(WarrenGamePlugin)
        .run();
}
