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
    // Spawn a pickupable key
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.8, 0.7, 0.3), // Gold color
            Vec2::new(12.0, 12.0)
        ),
        Transform::from_xyz(-100.0, 0.0, 1.0),
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
        Transform::from_xyz(100.0, 50.0, 1.0),
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
        Transform::from_xyz(0.0, -120.0, 1.0),
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
        Transform::from_xyz(60.0, 0.0, 1.0),
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