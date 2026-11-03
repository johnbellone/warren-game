//! Hotbar HUD: visual overlay showing the 6 block presets with a highlight
//! on the currently selected slot.
//!
//! Rendered by a dedicated 2D `Camera2d` layered on top of the 3D scene.
//! Layout: a horizontal panel of 6 `Node` + `Text` UI entities.
//! The selected slot gets a brighter background and a yellow border.

use bevy::prelude::*;
use bevy::text::TextFont;

use crate::hotbar::Hotbar;

// --------------------------------------------------------------------------
// Components & Resources
// --------------------------------------------------------------------------

/// Marker for a hotbar slot UI entity.
#[derive(Component)]
pub struct HotbarSlot {
    /// Index into `Hotbar.presets` (0 = Stone, 1 = Dirt, … 5 = Glass).
    pub index: usize,
}

/// Marker for the hotbar HUD UI camera.
#[derive(Component)]
pub struct HotbarHudCamera;

/// Tracks the spawned slot entities so the HUD can skip re-spawning.
#[derive(Resource, Default)]
pub struct HotbarHudState {
    pub spawned: bool,
}

// --------------------------------------------------------------------------
// Spawn System
// --------------------------------------------------------------------------

/// Spawns a 2D UI camera and 6 hotbar slot UI entities on the first frame.
pub fn spawn_hotbar_hud(
    mut commands: Commands,
    hotbar: Res<Hotbar>,
    mut state: ResMut<HotbarHudState>,
) {
    if state.spawned {
        return;
    }

    // 2D orthographic camera for UI, slightly in front of the 3D scene.
    commands.spawn((
        Camera2d,
        Camera::default(),
        Projection::Orthographic(OrthographicProjection::default_2d()),
        Transform::from_xyz(0.0, 0.0, 10.0),
        HotbarHudCamera,
    ));

    // Six slots spread horizontally, fixed near the bottom of the screen.
    let slot_width = 88.0_f32;
    let slot_gap = 6.0_f32;
    let slot_count = hotbar.presets.len();
    let total_width = slot_count as f32 * slot_width + (slot_count as f32 - 1.0) * slot_gap;
    let start_x = -total_width / 2.0 + slot_width / 2.0;

    for (i, preset) in hotbar.presets.iter().enumerate() {
        let x = start_x + i as f32 * (slot_width + slot_gap);
        commands.spawn((
            HotbarSlot { index: i },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                bottom: Val::Px(14.0),
                width: Val::Px(slot_width),
                height: Val::Px(52.0),
                border: UiRect::all(Val::Px(2.0)),
                column_gap: Val::Px(4.0),
                row_gap: Val::Px(2.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Text(format!("{}  {}", i + 1, preset.name)),
            TextFont::from_font_size(13.0),
            TextColor(Color::WHITE),
            BackgroundColor(Color::srgba(
                preset.color[0],
                preset.color[1],
                preset.color[2],
                0.28,
            )),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.25)),
        ));
    }

    state.spawned = true;
}

// --------------------------------------------------------------------------
// Update System
// --------------------------------------------------------------------------

/// Highlights the currently selected slot with a brighter background and
/// a gold border; de-highlights all others.
pub fn update_hotbar_hud(
    hotbar: Res<Hotbar>,
    mut slot_q: Query<(&HotbarSlot, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (slot, mut bg, mut border) in &mut slot_q {
        let Some(preset) = hotbar.presets.get(slot.index) else { continue };
        let selected = slot.index == hotbar.selected;
        *bg = BackgroundColor(Color::srgba(
            preset.color[0],
            preset.color[1],
            preset.color[2],
            if selected { 0.88 } else { 0.28 },
        ));
        *border = BorderColor::all(
            if selected {
                Color::srgb(1.0, 0.9, 0.2) // gold = active
            } else {
                Color::srgba(1.0, 1.0, 1.0, 0.25)
            },
        );
    }
}
