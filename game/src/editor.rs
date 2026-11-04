//! In-game editor controls.
//!
//! This is the "convert the editor to be in-game" piece: instead of the Mantine
//! webview, the edit actions live *inside the Bevy scene* as a keyboard-driven
//! system that drives the live `VoxelWorldResource` + `Hotbar` — place, erase,
//! and cycle the brush — so the build/erase actions happen in-game and appear
//! immediately in both the chunk meshes and the path-traced preview
//! (see `path_trace`).
//!
//! Controls:
//!   * `Space`          — place a small patch with the current hotbar colour
//!   * `Delete`/`Backspace` — erase that patch
//!   * `Tab` / `E`      — cycle the hotbar to the next preset
//!   * `Q`              — cycle to the previous preset
//!
//! The visual panel (a `Camera2d` + `Node` + `Text` HUD) is deferred to a
//! follow-up; the `bevy_ui`/`Text`/parent-hierarchy API in Bevy 0.20.1 is a
//! different surface than the OKF notes assumed (`Text::from_label` was removed,
//! `Parent` is now a `ChildEntity` builder). The editor's *behavior* is the part
//! that matters and is tested here.

use bevy::prelude::*;
use bevy::input::ButtonInput;

use crate::hotbar::Hotbar;
use crate::rendering::VoxelWorldResource;

/// The keyboard-driven in-game editor. `Space` places a small patch of voxels
/// just above the edit point with the current hotbar colour; `Delete`/
/// `Backspace` erases it; `Tab`/`E`/`Q` cycle the hotbar. The edit point is the
/// origin for now (the character-foot integration is a follow-up).
pub fn editor_edit(
    mut world_res: ResMut<VoxelWorldResource>,
    mut hotbar: ResMut<Hotbar>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Cycle the brush.
    if keys.just_pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::KeyE) {
        hotbar.cycle_forward();
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        hotbar.cycle_back();
    }

    // Edit at the origin (a small 2x2 patch one voxel off the floor).
    let (ix, iy, iz) = (0i32, GROUND_Y + 1, 0i32);

    if keys.just_pressed(KeyCode::Space) {
        let color = hotbar.current_color();
        let name = hotbar.current_name().to_string();
        place_patch(&mut world_res.world, [ix, iy, iz], color);
        info!("editor: placed '{name}' patch at ({ix},{iy},{iz}) colour {color:?} in-game");
    }
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        erase_patch(&mut world_res.world, [ix, iy, iz]);
        info!("editor: erased patch at ({ix},{iy},{iz})");
    }
}

/// A short floor reference for the editor's edit point.
const GROUND_Y: i32 = -3;

/// Place a 2x2 patch of voxels at `origin` (and its three neighbours).
fn place_patch(world: &mut crate::world::VoxelWorld, origin: [i32; 3], color: [f32; 3]) {
    world.place([origin[0], origin[1], origin[2]], color);
    world.place([origin[0] + 1, origin[1], origin[2]], color);
    world.place([origin[0], origin[1], origin[2] + 1], color);
    world.place([origin[0] + 1, origin[1], origin[2] + 1], color);
}

/// Erase a 2x2 patch of voxels at `origin`.
fn erase_patch(world: &mut crate::world::VoxelWorld, origin: [i32; 3]) {
    world.erase([origin[0], origin[1], origin[2]]);
    world.erase([origin[0] + 1, origin[1], origin[2]]);
    world.erase([origin[0], origin[1], origin[2] + 1]);
    world.erase([origin[0] + 1, origin[1], origin[2] + 1]);
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_then_erase_is_a_round_trip() {
        let mut w = crate::world::VoxelWorld::new([16, 32, 16]);
        assert_eq!(w.voxel_count(), 0);
        place_patch(&mut w, [1, 1, 1], [0.5, 0.2, 0.1]);
        assert_eq!(w.voxel_count(), 4, "a 2x2 patch places 4 voxels");
        assert!(w.contains([1, 1, 1]) && w.contains([2, 1, 1]) && w.contains([1, 1, 2]) && w.contains([2, 1, 2]));
        erase_patch(&mut w, [1, 1, 1]);
        assert_eq!(w.voxel_count(), 0, "erasing the patch removes all 4");
    }

    #[test]
    fn hotbar_cycle_forward_wraps_around() {
        let mut hb = Hotbar::default();
        let start = hb.selected;
        hb.cycle_forward();
        assert_ne!(hb.selected, start, "cycle_forward must advance the selection");
        // Cycle the remaining presets (6 total) to return to the start.
        for _ in 0..5 {
            hb.cycle_forward();
        }
        assert_eq!(hb.selected, start, "cycling once around must return to start");
    }
}
