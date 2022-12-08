use bevy::prelude::*;
use bevy::app::App;
use bevy::DefaultPlugins;
use inochi_bevy::Inochi2DPlugin;
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(Inochi2DPlugin);
    app.add_startup_system(load_icon);
    app.run();
}

fn load_icon(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("bevy.png"),
        ..default()
    });
}
