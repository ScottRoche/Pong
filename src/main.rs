use bevy::prelude::*;

fn init() {
    info!("Starting");
}

fn update() {
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, init)
        .add_systems(Update, update)
        .run();
}
