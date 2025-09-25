// src/objects.rs
use bevy::prelude::*;
use crate::interaction::{Interactable, InteractionAction};

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_example_objects);
    }
}

#[allow(dead_code)]
#[derive(Component)]
pub struct Item {
    pub name: String,
    pub can_pickup: bool,
}

#[allow(dead_code)]
#[derive(Component)]
pub struct Light {
    pub is_on: bool,
}

#[derive(Component)]
pub struct Door {
    pub is_open: bool,
    pub required_key_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Component)]
pub struct Generator {
    pub is_running: bool,
    pub fuel_level: f32,
    pub max_fuel: f32,
}

#[allow(dead_code)]
#[derive(Component)]
pub struct NPC {
    pub name: String,
    pub dialogue: Vec<String>,
}

// Marks an entity as blocking for simple 2D collision
#[derive(Component)]
pub struct Solid;

fn spawn_example_objects(mut commands: Commands) {
    // Hex-like starting room built from axis-aligned segments
    let wall_thickness = 12.0;
    let room_half_w = 160.0;
    let room_half_h = 120.0;

    // Top and bottom walls (slight overlap with verticals to avoid pixel gaps)
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.2, 0.25), Vec2::new(room_half_w * 2.0 + wall_thickness, wall_thickness)),
        Transform::from_xyz(0.0, room_half_h, 0.5),
        Solid,
        Name::new("Wall Top"),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.2, 0.25), Vec2::new(room_half_w * 2.0 + wall_thickness, wall_thickness)),
        Transform::from_xyz(0.0, -room_half_h, 0.5),
        Solid,
        Name::new("Wall Bottom"),
    ));

    // Left wall (full height to avoid gaps)
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.2, 0.25), Vec2::new(wall_thickness, room_half_h * 2.0)),
        Transform::from_xyz(-room_half_w, 0.0, 0.5),
        Solid,
        Name::new("Wall Left"),
    ));

    // Right walls with door gap (no gaps at corners)
    let door_gap_h = 36.0;
    let right_x = room_half_w;
    let segment_h = room_half_h - door_gap_h * 0.5;
    // Upper segment goes from gap to top
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.2, 0.25), Vec2::new(wall_thickness, segment_h)),
        Transform::from_xyz(right_x, door_gap_h * 0.5 + segment_h * 0.5, 0.5),
        Solid,
        Name::new("Wall Right Upper"),
    ));
    // Lower segment goes from bottom to gap
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.2, 0.25), Vec2::new(wall_thickness, segment_h)),
        Transform::from_xyz(right_x, -door_gap_h * 0.5 - segment_h * 0.5, 0.5),
        Solid,
        Name::new("Wall Right Lower"),
    ));

    // Door in the gap
    commands.spawn((
        Sprite::from_color(Color::srgb(0.5, 0.35, 0.15), Vec2::new(wall_thickness, door_gap_h)),
        Transform::from_xyz(right_x, 0.0, 0.6),
        Interactable { name: "Metal Door".to_string(), actions: vec![InteractionAction::Examine, InteractionAction::Open], interaction_radius: Some(40.0) },
        Door { is_open: false, required_key_id: Some("Rusty Key".to_string()) },
        Visibility::Visible,
        Solid,
        Name::new("Metal Door"),
    ));

    // Hallway to the right of the door (overlap door column by 1px to avoid seam)
    let hall_len = 268.0;
    let hall_half_w = hall_len / 2.0;
    let hall_half_h = 30.0;
    let hall_center_x = right_x + wall_thickness + hall_half_w - 6.0;
    commands.spawn((
        Sprite::from_color(Color::srgb(0.18, 0.18, 0.22), Vec2::new(hall_len, wall_thickness)),
        Transform::from_xyz(hall_center_x, hall_half_h, 0.5),
        Solid,
        Name::new("Hall Top"),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.18, 0.18, 0.22), Vec2::new(hall_len, wall_thickness)),
        Transform::from_xyz(hall_center_x, -hall_half_h, 0.5),
        Solid,
        Name::new("Hall Bottom"),
    ));

    // Spawn a pickupable key
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.8, 0.7, 0.3), // Gold color
            Vec2::new(12.0, 12.0)
        ),
        Transform::from_xyz(-80.0, 10.0, 1.0),
        Interactable {
            name: "Rusty Key".to_string(),
            actions: vec![
                InteractionAction::Examine,
                InteractionAction::Take,
            ],
            interaction_radius: Some(35.0), // Small object, normal radius
        },
        Item {
            name: "Rusty Key".to_string(),
            can_pickup: true,
        },
        Solid,
        Name::new("Rusty Key"),
    ));

    // Spawn a light/lamp
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.3, 0.3, 0.3), // Dark gray (off)
            Vec2::new(20.0, 28.0)
        ),
        Transform::from_xyz(60.0, 40.0, 1.0),
        Interactable {
            name: "Old Lamp".to_string(),
            actions: vec![
                InteractionAction::Examine,
                InteractionAction::TurnOn,
            ],
            interaction_radius: Some(40.0), // Medium object
        },
        Light { is_on: false },
        Solid,
        Name::new("Old Lamp"),
    ));

    // Spawn a generator - LARGER OBJECT
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.4, 0.4, 0.5), // Blue-gray
            Vec2::new(48.0, 48.0)  // Large size
        ),
        // Place so its left edge slightly overlaps the hallway end to avoid gaps
        Transform::from_xyz(hall_center_x + hall_half_w + 23.0, 0.0, 1.0),
        Interactable {
            name: "Generator".to_string(),
            actions: vec![
                InteractionAction::Examine,
                InteractionAction::Use,
                InteractionAction::Refuel,
            ],
            interaction_radius: Some(60.0), // Large object needs bigger radius
        },
        Generator {
            is_running: false,
            fuel_level: 2.5,
            max_fuel: 10.0,
        },
        Solid,
        Name::new("Generator"),
    ));

    // Spawn an NPC
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.6, 0.3, 0.8), // Purple
            Vec2::new(16.0, 20.0)
        ),
        Transform::from_xyz(20.0, -20.0, 1.0),
        Interactable {
            name: "Strange Figure".to_string(),
            actions: vec![
                InteractionAction::Talk,
                InteractionAction::Examine,
            ],
            interaction_radius: Some(40.0), // Human-sized
        },
        Solid,
        NPC {
            name: "Strange Figure".to_string(),
            dialogue: vec![
                "* ...".to_string(),
                "* The figure stares at you silently.".to_string(),
            ],
        },
        Name::new("Strange Figure"),
    ));

    // Spawn a chest/container
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.5, 0.3, 0.1), // Brown
            Vec2::new(24.0, 20.0)
        ),
        Transform::from_xyz(-50.0, -50.0, 1.0),
        Interactable {
            name: "Wooden Chest".to_string(),
            actions: vec![
                InteractionAction::Open,
                InteractionAction::Examine,
            ],
            interaction_radius: Some(40.0), // Medium object
        },
        Solid,
        Name::new("Wooden Chest"),
    ));
}