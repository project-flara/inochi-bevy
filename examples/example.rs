use bevy::app::App;
use bevy::DefaultPlugins;
use inochi_bevy::Inochi2DPlugin;
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(Inochi2DPlugin);
    app.run();
}
