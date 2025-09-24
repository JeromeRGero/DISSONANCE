// src/ui.rs
use bevy::prelude::*;
use bevy::color::palettes::basic::{WHITE, YELLOW};
use crate::interaction::{InteractionAction, InteractionEvent};
use crate::GameSet;
use crate::inventory::Inventory;

#[derive(Component)]
struct ContinueChevron;

#[derive(Component)]
struct CloseChevron;

#[derive(Component)]
struct ChevronBlink {
    timer: Timer,
}
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ContextMenuEvent>()
            .add_event::<LogEvent>()
            .insert_resource(UiState::default())
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                // Order matters here for consistent feel
                show_context_menu,
                handle_menu_navigation,
                handle_menu_selection,
                handle_menu_cancel,
            ).chain().in_set(GameSet::Ui))
            .add_systems(Update, (
                // Dialog open/update happens before input so the same-frame key press doesn't skip
                update_log_display,
                handle_dialog_input,
                blink_continue_chevron,
                update_inventory_ui,
            ).in_set(GameSet::Process));
    }
}

#[derive(Resource, Default)]
pub struct UiState {
    pub menu_open: bool,
    pub selected_index: usize,
    pub current_entity: Option<Entity>,
    pub current_actions: Vec<InteractionAction>,
    // Timestamp when the menu was opened; used to debounce input so we don't
    // immediately trigger a selection on the same frame/key press.
    pub menu_opened_at: f64,
    // Modal dialog state (Undertale-style): a queue of lines, shown one per press
    pub dialog_open: bool,
    pub dialog_queue: Vec<String>,
    pub dialog_index: usize,
    pub dialog_opened_at: f64,
}

#[derive(Event)]
pub struct ContextMenuEvent {
    pub entity: Entity,
    pub actions: Vec<InteractionAction>,
    pub object_name: String,
}

#[derive(Component)]
struct ContextMenuRoot;

#[derive(Component)]
struct ContextMenuBox;

#[derive(Component)]
struct MenuOption {
    index: usize,
}

#[derive(Component)]
struct MessageLogRoot;

#[derive(Component)]
struct MessageText;

#[derive(Event)]
pub struct LogEvent(pub String);

#[derive(Component)]
struct InventoryRoot;

#[derive(Component)]
struct InventoryList;

fn setup_ui(mut commands: Commands) {
    // Create the root UI container that will hold our menu
    // This stays spawned but hidden until we need it
    commands.spawn((
        Node {
            // Full screen container
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            // Center children
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Visibility::Hidden,  // Use Visibility to toggle menu
        GlobalZIndex(999),
        ContextMenuRoot,
    ))
    .with_children(|parent| {
        // The actual menu box
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(4.0)),
                min_width: Val::Px(200.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            BorderColor(WHITE.into()),
            ContextMenuBox,
        ));
    });

    // Message log UI at the bottom of the screen
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(96.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.07, 0.07, 0.1)),
        BorderColor(WHITE.into()),
        GlobalZIndex(900),
        Visibility::Hidden,
        MessageLogRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont { font_size: 18.0, ..default() },
            TextColor(WHITE.into()),
            MessageText,
        ));

        // Continue chevron in bottom-right, hidden until we have more lines
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(6.0),
                right: Val::Px(10.0),
                ..default()
            },
            Visibility::Hidden,
            ContinueChevron,
            ChevronBlink { timer: Timer::from_seconds(0.5, TimerMode::Repeating) },
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("v"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(WHITE.into()),
            ));
        });

        // Close chevron (last line indicator) — shows an 'x' on the last page
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(6.0),
                right: Val::Px(10.0),
                ..default()
            },
            Visibility::Hidden,
            CloseChevron,
            ChevronBlink { timer: Timer::from_seconds(0.8, TimerMode::Repeating) },
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("x"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(WHITE.into()),
            ));
        });
    });

    // Inventory panel UI (hidden by default)
    commands.spawn((
        Node {
            width: Val::Px(260.0),
            min_height: Val::Px(140.0),
            position_type: PositionType::Absolute,
            top: Val::Px(24.0),
            right: Val::Px(24.0),
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
        BorderColor(WHITE.into()),
        GlobalZIndex(925),
        Visibility::Hidden,
        InventoryRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Inventory"),
            TextFont { font_size: 22.0, ..default() },
            TextColor(YELLOW.into()),
        ));
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
            InventoryList,
        ));
    });
}

fn show_context_menu(
    mut events: EventReader<ContextMenuEvent>,
    mut commands: Commands,
    mut menu_root_query: Query<(Entity, &mut Visibility, &Children), With<ContextMenuRoot>>,
    menu_box_query: Query<(Entity, Option<&Children>), With<ContextMenuBox>>,
    mut ui_state: ResMut<UiState>,
    time: Res<Time>,
) {
    for event in events.read() {
        if let Ok((_root_entity, mut visibility, children)) = menu_root_query.single_mut() {
            // Show the menu
            *visibility = Visibility::Visible;
            ui_state.menu_open = true;
            ui_state.selected_index = 0;
            ui_state.current_entity = Some(event.entity);
            ui_state.current_actions = event.actions.clone();
            ui_state.menu_opened_at = time.elapsed().as_secs_f64();
            
            // Get the menu box entity
            if let Some(&menu_box_entity) = children.first() {
                if let Ok((menu_box, maybe_children)) = menu_box_query.get(menu_box_entity) {
                    // Clear any previous title/options under the menu box
                    if let Some(children_to_clear) = maybe_children {
                        for child in children_to_clear.iter() {
                            commands.entity(child).despawn();
                        }
                    }

                    // Add title and options
                    commands.entity(menu_box).with_children(|parent| {
                        parent.spawn((
                            Text::new(format!("[ {} ]", event.object_name)),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(WHITE.into()),
                            Node {
                                margin: UiRect::bottom(Val::Px(10.0)),
                                align_self: AlignSelf::Center,
                                ..default()
                            },
                        ));
                        
                        // Add each menu option
                        for (index, action) in event.actions.iter().enumerate() {
                            let is_selected = index == 0;
                            parent.spawn((
                                Text::new(action.to_string()),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(if is_selected { 
                                    YELLOW.into() 
                                } else { 
                                    WHITE.into() 
                                }),
                                Node {
                                    padding: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                                MenuOption { index },
                            ));
                        }
                    });
                    
                    info!("Menu opened for {} with {} actions", event.object_name, event.actions.len());
                }
            }
        }
    }
}

fn handle_menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UiState>,
    mut option_query: Query<(&MenuOption, &mut TextColor)>,
) {
    if !ui_state.menu_open {
        return;
    }
    
    let option_count = option_query.iter().count();
    if option_count == 0 {
        return;
    }
    
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        if ui_state.selected_index > 0 {
            ui_state.selected_index -= 1;
        } else {
            ui_state.selected_index = option_count - 1;
        }
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        ui_state.selected_index = (ui_state.selected_index + 1) % option_count;
    } else {
        return;
    }
    
    // Update colors
    for (option, mut text_color) in option_query.iter_mut() {
        text_color.0 = if option.index == ui_state.selected_index {
            YELLOW.into()
        } else {
            WHITE.into()
        };
    }
}

fn handle_menu_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_events: EventWriter<InteractionEvent>,
    mut menu_root_query: Query<&mut Visibility, With<ContextMenuRoot>>,
    mut ui_state: ResMut<UiState>,
    time: Res<Time>,
) {
    if !ui_state.menu_open {
        return;
    }
    
    // Debounce: ignore selection in the same frame shortly after opening
    const DEBOUNCE_SECS: f64 = 0.08;
    let since_open = time.elapsed().as_secs_f64() - ui_state.menu_opened_at;
    if since_open < DEBOUNCE_SECS {
        return;
    }

    let select = keyboard.just_pressed(KeyCode::KeyZ)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter);
    
    if select {
        if let Some(entity) = ui_state.current_entity {
            if let Some(action) = ui_state.current_actions.get(ui_state.selected_index) {
                info!("Executing action {:?} on entity {:?}", action, entity);
                interaction_events.write(InteractionEvent {
                    entity,
                    action: action.clone(),
                });
                
                // Hide menu
                if let Ok(mut visibility) = menu_root_query.single_mut() {
                    *visibility = Visibility::Hidden;
                }
                ui_state.menu_open = false;
            }
        }
    }
}

fn handle_menu_cancel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_root_query: Query<&mut Visibility, With<ContextMenuRoot>>,
    mut ui_state: ResMut<UiState>,
) {
    if !ui_state.menu_open {
        return;
    }
    
    let cancel = keyboard.just_pressed(KeyCode::KeyX)
        || keyboard.just_pressed(KeyCode::Escape)
        || keyboard.just_pressed(KeyCode::ShiftLeft);
    
    if cancel {
        if let Ok(mut visibility) = menu_root_query.single_mut() {
            *visibility = Visibility::Hidden;
        }
        ui_state.menu_open = false;
        info!("Menu cancelled");
    }
}

fn update_log_display(
    mut events: EventReader<LogEvent>,
    mut ui_state: ResMut<UiState>,
    mut text_query: Query<&mut Text, With<MessageText>>,
    mut root_vis_query: Query<&mut Visibility, With<MessageLogRoot>>,
    time: Res<Time>,
) {
    let mut received_any = false;
    for e in events.read() {
        ui_state.dialog_queue.push(e.0.clone());
        received_any = true;
    }

    if received_any {
        // If dialog is not open, open it and show the first line
        if !ui_state.dialog_open && !ui_state.dialog_queue.is_empty() {
            ui_state.dialog_open = true;
            ui_state.dialog_index = 0;
            ui_state.dialog_opened_at = time.elapsed().as_secs_f64();
            if let Ok(mut vis) = root_vis_query.single_mut() {
                *vis = Visibility::Visible;
            }
            if let Ok(mut text) = text_query.single_mut() {
                // Show cumulative lines up to current index (first line here)
                let shown = ui_state
                    .dialog_queue
                    .iter()
                    .take(ui_state.dialog_index + 1)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n");
                *text = Text::new(shown);
            }
        }
    }
}

fn handle_dialog_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UiState>,
    mut text_query: Query<&mut Text, With<MessageText>>,
    mut root_vis_query: Query<&mut Visibility, With<MessageLogRoot>>,
    time: Res<Time>,
) {
    if !ui_state.dialog_open {
        return;
    }

    // Debounce to avoid consuming the same key press that opened the dialog
    const DEBOUNCE_SECS: f64 = 0.08;
    let since_open = time.elapsed().as_secs_f64() - ui_state.dialog_opened_at;
    if since_open < DEBOUNCE_SECS {
        return;
    }

    let advance = keyboard.just_pressed(KeyCode::KeyZ)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter);

    if !advance {
        return;
    }

    ui_state.dialog_index += 1;
    if ui_state.dialog_index >= ui_state.dialog_queue.len() {
        // Close dialog
        if let Ok(mut vis) = root_vis_query.single_mut() {
            *vis = Visibility::Hidden;
        }
        if let Ok(mut text) = text_query.single_mut() {
            *text = Text::new(String::new());
        }
        ui_state.dialog_open = false;
        ui_state.dialog_queue.clear();
        ui_state.dialog_index = 0;
        return;
    }

    // Show cumulative lines up to current index
    if let Ok(mut text) = text_query.single_mut() {
        let shown = ui_state
            .dialog_queue
            .iter()
            .take(ui_state.dialog_index + 1)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        *text = Text::new(shown);
    }
}

fn blink_continue_chevron(
    time: Res<Time>,
    ui_state: Res<UiState>,
    mut cont_query: Query<(&mut Visibility, &mut ChevronBlink), (With<ContinueChevron>, Without<CloseChevron>)>,
    mut close_query: Query<(&mut Visibility, &mut ChevronBlink), (With<CloseChevron>, Without<ContinueChevron>)>,
) {
    let dialog_active = ui_state.dialog_open && !ui_state.dialog_queue.is_empty();
    let has_more_after = dialog_active && (ui_state.dialog_index + 1 < ui_state.dialog_queue.len());
    let on_last = dialog_active && (ui_state.dialog_index + 1 == ui_state.dialog_queue.len());

    if let Ok((mut vis, mut blink)) = cont_query.single_mut() {
        if has_more_after {
            blink.timer.tick(time.delta());
            if blink.timer.finished() {
                *vis = match *vis { Visibility::Visible => Visibility::Hidden, _ => Visibility::Visible };
            }
        } else {
            *vis = Visibility::Hidden;
            blink.timer.reset();
        }
    }

    if let Ok((mut vis, mut blink)) = close_query.single_mut() {
        if on_last {
            blink.timer.tick(time.delta());
            if blink.timer.finished() {
                *vis = match *vis { Visibility::Visible => Visibility::Hidden, _ => Visibility::Visible };
            }
        } else {
            *vis = Visibility::Hidden;
            blink.timer.reset();
        }
    }
}

fn update_inventory_ui(
    inventory: Res<Inventory>,
    mut root_query: Query<(&mut Visibility, &Children), With<InventoryRoot>>,
    list_query: Query<(Entity, Option<&Children>), With<InventoryList>>,
    mut commands: Commands,
) {
    if let Ok((mut visibility, children)) = root_query.single_mut() {
        // Toggle visibility
        *visibility = if inventory.is_open { Visibility::Visible } else { Visibility::Hidden };

        // If hidden, no need to rebuild
        if !inventory.is_open { return; }

        // Rebuild the items list whenever the inventory changes
        let mut found_list: Option<Entity> = None;
        for i in 0..children.len() {
            let child = children[i];
            if list_query.get(child).is_ok() {
                found_list = Some(child);
                break;
            }
        }
        if let Some(list_entity) = found_list {
            if let Ok((list, maybe_children)) = list_query.get(list_entity) {
                // Clear old lines
                if let Some(children_to_clear) = maybe_children {
                    for child in children_to_clear.iter() {
                        commands.entity(child).despawn();
                    }
                }
                // Build item lines
                commands.entity(list).with_children(|parent| {
                    if inventory.items.is_empty() {
                        parent.spawn((
                            Text::new("(Empty)"),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(WHITE.into()),
                        ));
                    } else {
                        for item in &inventory.items {
                            parent.spawn((
                                Text::new(format!("* {}", item.name)),
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(WHITE.into()),
                            ));
                        }
                    }
                });
            }
        }
    }
}