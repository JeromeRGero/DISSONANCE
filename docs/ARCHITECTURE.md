## Dissonance Architecture

This document describes the structure of the codebase, the flow of data and control between systems, and the conventions used throughout. The project is built with Bevy (ECS) and is intentionally small, readable, and deterministic per frame.

### High-level Overview

- Entry: `src/main.rs`
  - Configures Bevy plugins and windowing
  - Defines `GameSet` for deterministic system ordering
  - Adds core plugins: `PlayerPlugin`, `InteractionPlugin`, `InventoryPlugin`, `ObjectsPlugin`, `UiPlugin`

- Gameplay core (ECS):
  - Components hold state: `Player`, `Interactable`, `Light`, `Door`, `Generator`, `Item`, `Solid`
  - Resources hold global state: `Inventory`, `UiState`
  - Events connect systems: `InteractionEvent`, `ContextMenuEvent`, `LogEvent`
  - Systems are grouped into sets and chained: Detect → Input → Ui → Process

### System Ordering (GameSet)

- Detect: proximity detection, markers
- Input: read inputs, create UI or interaction events
- Ui: open/close context menus, navigate/select
- Process: apply game logic, log messages, update UI panels

This ordering is enforced per frame via `.configure_sets(Update, (...).chain())` in `main.rs`.

---

## Modules

### `src/main.rs`

- Sets up default Bevy plugins and pixel-art rendering.
- Creates the 2D camera and clear color.
- Registers all game plugins and `GameSet` ordering.

### `src/player.rs` (Player and Camera)

Components
- `Player { speed, interact_range, facing }`
- `InteractionIndicator` child sprite toggled when a target is nearby

Systems
- `spawn_player`: spawns player sprite and child indicator
- `player_movement`: WASD/arrow movement with AABB collision vs `Solid` sprites
  - Axis-aligned step on X then Y with overlap resolution
  - Player AABB uses `Vec2(8, 10)` half-extents approximating the sprite
- `update_player_facing`: updates `facing` based on input
- `camera_follow_player`: positions camera over player each frame

Notes
- Collision checks the player's bounding box vs each `Solid` sprite's `custom_size` or default size. The map uses slightly overlapping walls to avoid visible seams and collision leaks.

### `src/inventory.rs` (Inventory Resource)

Resources/Types
- `Inventory { items, max_size, is_open }`
- `InventoryItem { id, name, description, icon_color }`

Systems
- `toggle_inventory_display` (Input set): toggle inventory view and print list to log (console)

Helpers
- `add_item`, `remove_item`
- `has_item_id(&str) -> bool`
- `take_item_by_id(&str) -> bool` (consume one matching item)

### `src/ui.rs` (UI, Context Menu, Message Log)

Resources
- `UiState { menu_open, selected_index, current_entity, current_actions, menu_opened_at, dialog_open, dialog_queue, dialog_index, dialog_opened_at }`

Events
- `ContextMenuEvent { entity, actions, object_name }`
- `LogEvent(String)`

Systems (Ui set)
- `setup_ui`: builds UI tree for menu, message log, and inventory panel
- `show_context_menu`: shows a centered popup listing actions for the target entity
- `handle_menu_navigation`: UP/DOWN navigation with visual highlight
- `handle_menu_selection`: Z/Space/Enter triggers an `InteractionEvent`
- `handle_menu_cancel`: X/Esc closes the menu

Systems (Process set)
- `update_log_display` and `handle_dialog_input`: Undertale-style queued log
- `blink_continue_chevron`: blinking indicator for more pages/last page
- `update_inventory_ui`: rebuilds the inventory list when open

### `src/interaction.rs` (Proximity, Menu, Interactions)

Types
- `Interactable { name, actions, interaction_radius }`
- `InteractionAction`: high-level intents (Examine, Take, Use, TurnOn, TurnOff, Refuel, Talk, Open, Close)
- `InteractionEvent { entity, action, with_item_id: Option<String>, detailed: bool }`
  - We keep actions simple; optional context can be carried here as needed in future flows

Systems
- `check_nearby_interactables` (Detect):
  - Finds closest `Interactable` within its radius
  - Toggles `InteractionIndicator` visibility
- `handle_interaction_input` (Input):
  - On interaction key, either fires an immediate `InteractionEvent` (1 action) or opens a menu
  - Dynamically tailors actions based on target components:
    - `Light`: replaces TurnOn/TurnOff to match current `is_on`
    - `Door`: replaces Open/Close to match current `is_open`
- `process_interactions` (Process): applies game logic and writes `LogEvent`s
  - `Examine`, `Use`, `Talk`, etc. default messages
  - Light
    - TurnOn/TurnOff flip `Light.is_on` and recolor the entity sprite
  - Take
    - Adds an `InventoryItem` with `id = interactable.name`
    - Despawns the world entity
  - Door
    - Open: requires `required_key_id` if present; consumes it from inventory; removes `Solid`; hides the door sprite; marks `is_open = true`
    - Close: re-adds `Solid`; shows the door; marks `is_open = false`

### `src/objects.rs` (World Layout)

Components
- `Item { name, can_pickup }`
- `Light { is_on }`
- `Door { is_open, required_key_id }`
- `Generator { is_running, fuel_level, max_fuel }`
- `NPC { name, dialogue }`
- `Solid` (marker for collision)

Spawned Layout
- Hex-like starter room (axis-aligned rectangles):
  - Top/Bottom walls overlap with side walls to remove seams
  - Full-height left wall
  - Right wall split into upper/lower segments around the door gap
- Keyed door in the gap
  - `Interactable("Metal Door")`, `Door { is_open: false, required_key_id: Some("Rusty Key") }`, `Solid`
  - Door sprite visibility toggles on open/close
- Hallway
  - Two parallel bars (top/bottom) extended/thickened and nudged into the room to seal leaks
- Generator
  - Placed at hallway end; slightly overlaps the hallway to avoid escape gaps
- Items/NPC
  - Rusty Key inside the room (has `Solid` so it blocks until picked up)
  - Lamp inside the room (toggleable)
  - Optional NPC

---

## Data and Control Flows

Interaction Flow
1. Player presses interact (Z/Space/Enter)
2. `handle_interaction_input` finds nearest `Interactable`
3. If one action → `InteractionEvent` dispatched; else → `ContextMenuEvent` opens menu
4. UI shows menu; selection dispatches `InteractionEvent`
5. `process_interactions` mutates components/resources and emits `LogEvent`s

Inventory Flow
- Picking up an item adds an `InventoryItem { id, name, ... }`
- Doors check `inventory.has_item_id(key)` and may `take_item_by_id(key)` on open

Collision
- `player_movement` resolves AABB collisions vs `Solid` sprites by axis
- World geometry intentionally overlaps at edges to prevent seam leaks

Rendering
- Pixel-perfect style via `ImagePlugin::default_nearest()`
- Sprites are recolored to represent state changes (lamp on/off, door open/closed color hint)

Camera
- Single `Camera2d` follows the player each frame

---

## Build & Run

Controls
- Move: WASD / Arrow keys
- Interact: Z / Space / Enter
- Cancel: X / Esc / Left Shift
- Inventory: I

Run
```bash
cargo run
```

Rust/Bevy
- Targeted Bevy API is current as of the project; see `Cargo.toml`

---

## Extensibility & Future Work

- Persistence
  - Introduce a save/load resource that serializes component state (e.g., `Light.is_on`, `Door.is_open`, inventory contents)
  - Consider serde for resources and selected components

- Action Context
  - `InteractionEvent` already supports optional context; extend UI for "Use item with ..." flows when needed

- AI / NPC
  - Expand `NPC.dialogue` branching and add `Talk { topic }` context

- World Building
  - Move hard-coded layout to a lightweight scene/level format
  - Add tiles or meshes if needed, retaining AABB collision

- UX / UI
  - Visual context menu cursor, sfx, better fonts
  - Save/load UI, settings

---

## File Map

- `src/main.rs` — app setup, sets, plugins
- `src/player.rs` — player movement/camera, facing
- `src/inventory.rs` — inventory resource and helpers
- `src/interaction.rs` — interactables, events, action processing
- `src/ui.rs` — menu, dialog log, inventory panel
- `src/objects.rs` — world spawn, components (Item/Light/Door/Generator/NPC/Solid)
- `Cargo.toml` — dependencies


