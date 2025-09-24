use bevy::prelude::*;
use bevy::window::WindowResolution;

mod player;
mod interaction;
mod inventory;
mod objects;
mod ui;

use player::PlayerPlugin;
use interaction::InteractionPlugin;
use inventory::InventoryPlugin;
use objects::ObjectsPlugin;
use ui::UiPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    Detect,   // proximity detection, markers
    Input,    // read inputs, emit UI events
    Ui,       // show/hide menus, handle UI navigation
    Process,  // apply game logic, update logs
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "DISSONANCE".to_string(),
                    resolution: WindowResolution::new(640.0, 480.0),
                    resizable: false,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()) // Pixel-perfect rendering
        )
        // Ensure systems across plugins run in a deterministic, single-frame order
        .configure_sets(Update, (
            GameSet::Detect,
            GameSet::Input,
            GameSet::Ui,
            GameSet::Process,
        ).chain())
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
        .add_plugins((
            PlayerPlugin,
            InteractionPlugin,
            InventoryPlugin,
            ObjectsPlugin,
            UiPlugin,
        ))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    // To zoom: Query for OrthographicProjection component and modify its scale field
    // Smaller scale = zoomed in, Larger scale = zoomed out  
    // Example: projection.scale = 0.5; // 2x zoom in
}