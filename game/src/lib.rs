//! Warren Game — Voxel World Data Model
//!
//! Contains the core voxel world data structures and brush state that will
//! be used by the Bevy rendering engine in later phases.
//!
//! In MVP the voxel world is a sparse HashMap. This is sufficient for
//! hundreds to a few thousand voxels and avoids the complexity of
//! chunk-based storage.

use std::collections::HashMap;
use warren_protocol::{PeerInfo, Voxel, VoxelEdit, AssetType};

// --------------------------------------------------------------------------
// Voxel World
// --------------------------------------------------------------------------

/// A sparse voxel world: a HashMap of position → Voxel.
/// Only occupied positions are stored, so an empty world is O(1) memory.
#[derive(Debug, Clone)]
pub struct VoxelWorld {
    voxels: HashMap<[i32; 3], Voxel>,
    /// The maximum extent of the world in each axis (centered at origin).
    /// For MVP: 64 × 64 × 64 = 262,144 max voxels.
    bounds: [i32; 3],
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self {
            voxels: HashMap::new(),
            bounds: [64, 64, 64],
        }
    }
}

impl VoxelWorld {
    /// Create a new world with given half-extents (e.g. `[64, 64, 64]` for a 127³ cube).
    pub fn new(bounds: [i32; 3]) -> Self {
        Self {
            voxels: HashMap::new(),
            bounds,
        }
    }

    /// Place or replace a voxel at (x, y, z) with the given color.
    pub fn place(&mut self, position: [i32; 3], color: [f32; 3]) {
        self.voxels.insert(
            position,
            Voxel {
                position,
                color,
                asset_type: AssetType::Voxel,
            },
        );
    }

    /// Erase the voxel at (x, y, z).
    pub fn erase(&mut self, position: [i32; 3]) {
        self.voxels.remove(&position);
    }

    /// Apply a VoxelEdit delta (either place or erase).
    pub fn apply_edit(&mut self, edit: &VoxelEdit) {
        if edit.erase {
            self.erase(edit.position);
        } else {
            self.place(edit.position, edit.color);
        }
    }

    /// Get a voxel by position.
    pub fn get(&self, position: [i32; 3]) -> Option<&Voxel> {
        self.voxels.get(&position)
    }

    /// The world half-extents.
    pub fn bounds(&self) -> [i32; 3] {
        self.bounds
    }

    /// Number of voxels in the world.
    pub fn voxel_count(&self) -> usize {
        self.voxels.len()
    }

    /// Whether a position is within the world bounds.
    pub fn in_bounds(&self, position: [i32; 3]) -> bool {
        position[0].abs() < self.bounds[0]
            && position[1].abs() < self.bounds[1]
            && position[2].abs() < self.bounds[2]
    }

    /// Iterate over all voxels.
    pub fn iter(&self) -> impl Iterator<Item = &Voxel> {
        self.voxels.values()
    }

    /// Get a snapshot of all voxels as a Vec (for server sync).
    pub fn snapshot(&self) -> Vec<Voxel> {
        self.voxels.values().cloned().collect()
    }
}

// --------------------------------------------------------------------------
// Roster
// --------------------------------------------------------------------------

/// The current roster of connected peers.
#[derive(Debug, Clone, Default)]
pub struct Roster {
    pub peers: Vec<PeerInfo>,
}

impl Roster {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }

    pub fn upsert(&mut self, peer: PeerInfo) {
        if let Some(existing) = self.peers.iter_mut().find(|p| p.id == peer.id) {
            *existing = peer;
        } else {
            self.peers.push(peer);
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.peers.retain(|p| p.id != id);
    }
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_and_count() {
        let mut world = VoxelWorld::default();
        world.place([0, 0, 0], [1.0, 0.0, 0.0]);
        world.place([1, 0, 0], [0.0, 1.0, 0.0]);
        assert_eq!(world.voxel_count(), 2);
    }

    #[test]
    fn test_erase() {
        let mut world = VoxelWorld::default();
        world.place([5, 5, 5], [1.0, 1.0, 1.0]);
        world.erase([5, 5, 5]);
        assert_eq!(world.voxel_count(), 0);
    }

    #[test]
    fn test_in_bounds() {
        let world = VoxelWorld::new([16, 16, 16]);
        assert!(world.in_bounds([0, 0, 0]));
        assert!(world.in_bounds([15, 15, 15]));
        assert!(!world.in_bounds([16, 0, 0]));
        assert!(!world.in_bounds([17, 0, 0]));
    }

    #[test]
    fn test_apply_edit_place() {
        let mut world = VoxelWorld::default();
        let edit = VoxelEdit {
            position: [0, 1, 2],
            color: [0.5, 0.5, 1.0],
            erase: false,
        };
        world.apply_edit(&edit);
        assert_eq!(world.voxel_count(), 1);
        let v = world.get([0, 1, 2]).unwrap();
        assert_eq!(v.color, [0.5, 0.5, 1.0]);
    }

    #[test]
    fn test_apply_edit_erase() {
        let mut world = VoxelWorld::default();
        world.place([1, 1, 1], [1.0, 1.0, 1.0]);
        let edit = VoxelEdit {
            position: [1, 1, 1],
            color: [0.0, 0.0, 0.0],
            erase: true,
        };
        world.apply_edit(&edit);
        assert_eq!(world.voxel_count(), 0);
    }

    #[test]
    fn test_roster() {
        let mut roster = Roster::new();
        roster.upsert(PeerInfo {
            id: "c1".to_string(),
            name: "Alice".to_string(),
            color_rgb: [1.0, 0.0, 0.0],
            last_active_ms: 0,
        });
        roster.upsert(PeerInfo {
            id: "c2".to_string(),
            name: "Bob".to_string(),
            color_rgb: [0.0, 1.0, 0.0],
            last_active_ms: 0,
        });
        assert_eq!(roster.peers.len(), 2);
        roster.remove("c1");
        assert_eq!(roster.peers.len(), 1);
        assert!(roster.peers.iter().all(|p| p.id == "c2"));
    }

    #[test]
    fn test_snapshot() {
        let mut world = VoxelWorld::default();
        world.place([0, 0, 0], [1.0, 0.0, 0.0]);
        world.place([1, 1, 1], [0.0, 1.0, 0.0]);
        let snapshot = world.snapshot();
        assert_eq!(snapshot.len(), 2);
    }
}
