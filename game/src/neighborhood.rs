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

/// Cream limestone (Capitol exterior).
const STONE: [f32; 3] = [0.93, 0.91, 0.83];
/// Patina copper (the dome).
const COPPER: [f32; 3] = [0.53, 0.71, 0.58];
/// Dark glass for a few window bands.
const GLASS: [f32; 3] = [0.15, 0.18, 0.30];
/// Gold-ish Statue of Freedom.
const GOLD: [f32; 3] = [0.85, 0.75, 0.35];
/// Plaza stone (the 10-deep ground).
const PLAZA: [f32; 3] = [0.55, 0.53, 0.50];
/// Grass on the top layer of the ground (visual interest).
const GRASS: [f32; 3] = [0.30, 0.55, 0.25];

// --- resolution ----------------------------------------------------------

/// Voxel resolution multiplier: 3× means every 1 m of the original generator
/// is 3 voxels, so each building is ~3× denser along every axis and the total
/// voxel count is ~27× the pre-scale ~9,699 (~262k voxels). The voxel world
/// stores integer coordinates in *voxel units*; multiplying every placed
/// coordinate by `SCALE` is exactly "a finer voxel grid" — the physical scene
/// stays the same size, but with 3× the resolution.
pub const SCALE: i32 = 3;

/// Depth of the procedural voxel ground in voxels (a slab 10 layers deep, below
/// the origin at y = 0, occupying y = -(GROUND_DEPTH)..=-1).
pub const GROUND_DEPTH: i32 = 10;

/// Half-extent (in voxel units) of the square plaza the world sits on. Chosen
/// wide enough to cover the scaled Capitol footprint (~±54 at SCALE=3).
pub const GROUND_HALF: i32 = 64;

/// Startup system: populate the world with the stylized U.S. Capitol
/// neighborhood. Runs once on app start (before rendering begins).
pub fn load_capitol_neighborhood(mut world_res: ResMut<VoxelWorldResource>) {
    populate_capitol_neighborhood(&mut world_res.world);
    info!(
        "Loaded U.S. Capitol neighborhood: {} voxels in {} chunks (ground {} deep, resolution {}x)",
        world_res.world.voxel_count(),
        world_res.world.chunk_count(),
        GROUND_DEPTH,
        SCALE,
    );
}

/// A stylized U.S. Capitol centered on the world origin, sitting on a
/// 10-deep procedural plaza.
///
/// The world should be large enough that `y` can reach the dome top (~25·SCALE).
/// Pass `VoxelWorld::new([128, 384, 128])` or similar for this scene.
pub fn populate_capitol_neighborhood(world: &mut VoxelWorld) {
    build_ground(world);
    build_podium(world);
    build_wings(world);
    build_rotunda(world);
    build_dome(world);
    build_statue(world);
}

/// Procedural 10-deep voxel plaza, generated *below* the origin so the Capitol
/// and the character rest on its top (y = 0). The top layer is grass, the 9
/// beneath are plaza stone. No overlapping planes — each layer is a distinct
/// voxel height, so the mesher emits one top face per column and there is no
/// z-fighting or blinking. (This replaces the old textured ground+grid plane.)
fn build_ground(world: &mut VoxelWorld) {
    let half = GROUND_HALF;
    for y in -(GROUND_DEPTH)..=-1_i32 {
        let color = if y == -1 { GRASS } else { PLAZA };
        for pz in -half..=half {
            for px in -half..=half {
                world.place([px, y, pz], color);
            }
        }
    }
}

/// A solid `RangeInclusive` scaled by `SCALE` (a resolution-3x coordinate).
fn s(r: std::ops::RangeInclusive<i32>) -> std::ops::RangeInclusive<i32> {
    std::ops::RangeInclusive::new(r.start().saturating_mul(SCALE), r.end().saturating_mul(SCALE))
}

/// A broad low platform the whole building sits on.
fn build_podium(world: &mut VoxelWorld) {
    // 32 (x) × 22 (z) footprint, 3 m tall (×SCALE resolution).
    box_(world, s(-16..=16), s(0..=2), s(-11..=11), STONE);
    // A slightly smaller step on top for visual contrast.
    box_(world, s(-13..=13), s(3..=4), s(-8..=8), [0.88, 0.86, 0.78]);
}

/// Four wings extending N/S and E/W from the central mass, each with a window
/// band down the middle.
fn build_wings(world: &mut VoxelWorld) {
    // North & south wings extend along X.
    for z_side in [-1_i32, 1] {
        let z0 = z_side * 8;
        let z1 = z_side * 11;
        let (a, b) = if z_side < 0 { (z1, z0) } else { (z0, z1) };
        box_(world, s(-18..=18), s(5..=9), s(a..=b), STONE);
        // Dark window band.
        box_(world, s(-16..=16), s(7..=8), s(a..=b), GLASS);
    }
    // East & west wings extend along Z.
    for x_side in [-1_i32, 1] {
        let x0 = x_side * 8;
        let x1 = x_side * 11;
        let (a, b) = if x_side < 0 { (x1, x0) } else { (x0, x1) };
        box_(world, s(a..=b), s(5..=9), s(-18..=18), STONE);
        box_(world, s(a..=b), s(7..=8), s(-16..=16), GLASS);
    }
}

/// The central rotunda — a cylindrical drum the dome sits on.
fn build_rotunda(world: &mut VoxelWorld) {
    // Solid cylinder radius 8, from y=5 to y=16.
    cylinder_sc(world, 8, 5, 16, STONE);
    // A ring of windows (the rotunda colonnade) at the drum top.
    cylinder_sc(world, 8, 15, 15, GLASS);
    // A wider base ring where the drum meets the platform.
    cylinder_sc(world, 9, 5, 5, STONE);
}

/// The dome: a copper hemisphere sitting on the rotunda drum.
fn build_dome(world: &mut VoxelWorld) {
    // Hemisphere radius 9 from y=16 upward, copper patina color.
    hemisphere_sc(world, 9, 16, COPPER);
}

/// The Statue of Freedom at the very top — a small block.
fn build_statue(world: &mut VoxelWorld) {
    let top = 16 + 9; // hemisphere top
    box_(world, s(-1..=1), s(top..=top + 3), s(-1..=1), GOLD); // gold-ish
}

// --- shape helpers -------------------------------------------------------

/// A solid `RangeInclusive` in a given axis multiplied by `SCALE`.
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

/// A solid vertical cylinder (radius r) from y0..=y1, all coordinates ×SCALE.
fn cylinder_sc(world: &mut VoxelWorld, r: i32, y0: i32, y1: i32, color: [f32; 3]) {
    let r = r.saturating_mul(SCALE);
    let y0 = y0.saturating_mul(SCALE);
    let y1 = y1.saturating_mul(SCALE);
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

/// A solid hemisphere (radius r) rising from `y_base` upward, all ×SCALE.
fn hemisphere_sc(world: &mut VoxelWorld, r: i32, y_base: i32, color: [f32; 3]) {
    let r = r.saturating_mul(SCALE);
    let y_base = y_base.saturating_mul(SCALE);
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

    fn make_world() -> VoxelWorld {
        VoxelWorld::new([128, 384, 128])
    }

    #[test]
    fn capitol_populates_a_sensible_count() {
        let mut world = make_world();
        populate_capitol_neighborhood(&mut world);
        // With SCALE=3 the dense Capitol + 10-deep plaza is hundreds of
        // thousands of voxels, not millions.
        let n = world.voxel_count();
        assert!(n > 100_000, "too few: {n}");
        assert!(n < 600_000, "too many: {n}");
    }

    #[test]
    fn capitol_has_a_dome_above_y15() {
        let mut world = make_world();
        populate_capitol_neighborhood(&mut world);
        // The dome / statue should reach well above the drum: at SCALE=3 the
        // dome top (~y=25) becomes ~y=75. Probe a column of candidate heights.
        let mut found = false;
        for y in (30..=90).step_by(1) {
            if world.get([0, y, 0]).is_some() {
                found = true;
                break;
            }
        }
        assert!(found, "no dome voxels found above y=30");
    }

    #[test]
    fn capitol_is_spread_over_multiple_chunks() {
        let mut world = make_world();
        populate_capitol_neighborhood(&mut world);
        // y reaches ~75, ground y -10..=-1, x±64 z±64 -> many chunks.
        assert!(world.chunk_count() >= 4);
    }

    #[test]
    fn ground_is_ten_voxels_deep_below_origin() {
        // Probe a plaza corner column (no Capitol sits there) so we can count
        // the ground layers in isolation.
        let px = GROUND_HALF - 2;
        let pz = GROUND_HALF - 2;
        let mut world = VoxelWorld::new([128, 384, 128]);
        super::build_ground(&mut world);
        // The 10 layers below the origin must all be filled...
        for y in -(GROUND_DEPTH)..=-1_i32 {
            assert!(world.get([px, y, pz]).is_some(), "missing ground layer at y={y}");
        }
        // ...and nothing pokes through the top (y = 0) on a bare plaza.
        assert!(!world.get([px, 0, pz]).is_some(), "ground must not poke through y=0");
        assert_eq!(GROUND_DEPTH, 10, "ground must be exactly 10 voxels deep");
    }

    #[test]
    fn resolution_scale_is_three() {
        // The denser-resolution requirement is encoded as a 3x factor.
        assert_eq!(SCALE, 3, "voxel resolution must be 3x");
        // And a 3x-scale generator yields ~27x the voxel count of the 1x one.
        let mut big = VoxelWorld::new([128, 384, 128]);
        populate_capitol_neighborhood(&mut big);
        assert!(big.voxel_count() >= 9_699 * 20,
            "3x resolution should grow the voxel count well past the 1x ~9,699 baseline");
    }
}
