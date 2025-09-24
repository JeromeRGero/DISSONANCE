use bevy::prelude::*;
use crate::objects::Solid;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (
                player_movement,
                update_player_facing,
            ));
    }
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub interact_range: f32,
    pub facing: Direction,
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn spawn_player(mut commands: Commands) {
    // Main player entity
    commands.spawn((
        Sprite::from_color(
            Color::srgb(1.0, 1.0, 0.0), // Yellow like Frisk
            Vec2::new(16.0, 20.0)
        ),
        Transform::from_xyz(0.0, 0.0, 10.0),
        Player { 
            speed: 120.0,
            interact_range: 30.0,
            facing: Direction::Down,
        },
        Name::new("Player"),
    ))
    .with_children(|parent| {
        // Interaction indicator (shows when near interactable)
        parent.spawn((
            Sprite::from_color(
                Color::srgb(1.0, 0.0, 0.0), // Red for visibility
                Vec2::new(16.0, 16.0) // Bigger size
            ),
            Transform::from_xyz(0.0, 20.0, 1.0), // Higher above player
            Visibility::Hidden,
            InteractionIndicator,
        ));
    });
}

#[derive(Component)]
pub struct InteractionIndicator;

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut Transform), Without<Solid>>,
    solid_query: Query<(&Transform, &Sprite), (With<Solid>, Without<Player>)>,
    ui_state: Res<crate::ui::UiState>,
) {
    // Don't move if menu is open
    if ui_state.menu_open || ui_state.dialog_open {
        return;
    }

    for (player, mut transform) in query.iter_mut() {
        let mut movement = Vec2::ZERO;

        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            movement.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            movement.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            movement.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            movement.x += 1.0;
        }

        if movement.length() > 0.0 {
            movement = movement.normalize();
            // Proposed movement
            let delta = movement * player.speed * time.delta_secs();

            // Player AABB (half extents) — approximate sprite size
            let half = Vec2::new(8.0, 10.0);

            // Move X then Y, resolving collisions against solids (AABB)
            // X axis
            transform.translation.x += delta.x;
            // Query solids in the world and resolve overlaps
            for (solid_tf, sprite) in solid_query.iter() {
                let solid_size = sprite.custom_size.unwrap_or(Vec2::splat(16.0));
                let s_half = solid_size / 2.0;
                let s_min_x = solid_tf.translation.x - s_half.x;
                let s_max_x = solid_tf.translation.x + s_half.x;
                let s_min_y = solid_tf.translation.y - s_half.y;
                let s_max_y = solid_tf.translation.y + s_half.y;

                let player_min_x = transform.translation.x - half.x;
                let player_max_x = transform.translation.x + half.x;
                let player_min_y = transform.translation.y - half.y;
                let player_max_y = transform.translation.y + half.y;

                let overlap_x = player_max_x > s_min_x && player_min_x < s_max_x;
                let overlap_y = player_max_y > s_min_y && player_min_y < s_max_y;
                if overlap_x && overlap_y {
                    // Push out along X based on direction
                    if delta.x > 0.0 {
                        transform.translation.x = s_min_x - half.x;
                    } else if delta.x < 0.0 {
                        transform.translation.x = s_max_x + half.x;
                    }
                }
            }

            // Y axis
            transform.translation.y += delta.y;
            for (solid_tf, sprite) in solid_query.iter() {
                let solid_size = sprite.custom_size.unwrap_or(Vec2::splat(16.0));
                let s_half = solid_size / 2.0;
                let s_min_x = solid_tf.translation.x - s_half.x;
                let s_max_x = solid_tf.translation.x + s_half.x;
                let s_min_y = solid_tf.translation.y - s_half.y;
                let s_max_y = solid_tf.translation.y + s_half.y;

                let player_min_x = transform.translation.x - half.x;
                let player_max_x = transform.translation.x + half.x;
                let player_min_y = transform.translation.y - half.y;
                let player_max_y = transform.translation.y + half.y;

                let overlap_x = player_max_x > s_min_x && player_min_x < s_max_x;
                let overlap_y = player_max_y > s_min_y && player_min_y < s_max_y;
                if overlap_x && overlap_y {
                    if delta.y > 0.0 {
                        transform.translation.y = s_min_y - half.y;
                    } else if delta.y < 0.0 {
                        transform.translation.y = s_max_y + half.y;
                    }
                }
            }
        }
    }
}

fn update_player_facing(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Player>,
    ui_state: Res<crate::ui::UiState>,
) {
    if ui_state.menu_open || ui_state.dialog_open {
        return;
    }

    for mut player in query.iter_mut() {
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            player.facing = Direction::Up;
        } else if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            player.facing = Direction::Down;
        } else if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            player.facing = Direction::Left;
        } else if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            player.facing = Direction::Right;
        }
    }
}

// Sprite::size() provides the logical size set at spawn for our AABB.