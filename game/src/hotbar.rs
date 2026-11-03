//! Hotbar system — Minecraft-style block type switcher.
//!
//! Number keys 1-6 select block presets, E cycles forward, Q cycles backward.
//! The selected block's color is used when placing new voxels.

use bevy::prelude::*;

// --------------------------------------------------------------------------
// Block Presets
// --------------------------------------------------------------------------

/// A named block preset with a display name and RGB color.
#[derive(Debug, Clone, Copy)]
pub struct BlockPreset {
    pub name: &'static str,
    pub color: [f32; 3],
    pub key: u8,
}

// --------------------------------------------------------------------------
// Hotbar Resource
// --------------------------------------------------------------------------

/// The hotbar holds the list of block presets and the currently selected index.
#[derive(Resource)]
pub struct Hotbar {
    pub presets: Vec<BlockPreset>,
    pub selected: usize,
}

impl Default for Hotbar {
    fn default() -> Self {
        Hotbar::new()
    }
}

impl Hotbar {
    /// Build a default hotbar with 6 block presets.
    pub fn new() -> Self {
        Self {
            presets: vec![
                BlockPreset { name: "Stone",  color: [0.55, 0.55, 0.55], key: 1 },
                BlockPreset { name: "Dirt",   color: [0.65, 0.40, 0.25], key: 2 },
                BlockPreset { name: "Grass",  color: [0.35, 0.70, 0.20], key: 3 },
                BlockPreset { name: "Wood",   color: [0.50, 0.35, 0.15], key: 4 },
                BlockPreset { name: "Brick",  color: [0.80, 0.25, 0.20], key: 5 },
                BlockPreset { name: "Glass",  color: [0.75, 0.90, 0.95], key: 6 },
            ],
            selected: 0,
        }
    }

    /// Get the current block's color.
    pub fn current_color(&self) -> [f32; 3] {
        self.presets.get(self.selected).map(|p| p.color).unwrap_or([0.5, 0.5, 0.5])
    }

    /// Get the current block's name.
    pub fn current_name(&self) -> &'static str {
        self.presets.get(self.selected).map(|p| p.name).unwrap_or("Unknown")
    }

    /// Cycle to the next preset (wrap-around).
    pub fn cycle_forward(&mut self) {
        if !self.presets.is_empty() {
            self.selected = (self.selected + 1) % self.presets.len();
        }
    }

    /// Cycle to the previous preset (wrap-around).
    pub fn cycle_back(&mut self) {
        if !self.presets.is_empty() && self.selected > 0 {
            self.selected -= 1;
        }
    }
}

// --------------------------------------------------------------------------
// Input System
// --------------------------------------------------------------------------

/// Handle hotbar input:
/// - Keys 1-6 select directly
/// - E / PageUp cycles forward
/// - Q / PageDown / Backspace cycles backward
pub fn update_hotbar(
    key_input: Res<ButtonInput<KeyCode>>,
    mut hotbar: ResMut<Hotbar>,
) {
    // Number keys 1-6 select a preset directly
    let mut matched_index: Option<usize> = None;
    for (i, preset) in hotbar.presets.iter().enumerate() {
        let key = match preset.key {
            1 => KeyCode::Digit1,
            2 => KeyCode::Digit2,
            3 => KeyCode::Digit3,
            4 => KeyCode::Digit4,
            5 => KeyCode::Digit5,
            6 => KeyCode::Digit6,
            _ => continue,
        };
        if key_input.just_pressed(key) {
            matched_index = Some(i);
            break;
        }
    }
    if let Some(i) = matched_index {
        hotbar.selected = i;
        let name = hotbar.current_name();
        eprintln!("[hotbar] selected: {name}");
        return;
    }

    // E / PageUp: next preset
    if key_input.just_pressed(KeyCode::KeyE) || key_input.just_pressed(KeyCode::PageUp) {
        hotbar.cycle_forward();
        eprintln!("[hotbar] \u{2192} {}", hotbar.current_name());
        return;
    }

    // Q / PageDown / Backspace: previous preset
    if key_input.just_pressed(KeyCode::KeyQ)
        || key_input.just_pressed(KeyCode::PageDown)
        || key_input.just_pressed(KeyCode::Backspace)
    {
        hotbar.cycle_back();
        eprintln!("[hotbar] \u{2192} {}", hotbar.current_name());
    }
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotbar_default_has_six_presets() {
        let hb = Hotbar::new();
        assert_eq!(hb.presets.len(), 6);
        assert_eq!(hb.selected, 0);
    }

    #[test]
    fn test_current_color_from_selected() {
        let mut hb = Hotbar::new();
        assert_eq!(hb.current_color(), hb.presets[0].color);
        hb.selected = 2;
        assert_eq!(hb.current_color(), hb.presets[2].color);
        assert_eq!(hb.current_name(), "Grass");
    }

    #[test]
    fn test_cycle_forward_wraps() {
        let mut hb = Hotbar::new();
        assert_eq!(hb.selected, 0);
        hb.cycle_forward();
        assert_eq!(hb.selected, 1);
        for _ in 0..5 {
            hb.cycle_forward();
        }
        assert_eq!(hb.selected, 0, "should wrap back to 0 after cycling past end");
    }

    #[test]
    fn test_cycle_back_stops_at_zero() {
        let mut hb = Hotbar::new();
        assert_eq!(hb.selected, 0);
        hb.cycle_back();
        assert_eq!(hb.selected, 0, "cycle_back at 0 should not underflow");
    }

    #[test]
    fn test_select_by_index() {
        let mut hb = Hotbar::new();
        hb.selected = 3; // Wood
        assert_eq!(hb.current_name(), "Wood");
    }
}
