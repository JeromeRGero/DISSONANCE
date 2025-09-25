// src/interaction.rs
use bevy::prelude::*;
use crate::player::{Player, InteractionIndicator};
use crate::ui::{ContextMenuEvent, UiState, LogEvent};
use crate::GameSet;
use crate::objects::{Light, Door, Solid};
use crate::inventory::{Inventory, InventoryItem};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractionEvent>()
            .add_systems(Update, (
                check_nearby_interactables.in_set(GameSet::Detect),
                handle_interaction_input.in_set(GameSet::Input),
                process_interactions.in_set(GameSet::Process),
            ));
    }
}

#[derive(Event)]
pub struct InteractionEvent {
    pub entity: Entity,
    pub action: InteractionAction,
    // Optional context carried with the selection/execution
    pub with_item_id: Option<String>,
    pub detailed: bool,
}

#[derive(Clone, Debug)]
pub enum InteractionAction {
    Examine,
    Take,
    Use,
    TurnOn,
    TurnOff,
    Refuel,
    Talk,
    Open,
    Close,
    Custom(String),
}

impl InteractionAction {
    pub fn to_string(&self) -> String {
        match self {
            Self::Examine => "* Check".to_string(),
            Self::Take => "* Take".to_string(),
            Self::Use => "* Use".to_string(),
            Self::TurnOn => "* Turn On".to_string(),
            Self::TurnOff => "* Turn Off".to_string(),
            Self::Refuel => "* Refuel".to_string(),
            Self::Talk => "* Talk".to_string(),
            Self::Open => "* Open".to_string(),
            Self::Close => "* Close".to_string(),
            Self::Custom(s) => format!("* {}", s),
        }
    }
}

#[derive(Component)]
pub struct Interactable {
    pub name: String,
    pub actions: Vec<InteractionAction>,
    pub interaction_radius: Option<f32>, // Optional custom radius
}

impl Default for Interactable {
    fn default() -> Self {
        Self {
            name: "Object".to_string(),
            actions: vec![InteractionAction::Examine],
            interaction_radius: None, // Use default radius
        }
    }
}

#[derive(Component)]
pub struct NearbyInteractable;

fn check_nearby_interactables(
    player_query: Query<(&Player, &Transform, &Children)>,
    interactables: Query<(Entity, &Interactable, &Transform), Without<NearbyInteractable>>,
    mut indicator_query: Query<&mut Visibility, With<InteractionIndicator>>,
    mut commands: Commands,
    existing_nearby: Query<Entity, With<NearbyInteractable>>,
) {
    // Clear all existing nearby markers
    for entity in existing_nearby.iter() {
        commands.entity(entity).remove::<NearbyInteractable>();
    }

    for (_player, player_transform, children) in player_query.iter() {
        let mut closest_interactable: Option<Entity> = None;
        let mut closest_distance = f32::MAX;

        for (entity, interactable, transform) in interactables.iter() {
            let distance = player_transform.translation.truncate()
                .distance(transform.translation.truncate());
            
            // Use the object's custom interaction radius, or default to 40
            let radius = interactable.interaction_radius.unwrap_or(40.0);
            if distance <= radius && distance < closest_distance {
                closest_distance = distance;
                closest_interactable = Some(entity);
            }
        }

        // Update indicator visibility
        for &child in children {
            if let Ok(mut visibility) = indicator_query.get_mut(child) {
                *visibility = if closest_interactable.is_some() {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }

        // Mark the closest as nearby
        if let Some(entity) = closest_interactable {
            commands.entity(entity).insert(NearbyInteractable);
        }
    }
}

fn handle_interaction_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    interactables_query: Query<(Entity, &Interactable, &Transform)>,
    mut menu_events: EventWriter<ContextMenuEvent>,
    mut interaction_events: EventWriter<InteractionEvent>,
    ui_state: Res<UiState>,
    lights: Query<&Light>,
    doors: Query<&Door>,
) {
    // Don't process interaction if menu is already open
    if ui_state.menu_open || ui_state.dialog_open {
        return;
    }

    // Check for interaction key
    let interact_pressed = keyboard.just_pressed(KeyCode::KeyZ) 
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter);

    if interact_pressed {
        // Find nearest interactable in range from the player
    if let Ok(player_tf) = player_query.single() {
            let mut best: Option<(Entity, &Interactable)> = None;
            let mut best_dist = f32::MAX;
            for (entity, interactable, tf) in interactables_query.iter() {
                let d = player_tf.translation.truncate().distance(tf.translation.truncate());
                let radius = interactable.interaction_radius.unwrap_or(40.0);
                if d <= radius && d < best_dist {
                    best_dist = d;
                    best = Some((entity, interactable));
                }
            }

            if let Some((entity, interactable)) = best {
                // Build actions dynamically based on current component state (Light, Door)
                let mut actions = interactable.actions.clone();
                if let Ok(light) = lights.get(entity) {
                    // Ensure only the correct toggle option is present
                    actions.retain(|a| !matches!(a, InteractionAction::TurnOn | InteractionAction::TurnOff));
                    if light.is_on {
                        actions.push(InteractionAction::TurnOff);
                    } else {
                        actions.push(InteractionAction::TurnOn);
                    }
                }
                if let Ok(door) = doors.get(entity) {
                    actions.retain(|a| !matches!(a, InteractionAction::Open | InteractionAction::Close));
                    if door.is_open {
                        actions.push(InteractionAction::Close);
                    } else {
                        actions.push(InteractionAction::Open);
                    }
                }

                info!("Interacting with: {} ({} actions)", interactable.name, actions.len());
                if actions.len() == 1 {
                    interaction_events.write(InteractionEvent { entity, action: actions[0].clone(), with_item_id: None, detailed: false });
                } else {
                    menu_events.write(ContextMenuEvent {
                        entity,
                        actions,
                        object_name: interactable.name.clone(),
                    });
                }
            }
        }
    }
}

fn process_interactions(
    mut events: EventReader<InteractionEvent>,
    mut commands: Commands,
    interactables: Query<&Interactable>,
    mut inventory: ResMut<Inventory>,
    mut log_writer: EventWriter<LogEvent>,
    mut lights: Query<&mut Light>,
    mut doors: Query<&mut Door>,
    mut sprites: Query<&mut Sprite>,
    mut visibilities: Query<&mut Visibility>,
) {
    for event in events.read() {
        info!("Processing interaction: {:?}", event.action);
        
        if let Ok(interactable) = interactables.get(event.entity) {
            match &event.action {
                InteractionAction::Examine => {
                    let l1 = format!("* You examine the {}.", interactable.name);
                    let l2 = format!("* It appears to be a regular {}.", interactable.name);
                    info!("{}", l1);
                    info!("{}", l2);
                    log_writer.write(LogEvent(l1));
                    log_writer.write(LogEvent(l2));
                }
                InteractionAction::Take => {
                    let added = inventory.add_item(InventoryItem {
                        id: interactable.name.clone(),
                        name: interactable.name.clone(),
                        description: format!("A {} that you picked up.", interactable.name),
                        icon_color: Color::WHITE,
                    });
                    
                    if added {
                        let l = format!("* You obtained the {}!", interactable.name);
                        info!("{}", l);
                        log_writer.write(LogEvent(l));
                        // Despawn the entity completely (recursive by default in 0.16)
                        commands.entity(event.entity).despawn();
                    } else {
                        let l = "* Your inventory is full!".to_string();
                        info!("{}", l);
                        log_writer.write(LogEvent(l));
                    }
                }
                InteractionAction::Use => {
                    let l1 = format!("* You use the {}.", interactable.name);
                    let l2 = "* Nothing happens.".to_string();
                    info!("{}", l1);
                    info!("{}", l2);
                    log_writer.write(LogEvent(l1));
                    log_writer.write(LogEvent(l2));
                }
                InteractionAction::Talk => {
                    let l1 = format!("* You speak to the {}.", interactable.name);
                    let l2 = "* ...".to_string();
                    let l3 = "* It doesn't respond.".to_string();
                    info!("{}", l1);
                    info!("{}", l2);
                    info!("{}", l3);
                    log_writer.write(LogEvent(l1));
                    log_writer.write(LogEvent(l2));
                    log_writer.write(LogEvent(l3));
                }
                InteractionAction::Open => {
                    // Doors: require key to open if specified, remove Solid when opened
                    if let Ok(mut door) = doors.get_mut(event.entity) {
                        if door.is_open {
                            let l = format!("* The {} is already open.", interactable.name);
                            info!("{}", l);
                            log_writer.write(LogEvent(l));
                        } else {
                            let can_open = match &door.required_key_id {
                                Some(key_id) => inventory.has_item_id(key_id),
                                None => true,
                            };
                            if can_open {
                                if let Some(key_id) = &door.required_key_id {
                                    let _ = inventory.take_item_by_id(key_id);
                                }
                                door.is_open = true;
                                commands.entity(event.entity).remove::<Solid>();
                                if let Ok(mut sprite) = sprites.get_mut(event.entity) {
                                    sprite.color = Color::srgb(0.6, 0.45, 0.2);
                                }
                                if let Ok(mut vis) = visibilities.get_mut(event.entity) {
                                    *vis = Visibility::Hidden;
                                }
                                let l1 = format!("* You open the {}.", interactable.name);
                                let l2 = match &door.required_key_id {
                                    Some(_) => "* The lock clicks open.".to_string(),
                                    None => "* It swings open.".to_string(),
                                };
                                info!("{}", l1);
                                info!("{}", l2);
                                log_writer.write(LogEvent(l1));
                                log_writer.write(LogEvent(l2));
                            } else {
                                let l1 = format!("* The {} is locked.", interactable.name);
                                let l2 = "* You need a matching key.".to_string();
                                info!("{}", l1);
                                info!("{}", l2);
                                log_writer.write(LogEvent(l1));
                                log_writer.write(LogEvent(l2));
                            }
                        }
                    } else {
                        let l1 = format!("* You open the {}.", interactable.name);
                        let l2 = "* It's empty inside.".to_string();
                        info!("{}", l1);
                        info!("{}", l2);
                        log_writer.write(LogEvent(l1));
                        log_writer.write(LogEvent(l2));
                    }
                }
                InteractionAction::TurnOn => {
                    let mut already_on = false;
                    if let Ok(mut light) = lights.get_mut(event.entity) {
                        already_on = light.is_on;
                        light.is_on = true;
                    }
                    if let Ok(mut sprite) = sprites.get_mut(event.entity) {
                        sprite.color = Color::srgb(1.0, 0.9, 0.3);
                    }
                    let l1 = format!("* You flip the switch on the {}.", interactable.name);
                    let l2 = if already_on { "* It's already on.".to_string() } else { "* It hums to life.".to_string() };
                    info!("{}", l1);
                    info!("{}", l2);
                    log_writer.write(LogEvent(l1));
                    log_writer.write(LogEvent(l2));
                }
                InteractionAction::Close => {
                    if let Ok(mut door) = doors.get_mut(event.entity) {
                        if !door.is_open {
                            let l = format!("* The {} is already closed.", interactable.name);
                            info!("{}", l);
                            log_writer.write(LogEvent(l));
                        } else {
                            door.is_open = false;
                            commands.entity(event.entity).insert(Solid);
                            if let Ok(mut sprite) = sprites.get_mut(event.entity) {
                                sprite.color = Color::srgb(0.5, 0.35, 0.15);
                            }
                            if let Ok(mut vis) = visibilities.get_mut(event.entity) {
                                *vis = Visibility::Visible;
                            }
                            let l1 = format!("* You close the {}.", interactable.name);
                            let l2 = "* It latches shut.".to_string();
                            info!("{}", l1);
                            info!("{}", l2);
                            log_writer.write(LogEvent(l1));
                            log_writer.write(LogEvent(l2));
                        }
                    } else {
                        let l = format!("* You close the {}.", interactable.name);
                        info!("{}", l);
                        log_writer.write(LogEvent(l));
                    }
                }
                InteractionAction::TurnOff => {
                    let mut already_off = false;
                    if let Ok(mut light) = lights.get_mut(event.entity) {
                        already_off = !light.is_on;
                        light.is_on = false;
                    }
                    if let Ok(mut sprite) = sprites.get_mut(event.entity) {
                        sprite.color = Color::srgb(0.3, 0.3, 0.3);
                    }
                    let l1 = format!("* You flip the switch on the {}.", interactable.name);
                    let l2 = if already_off { "* It's already off.".to_string() } else { "* It goes dark.".to_string() };
                    info!("{}", l1);
                    info!("{}", l2);
                    log_writer.write(LogEvent(l1));
                    log_writer.write(LogEvent(l2));
                }
                InteractionAction::Refuel => {
                    let l1 = format!("* You search for fuel to add to the {}.", interactable.name);
                    let l2 = "* You don't have any fuel.".to_string();
                    info!("{}", l1);
                    info!("{}", l2);
                    log_writer.write(LogEvent(l1));
                    log_writer.write(LogEvent(l2));
                }
                _ => {
                    let action_str = event
                        .action
                        .to_string()
                        .replace("* ", "")
                        .to_lowercase();
                    let l = format!("* You {} the {}.", action_str, interactable.name);
                    info!("{}", l);
                    log_writer.write(LogEvent(l));
                }
            }
        }
    }
}