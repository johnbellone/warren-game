use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_systems(Update, hello_world)
        .run();
}

fn hello_world() {
    println!("Hello, Bevy world!");
}
