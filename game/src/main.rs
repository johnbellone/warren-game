//! Warren Game — standalone Bevy voxel editor binary.

use bevy::prelude::*;
use warren_game::WarrenGamePlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WarrenGamePlugin))
        .run();
}
