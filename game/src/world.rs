//! Voxel world data model and roster.
//!
//! Phase 1 of chunked storage: the world is a *sparse map of chunks*. Each
//! chunk is a 32×32×32 cube (one city-block) of voxels, stored only when it
//! contains at least one occupied cell. This bounds every operation to a single
//! 32³ cube and is the foundation for per-chunk mesh generation and
//! per-chunk network streaming (Phase 2).
//!
//! The public API (`place`/`erase`/`get`/… /`iter`/`snapshot`) is unchanged
//! from the flat-map version, so callers do not need to change.

use std::collections::{BTreeMap, HashMap, HashSet};
use warren_protocol::{PeerInfo, Voxel, VoxelEdit, AssetType};

/// Chunk edge length in voxels (32³ = 32,768 cells ≈ one city block).
pub const CHUNK: i32 = 32;
const CHUNK2: usize = CHUNK as usize * CHUNK as usize; // 1024

// --------------------------------------------------------------------------
// Chunk storage
// --------------------------------------------------------------------------

/// Origin (minimum corner) of a chunk in world space. Always a multiple of 32.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos(pub [i32; 3]);

/// A voxel stored inside a chunk. No world position — it is derived from the
/// chunk origin + local offset, saving 12 bytes per voxel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalVoxel {
    pub color: [f32; 3],
    pub asset_type: AssetType,
}

/// A single chunk: a sparse map of local flat index → voxel.
#[derive(Debug, Clone, Default)]
pub struct Chunk {
    cells: BTreeMap<usize, LocalVoxel>,
}

impl Chunk {
    /// Encode a local (0..31)³ offset into a flat index.
    fn key(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK as usize + z * CHUNK2
    }

    fn set(&mut self, x: usize, y: usize, z: usize, v: LocalVoxel) {
        self.cells.insert(Self::key(x, y, z), v);
    }

    fn get(&self, x: usize, y: usize, z: usize) -> Option<&LocalVoxel> {
        self.cells.get(&Self::key(x, y, z))
    }

    fn remove(&mut self, x: usize, y: usize, z: usize) -> bool {
        self.cells.remove(&Self::key(x, y, z)).is_some()
    }

    fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    fn count(&self) -> usize {
        self.cells.len()
    }

    /// Yield each occupied cell as its local (x,y,z) offset + the voxel.
    pub fn iter_local(&self) -> impl Iterator<Item = ([usize; 3], &LocalVoxel)> {
        self.cells.iter().map(|(k, lv)| {
            let z = k / CHUNK2;
            let y = (k % CHUNK2) / CHUNK as usize;
            let x = k % CHUNK as usize;
            ([x, y, z], lv)
        })
    }
}

// --------------------------------------------------------------------------
// Voxel World
// --------------------------------------------------------------------------

/// A sparse, chunked voxel world.
///
/// World positions are split into 32³ chunks; only non-empty chunks are stored,
/// so an empty world is O(1) and a large neighborhood stays compact.
#[derive(Debug, Clone, Default)]
pub struct VoxelWorld {
    /// Occupied chunks, keyed by chunk origin. Defaulted via `#[derive(Default)]`
    /// and `new()`.
    chunks: HashMap<ChunkPos, Chunk>,
    /// The maximum half-extent of the world in each axis (centered at origin).
    /// Used by `in_bounds`.
    bounds: [i32; 3],
    /// Chunks whose geometry changed since the last mesh sync. A voxel edit
    /// dirties its own chunk plus the up to 6 face-neighbour chunks, because a
    /// face on a chunk seam belongs to both chunks.
    dirty: HashSet<ChunkPos>,
}

impl VoxelWorld {
    /// Create a new world with given half-extents (e.g. `[64, 128, 64]`).
    pub fn new(bounds: [i32; 3]) -> Self {
        Self {
            chunks: HashMap::new(),
            bounds,
            dirty: HashSet::new(),
        }
    }

    /// The world half-extents.
    pub fn bounds(&self) -> [i32; 3] {
        self.bounds
    }

    /// Number of occupied chunks.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    // --- coordinate math -------------------------------------------------

    fn chunk_of(p: [i32; 3]) -> ChunkPos {
        ChunkPos([
            p[0].div_euclid(CHUNK),
            p[1].div_euclid(CHUNK),
            p[2].div_euclid(CHUNK),
        ])
    }

    /// Mark the chunk containing `position` — plus any chunk it shares a face
    /// boundary with — as needing a mesh rebuild. A seam face belongs to both
    /// chunks, so editing a boundary voxel dirties both sides.
    fn mark_dirty(&mut self, position: [i32; 3]) {
        let c = Self::chunk_of(position);
        let l = Self::local_of(position);
        self.dirty.insert(c);
        for axis in 0..3usize {
            if l[axis] == 0 {
                let mut neg = c.0;
                neg[axis] -= 1;
                self.dirty.insert(ChunkPos(neg));
            }
            if l[axis] == CHUNK as usize - 1 {
                let mut pos = c.0;
                pos[axis] += 1;
                self.dirty.insert(ChunkPos(pos));
            }
        }
    }

    /// Take (and clear) the set of chunks that need rebuilding. Returns an empty
    /// set when nothing changed.
    pub fn take_dirty(&mut self) -> HashSet<ChunkPos> {
        std::mem::take(&mut self.dirty)
    }

    /// Mark every chunk currently in the world as dirty (used on full
    /// re-generation, e.g. loading a neighborhood at startup).
    pub fn dirty_all(&mut self) {
        self.dirty = self.chunks.keys().copied().collect();
    }

    fn local_of(p: [i32; 3]) -> [usize; 3] {
        [
            p[0].rem_euclid(CHUNK) as usize,
            p[1].rem_euclid(CHUNK) as usize,
            p[2].rem_euclid(CHUNK) as usize,
        ]
    }

    // --- mutation --------------------------------------------------------

    /// Place or replace a voxel at `position` with the given RGB color.
    pub fn place(&mut self, position: [i32; 3], color: [f32; 3]) {
        let c = Self::chunk_of(position);
        let l = Self::local_of(position);
        let chunk = self.chunks.entry(c).or_default();
        chunk.set(l[0], l[1], l[2], LocalVoxel { color, asset_type: AssetType::Voxel });
        self.mark_dirty(position);
    }

    /// Erase the voxel at `position`, dropping the chunk if it becomes empty.
    pub fn erase(&mut self, position: [i32; 3]) {
        let c = Self::chunk_of(position);
        let l = Self::local_of(position);
        if let Some(chunk) = self.chunks.get_mut(&c) {
            chunk.remove(l[0], l[1], l[2]);
            if chunk.is_empty() {
                self.chunks.remove(&c);
            }
        }
        self.mark_dirty(position);
    }

    /// Apply a VoxelEdit delta (either place or erase).
    pub fn apply_edit(&mut self, edit: &VoxelEdit) {
        if edit.erase {
            self.erase(edit.position);
        } else {
            self.place(edit.position, edit.color);
        }
    }

    // --- queries ---------------------------------------------------------

    /// Get a voxel by world position, reconstructing its full `Voxel`.
    pub fn get(&self, position: [i32; 3]) -> Option<Voxel> {
        let c = Self::chunk_of(position);
        let l = Self::local_of(position);
        self.chunks
            .get(&c)?
            .get(l[0], l[1], l[2])
            .map(|lv| Voxel {
                position,
                color: lv.color,
                asset_type: lv.asset_type,
            })
    }

    /// Whether a voxel exists at `position`.
    pub fn contains(&self, position: [i32; 3]) -> bool {
        self.get(position).is_some()
    }

    /// Get a live reference to the chunk at `cp` (if present).
    /// Used by the mesher to enumerate a chunk's voxels for face culling.
    pub fn get_chunk(&self, cp: &ChunkPos) -> Option<&Chunk> {
        self.chunks.get(cp)
    }

    /// Whether the chunk at `cp` is currently stored.
    pub fn has_chunk(&self, cp: &ChunkPos) -> bool {
        self.chunks.contains_key(cp)
    }

    /// Whether a position is within the world half-extents.
    pub fn in_bounds(&self, position: [i32; 3]) -> bool {
        position[0].abs() < self.bounds[0]
            && position[1].abs() < self.bounds[1]
            && position[2].abs() < self.bounds[2]
    }

    /// Total number of occupied voxels across all chunks.
    pub fn voxel_count(&self) -> usize {
        self.chunks.values().map(|c| c.count()).sum()
    }

    /// Iterate every occupied voxel (owned, with reconstructed world position).
    ///
    /// Reconstructs `Voxel.position` from the chunk origin + local offset, so
    /// the flat `snapshot()` / render path can keep using the same shape.
    pub fn iter(&self) -> impl Iterator<Item = Voxel> + '_ {
        self.chunks.iter().flat_map(move |(cp, chunk)| {
            chunk.iter_local().map(move |(local, lv)| Voxel {
                position: [
                    cp.0[0] * CHUNK + local[0] as i32,
                    cp.0[1] * CHUNK + local[1] as i32,
                    cp.0[2] * CHUNK + local[2] as i32,
                ],
                color: lv.color,
                asset_type: lv.asset_type,
            })
        })
    }

    /// Get a snapshot of all voxels (for server sync).
    pub fn snapshot(&self) -> Vec<Voxel> {
        self.iter().collect()
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
    fn test_chunk_local_roundtrip() {
        // Every local offset in the negative and positive ranges must map back
        // to its original world position via chunk*32 + local.
        for p in [[0,0,0], [31,31,31], [32,0,0], [33,0,0],
                  [-1,0,0], [-32,0,0], [-33,0,0], [100, 200, -77], [9,9,9]] {
            let c = VoxelWorld::chunk_of(p);
            let l = VoxelWorld::local_of(p);
            assert!((0..3).all(|i| l[i] < 32), "local must be 0..31 for {p:?}");
            let back = [
                c.0[0] * CHUNK + l[0] as i32,
                c.0[1] * CHUNK + l[1] as i32,
                c.0[2] * CHUNK + l[2] as i32,
            ];
            assert_eq!(back, p, "roundtrip failed for {p:?}");
        }
    }

    #[test]
    fn test_place_and_count() {
        let mut world: VoxelWorld = VoxelWorld::default();
        world.place([0, 0, 0], [1.0, 0.0, 0.0]);
        world.place([1, 0, 0], [0.0, 1.0, 0.0]);
        assert_eq!(world.voxel_count(), 2);
        assert_eq!(world.chunk_count(), 1); // both in the same chunk
    }

    #[test]
    fn test_place_get_with_negative_coords() {
        let mut world: VoxelWorld = VoxelWorld::default();
        world.place([-129, -1, -32], [0.5, 0.5, 0.5]);
        let v = world.get([-129, -1, -32]).unwrap();
        assert_eq!(v.position, [-129, -1, -32]);
        // -129 / 32 = -4.03125 -> div_euclid = -5, rem = 31
        assert_eq!(VoxelWorld::chunk_of([-129, -1, -32]), ChunkPos([-5, -1, -1]));
    }

    #[test]
    fn test_voxels_in_different_chunks() {
        let mut world: VoxelWorld = VoxelWorld::default();
        world.place([0, 0, 0], [1.0, 0.0, 0.0]);
        world.place([64, 0, 0], [0.0, 1.0, 0.0]); // 2 chunks apart
        assert_eq!(world.voxel_count(), 2);
        assert_eq!(world.chunk_count(), 2);
    }

    #[test]
    fn test_erase_drops_empty_chunk() {
        let mut world: VoxelWorld = VoxelWorld::default();
        world.place([5, 5, 5], [1.0, 0.0, 0.0]);
        assert_eq!(world.chunk_count(), 1);
        world.erase([5, 5, 5]);
        assert_eq!(world.voxel_count(), 0);
        assert_eq!(world.chunk_count(), 0);
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
        let mut world: VoxelWorld = VoxelWorld::default();
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
        let mut world: VoxelWorld = VoxelWorld::default();
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
    fn test_iter_and_snapshot() {
        let mut world: VoxelWorld = VoxelWorld::default();
        world.place([0, 0, 0], [1.0, 0.0, 0.0]);
        world.place([50, 10, 10], [0.0, 1.0, 0.0]);
        world.place([0, 40, 0], [0.0, 0.0, 1.0]);
        let snap: Vec<Voxel> = world.iter().collect();
        assert_eq!(snap.len(), 3);
        // Each reconstructed position must be correct.
        assert!(snap.iter().any(|v| v.position == [50, 10, 10]));
        assert!(snap.iter().any(|v| v.position == [0, 40, 0]));
        assert_eq!(world.snapshot().len(), 3);
    }

    #[test]
    fn test_contains() {
        let mut world: VoxelWorld = VoxelWorld::default();
        assert!(!world.contains([1, 1, 1]));
        world.place([3, 0, 5], [1.0, 0.5, 0.0]);
        assert!(world.contains([3, 0, 5]));
        assert!(!world.contains([99, 0, 99]));
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
}
