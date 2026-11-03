//! Stylized, to-scale U.S. Capitol "neighborhood" generator.
//!
//! Phase 1 of the chunked-storage work: fill the world with a recognizable
//! blocky U.S. Capitol (podium + rotunda + dome + four wings) at 1 voxel = 1 m
//! so a ~1.85 m person stands next to it at human scale. The result is only a
//! few thousand voxels, which the per-voxel renderer still handles on a modern
//! GPU. (Phase 2 replaces per-voxel meshes with greedy chunk-mesh culling.)
//!
//! All geometry is centered on the world origin; the dome rises to ~y=20.

use crate::world::VoxelWorld;
use crate::rendering::VoxelWorldResource;
use bevy::prelude::*;

// --- colors (RGB, 0..1) --------------------------------------------------

/// Startup system: populate the world with the stylized U.S. Capitol
/// neighborhood. Runs once on app start (before rendering begins).
pub fn load_capitol_neighborhood(mut world_res: ResMut<VoxelWorldResource>) {
    populate_capitol_neighborhood(&mut world_res.world);
    info!("Loaded U.S. Capitol neighborhood: {} voxels in {} chunks", world_res.world.voxel_count(), world_res.world.chunk_count());
}

/// Cream limestone (Capitol exterior).
const STONE: [f32; 3] = [0.93, 0.91, 0.83];
/// Patina copper (the dome).
const COPPER: [f32; 3] = [0.53, 0.71, 0.58];
/// Dark glass for a few window bands.
const GLASS: [f32; 3] = [0.15, 0.18, 0.30];

/// Build a stylized U.S. Capitol centered on the world origin.
///
/// The world should be large enough that `y` can reach ~20 m (dome top).
/// Pass `VoxelWorld::new([64, 128, 64])` or similar for this scene.
pub fn populate_capitol_neighborhood(world: &mut VoxelWorld) {
    build_podium(world);
    build_wings(world);
    build_rotunda(world);
    build_dome(world);
    build_statue(world);
}

/// A broad low platform the whole building sits on.
fn build_podium(world: &mut VoxelWorld) {
    // 32 (x) × 22 (z) footprint, 3 m tall.
    box_(world, -16..=16, 0..=2, -11..=11, STONE);
    // A slightly smaller step on top for visual contrast.
    box_(world, -13..=13, 3..=4, -8..=8, [0.88, 0.86, 0.78]);
}

/// Four wings extending N/S and E/W from the central mass, each with a window
/// band down the middle.
fn build_wings(world: &mut VoxelWorld) {
    // North & south wings extend along X.
    for z_side in [-1_i32, 1] {
        let z0 = z_side * 8;
        let z1 = z_side * 11;
        let (a, b) = if z_side < 0 { (z1, z0) } else { (z0, z1) };
        box_(world, -18..=18, 5..=9, a..=b, STONE);
        // Dark window band.
        box_(world, -16..=16, 7..=8, a..=b, GLASS);
    }
    // East & west wings extend along Z.
    for x_side in [-1_i32, 1] {
        let x0 = x_side * 8;
        let x1 = x_side * 11;
        let (a, b) = if x_side < 0 { (x1, x0) } else { (x0, x1) };
        box_(world, a..=b, 5..=9, -18..=18, STONE);
        box_(world, a..=b, 7..=8, -16..=16, GLASS);
    }
}

/// The central rotunda — a cylindrical drum the dome sits on.
fn build_rotunda(world: &mut VoxelWorld) {
    // Solid cylinder radius 8, from y=5 to y=16.
    cylinder(world, 8, 5, 16, STONE);
    // A ring of windows (the rotunda colonnade) at the drum top.
    cylinder(world, 8, 15, 15, GLASS);
    // A wider base ring where the drum meets the platform.
    cylinder(world, 9, 5, 5, STONE);
}

/// The dome: a copper hemisphere sitting on the rotunda drum.
fn build_dome(world: &mut VoxelWorld) {
    // Hemisphere radius 9 from y=16 upward, copper patina color.
    hemisphere(world, 9, 16, COPPER);
}

/// The Statue of Freedom at the very top — a small block.
fn build_statue(world: &mut VoxelWorld) {
    let top = 16 + 9; // hemisphere top
    box_(world, -1..=1, top..=top + 3, -1..=1, [0.85, 0.75, 0.35]); // gold-ish
}

// --- shape helpers -------------------------------------------------------

/// Place a solid box (inclusive ranges) with `color`.
fn box_(world: &mut VoxelWorld, x: std::ops::RangeInclusive<i32>, y: std::ops::RangeInclusive<i32>,
        z: std::ops::RangeInclusive<i32>, color: [f32; 3]) {
    for py in y {
        for pz in z.clone() {
            for px in x.clone() {
                world.place([px, py, pz], color);
            }
        }
    }
}

/// Place a solid vertical cylinder (radius r) from y0..=y1.
fn cylinder(world: &mut VoxelWorld, r: i32, y0: i32, y1: i32, color: [f32; 3]) {
    let r2 = r as f32 * r as f32;
    for py in y0..=y1 {
        for dz in -r..=r {
            for dx in -r..=r {
                let d2 = dx as f32 * dx as f32 + dz as f32 * dz as f32;
                if d2 <= r2 {
                    world.place([dx, py, dz], color);
                }
            }
        }
    }
}

/// Place a solid hemisphere (radius r) rising from `y_base` upward.
fn hemisphere(world: &mut VoxelWorld, r: i32, y_base: i32, color: [f32; 3]) {
    for dy in 0..=r {
        let max_d = ((r as f32 * r as f32 - dy as f32 * dy as f32).sqrt()) as i32;
        for dz in -max_d..=max_d {
            let max_dx = (((r as f32 * r as f32) - dz as f32 * dz as f32 - dy as f32 * dy as f32).sqrt()) as i32;
            for dx in -max_dx..=max_dx {
                world.place([dx, y_base + dy, dz], color);
            }
        }
    }
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::VoxelWorld;

    #[test]
    fn capitol_populates_a_sensible_count() {
        let mut world = VoxelWorld::new([64, 128, 64]);
        populate_capitol_neighborhood(&mut world);
        // Recognizable Capitol should be a few thousand voxels, not millions.
        let n = world.voxel_count();
        assert!(n > 1500, "too few: {n}");
        assert!(n < 30_000, "too many: {n}");
    }

    #[test]
    fn capitol_has_a_dome_above_y15() {
        let mut world = VoxelWorld::new([64, 128, 64]);
        populate_capitol_neighborhood(&mut world);
        // The dome / statue should reach well above the drum.
        let top = (0..world.voxel_count()).count(); // noop to touch count
        let _ = top;
        assert!(world.get([0, 25, 0]).is_some() || world.get([0, 24, 0]).is_some()
            || world.get([0, 26, 0]).is_some());
    }

    #[test]
    fn capitol_is_spread_over_multiple_chunks() {
        let mut world = VoxelWorld::new([64, 128, 64]);
        populate_capitol_neighborhood(&mut world);
        // y reaches ~25, x ±24, z ±24 -> at least 2 chunks on the y axis alone.
        assert!(world.chunk_count() >= 2);
    }
}
