//! Voxel-level collision resolution (Minecraft-style swept AABB vs the grid).
//!
//! Replaces the single static ground slab the physics layer used. Each frame the
//! physics integrator moves the character by gravity + input; this resolver then
//! sweeps the character's bounding box against the voxel grid per axis so it
//! cannot sink or walk through walls, with single-voxel step-up. It is a
//! hand-rolled *grid controller* layered on Avian's gravity — cheap (a hand
//! full of grid lookups per frame, independent of world size) and faithful to
//! the voxel genre (the user chose this over full-mesh colliders).
//!
//! The core is pure (`VoxelWorld` + arrays), so it is fully unit-testable.

use crate::world::VoxelWorld;

/// The character's half-extents (matches the `Collider::cuboid(0.9, 1.8, 0.9)`
/// the physics layer builds — `cuboid` takes FULL dims, so half = 0.45 / 0.9 /
/// 0.45).
const CHAR_HALF: [f32; 3] = [0.45, 0.9, 0.45];
/// Single-step height (one voxel) the character auto-ascends.
const STEP_UP: f32 = 1.0;
/// Boundary epsilon: a face exactly on a plane is inside the cell, not crossing.
const EPS: f32 = 1e-3;

/// Integrate the character against the voxel grid this frame and return the
/// corrected center and velocity.
pub fn integrate_against_grid(
    world: &VoxelWorld,
    center: [f32; 3],
    velocity: [f32; 3],
    dt: f32,
) -> ([f32; 3], [f32; 3]) {
    let mut center = center;
    let mut velocity = velocity;

    // Vertical first.
    let mut move_y = velocity[1] * dt;
    let grounded = resolve_axis(world, &mut center, 1, &mut move_y);
    velocity[1] = if grounded { 0.0 } else { move_y / dt.max(1e-4) };

    // Then X and Z, each with a step-up attempt.
    let mut move_x = velocity[0] * dt;
    if move_x.abs() > EPS {
        if resolve_axis_with_step(world, &mut center, 0, &mut move_x) {
            velocity[0] = 0.0;
        }
    }
    let mut move_z = velocity[2] * dt;
    if move_z.abs() > EPS {
        if resolve_axis_with_step(world, &mut center, 2, &mut move_z) {
            velocity[2] = 0.0;
        }
    }

    (center, velocity)
}

/// One-axis sweep with a single-voxel step-up on horizontal axes. Returns
/// `true` when the character was fully blocked (not steppable).
fn resolve_axis_with_step(world: &VoxelWorld, center: &mut [f32; 3], axis: usize, amount: &mut f32) -> bool {
    if resolve_axis(world, center, axis, amount) {
        // Try to step up over a ~1-voxel obstacle on a horizontal move.
        if axis != 1 && try_step_up(world, center, STEP_UP) {
            *amount = 0.0;
            return false; // stepped over
        }
        return true; // still blocked
    }
    false
}

/// One-axis resolve: move `axis` by `amount`; on a solid hit, snap to the
/// blocking face and zero the move. Returns `true` when a wall was hit.
///
/// * Descending: the top face = `idx + 1`; snap `center = f - half`.
/// * Ascending: the bottom face = `idx`; snap `center = f + half`.
fn resolve_axis(world: &VoxelWorld, center: &mut [f32; 3], axis: usize, amount: &mut f32) -> bool {
    let start = center[axis];
    let step = if *amount > 0.0 { 1.0 } else { -1.0 };
    center[axis] = start + *amount;

    if !aabb_touches_solid(world, *center) {
        return false;
    }

    let half = CHAR_HALF[axis];
    let mut face: Option<i32> = None;
    for cell in overlapping_cells(world, *center) {
        if world.contains(cell) {
            let idx = cell[axis];
            let f = if step < 0.0 { idx + 1 } else { idx };
            face = Some(match face {
                Some(cur) => if step < 0.0 { f.max(cur) } else { f.min(cur) },
                None => f,
            });
        }
    }
    if let Some(f) = face {
        center[axis] = if step < 0.0 {
            f as f32 - half - EPS
        } else {
            f as f32 + half + EPS
        };
        *amount = 0.0;
        return true;
    }
    center[axis] = start;
    false
}

/// Try to lift the character by `up_by` (one voxel) if that clears the
/// collision. Returns `true` when it cleared.
fn try_step_up(world: &VoxelWorld, center: &mut [f32; 3], up_by: f32) -> bool {
    let saved = center[1];
    center[1] = saved + up_by;
    let ok = !aabb_touches_solid(world, *center);
    if !ok {
        center[1] = saved;
    }
    ok
}

/// Whether the character's AABB (center ± `CHAR_HALF`) overlaps any solid voxel.
pub fn aabb_touches_solid(world: &VoxelWorld, c: [f32; 3]) -> bool {
    overlapping_cells(world, c).any(|cell| world.contains(cell))
}

/// Every voxel cell the character's AABB overlaps (low end inset by `EPS`).
fn overlapping_cells(_world: &VoxelWorld, c: [f32; 3]) -> impl Iterator<Item = [i32; 3]> + '_ {
    let (x0, x1) = cell_range(c[0], CHAR_HALF[0]);
    let (y0, y1) = cell_range(c[1], CHAR_HALF[1]);
    let (z0, z1) = cell_range(c[2], CHAR_HALF[2]);
    let out = (x0..=x1).flat_map(move |x| {
        (y0..=y1).flat_map(move |y| (z0..=z1).map(move |z| [x, y, z]))
    });
    out
}

/// A cell range with the low end inset by `EPS` (a face exactly on a plane is
/// inside the current cell, not the neighbour).
fn cell_range(center: f32, half: f32) -> (i32, i32) {
    let lo = (center - half + EPS).floor() as i32;
    let hi = (center + half - EPS).floor() as i32;
    (lo, hi)
}

// --------------------------------------------------------------------------
// Tests — no Bevy, no window.
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> VoxelWorld {
        VoxelWorld::new([32, 16, 32])
    }

    fn floor_slab(w: &mut VoxelWorld, half: i32, top_y: i32) {
        for x in -half..=half {
            for z in -half..=half {
                for y in -3..=top_y {
                    w.place([x, y, z], [0.0, 1.0, 0.0]);
                }
            }
        }
    }

    #[test]
    fn character_falls_onto_a_voxel_floor_and_rests() {
        let mut w = grid();
        floor_slab(&mut w, 4, 0);
        let y_rest = 1.0 - CHAR_HALF[1]; // top surface of the slab

        let mut pos = [0.0, 3.5, 0.0];
        let mut vel = [0.0, 0.0, 0.0];
        let g = 9.8 / 60.0;
        for _ in 0..90 {
            vel[1] -= g;
            let np = integrate_against_grid(&w, pos, vel, 1.0 / 60.0);
            pos = np.0;
            vel = np.1;
            if pos[1] <= y_rest + 0.001 {
                break;
            }
        }
        assert!((pos[1] - y_rest).abs() < 0.05, "rests at {:.3}, expected {y_rest:.3}", pos[1]);
        assert!(vel[1].abs() <= 0.01 + g, "falling-then-resting settles");
    }

    #[test]
    fn character_cannot_walk_through_a_wall() {
        let mut w = grid();
        for x in 0..2 {
            for y in 0..=2 {
                for z in -4..=4 {
                    w.place([x, y, z], [1.0, 0.0, 0.0]);
                }
            }
        }
        let mut pos = [-1.0, 1.0, 0.0];
        let mut vel = [10.0, 0.0, 0.0];
        for _ in 0..12 {
            let np = integrate_against_grid(&w, pos, vel, 1.0 / 60.0);
            pos = np.0;
            vel = np.1;
        }
        assert!(pos[0] <= CHAR_HALF[0] + 0.01, "x={:.3} must stop at the wall", pos[0]);
    }

    #[test]
    fn character_can_walk_onto_a_lower_platform() {
        let mut w = grid();
        for x in -2..=2 {
            for z in -2..=2 {
                for y in 0..=2 {
                    w.place([x, y, z], [0.5, 0.5, 0.5]);
                }
            }
        }
        let y_rest = 3.0 - CHAR_HALF[1]; // top surface of the platform

        let mut pos = [0.0, 5.0, 0.0];
        let mut vel = [0.0, 0.0, 0.0];
        let g = 9.8 / 60.0;
        for _ in 0..90 {
            vel[1] -= g;
            let np = integrate_against_grid(&w, pos, vel, 1.0 / 60.0);
            pos = np.0;
            vel = np.1;
            if pos[1] <= y_rest + 0.001 {
                break;
            }
        }
        assert!((pos[1] - y_rest).abs() < 0.06, "rests on platform top at {:.3}, expected {y_rest:.3}", pos[1]);
    }

    #[test]
    fn empty_world_is_pass_through() {
        let w = grid();
        let np = integrate_against_grid(&w, [0.0, 5.0, 0.0], [0.0, -1.0, 0.0], 1.0 / 60.0);
        assert!(np.0[1] < 5.0, "free-fall in an empty world");
    }

    #[test]
    fn aabb_excludes_exact_boundary_neighbor() {
        assert!(!aabb_touches_solid(&grid(), [0.0, CHAR_HALF[1], 0.0]));
        assert!(!aabb_touches_solid(&grid(), [0.0, CHAR_HALF[1] - 0.05, 0.0]));
    }
}
