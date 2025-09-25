use bevy::prelude::*;
use crate::GameSet;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Inventory::new(8))
            .add_systems(Update, toggle_inventory_display.in_set(GameSet::Input));
    }
}

#[derive(Resource)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
    pub max_size: usize,
    pub is_open: bool,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new(8)
    }
}

impl Inventory {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: Vec::new(),
            max_size,
            is_open: false,
        }
    }

    pub fn add_item(&mut self, item: InventoryItem) -> bool {
        if self.items.len() < self.max_size {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    pub fn remove_item(&mut self, index: usize) -> Option<InventoryItem> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn take_item_by_id(&mut self, id: &str) -> bool {
        if let Some(pos) = self.items.iter().position(|i| i.id == id) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn has_item_id(&self, id: &str) -> bool {
        self.items.iter().any(|i| i.id == id)
    }
}

#[derive(Clone)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_color: Color,
}

fn toggle_inventory_display(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
) {
    // Toggle with I key
    if keyboard.just_pressed(KeyCode::KeyI) {
        inventory.is_open = !inventory.is_open;
        if inventory.is_open {
            info!("=== INVENTORY ===");
            if inventory.items.is_empty() {
                info!("* Empty");
            } else {
                for item in &inventory.items {
                    info!("* {}", item.name);
                }
            }
            info!("================");
        }
    }
}