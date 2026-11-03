//! Per-chunk greedy face-culling mesher for Bevy 0.20.
//!
//! Phase 2 of the chunked-storage work. Instead of spawning one `Mesh3d(Cuboid)`
//! per voxel (~9,700 draw calls for the Capitol), this builds one merged mesh
//! per **chunk × distinct color**, culling faces whose neighbouring voxel is
//! solid. A face is only emitted when the neighbour (possibly in an *adjacent*
//! chunk) is empty, so inter-chunk seams cull correctly. The result is a
//! handful of draw calls instead of thousands.
//!
//! Design choice: one mesh per distinct voxel color within a chunk, each with a
//! single-color `StandardMaterial`, rather than per-vertex color (which the
//! Bevy 0.20 StandardMaterial path does not toggle cleanly). For the stylized
//! Capitol — a few colors — this is ~16 meshes; still 3 orders of magnitude
//! fewer than per-voxel.

use std::collections::HashMap;

use crate::world::{ChunkPos, VoxelWorld};

/// One culled mesh group for a single chunk and a single color.
pub struct FaceData {
    /// The color this group is rendered with (RGB, 0..1).
    pub color: [f32; 3],
    /// Absolute world-space vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// Per-vertex UVs (0..1 per face).
    pub uvs: Vec<[f32; 2]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

impl FaceData {
    fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

// --- face geometry -------------------------------------------------------
//
// Unit cube from (0,0,0) to (1,1,1). Each face lists its outward normal and
// its 4 corner offsets (CCW when viewed from outside), with a (u,v) per corner.
// Two triangles per face: (0,1,2) and (0,2,3).

#[derive(Clone, Copy)]
struct FaceDef {
    /// Offset of the neighbouring voxel on this face.
    neighbor: [i32; 3],
    normal: [f32; 3],
    corners: [[f32; 3]; 4],
    uvs: [[f32; 2]; 4],
}

const FACES: [FaceDef; 6] = [
    // +X
    FaceDef {
        neighbor: [1, 0, 0],
        normal: [1.0, 0.0, 0.0],
        corners: [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
        uvs: [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ],
    },
    // -X
    FaceDef {
        neighbor: [-1, 0, 0],
        normal: [-1.0, 0.0, 0.0],
        corners: [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        uvs: [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ],
    },
    // +Y (top)
    FaceDef {
        neighbor: [0, 1, 0],
        normal: [0.0, 1.0, 0.0],
        corners: [
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
        uvs: [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ],
    },
    // -Y (bottom)
    FaceDef {
        neighbor: [0, -1, 0],
        normal: [0.0, -1.0, 0.0],
        corners: [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
        ],
        uvs: [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ],
    },
    // +Z
    FaceDef {
        neighbor: [0, 0, 1],
        normal: [0.0, 0.0, 1.0],
        corners: [
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ],
        uvs: [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ],
    },
    // -Z
    FaceDef {
        neighbor: [0, 0, -1],
        normal: [0.0, 0.0, -1.0],
        corners: [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        uvs: [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ],
    },
];

/// Quantize an RGB color to a `[u8; 3]` key (0–1 → 0–255). Floats are not
/// `Hash`/`Eq`, so we bucket by the rounded 8-bit color. The generated palette
/// uses a handful of distinct RGB values that survive this quantization exactly.
fn color_key(c: [f32; 3]) -> [u8; 3] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5).round() as u8,
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5).round() as u8,
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5).round() as u8,
    ]
}

/// Collect culled faces for a single chunk, grouped by color.
///
/// For every occupied voxel, each of the 6 faces is emitted **only if the
/// neighbouring voxel is empty** (looked up across chunk boundaries via
/// `world.get`). Groups are returned as [`FaceData`], one per distinct color.
pub fn collect_chunk_faces(world: &VoxelWorld, cp: &ChunkPos) -> Vec<FaceData> {
    // Index of each color key into `groups`.
    let mut key_to_index: HashMap<[u8; 3], usize> = HashMap::new();
    let mut groups: Vec<FaceData> = Vec::new();

    let chunk = match world.get_chunk(cp) {
        Some(c) => c,
        None => return Vec::new(),
    };

    for (local, lv) in chunk.iter_local() {
        // World position of the lower corner of this unit voxel.
        let gx = cp.0[0] * 32 + local[0] as i32;
        let gy = cp.0[1] * 32 + local[1] as i32;
        let gz = cp.0[2] * 32 + local[2] as i32;
        let origin = [gx as f32, gy as f32, gz as f32];

        for face in FACES {
            // A face is visible only when its neighbour voxel is empty.
            let n = [
                gx + face.neighbor[0],
                gy + face.neighbor[1],
                gz + face.neighbor[2],
            ];
            if world.get(n).is_some() {
                continue; // neighbour is solid: cull this face
            }

            let key = color_key(lv.color);
            let idx = *key_to_index
                .entry(key)
                .or_insert_with(|| {
                    let i = groups.len();
                    groups.push(FaceData {
                        color: lv.color,
                        positions: Vec::new(),
                        normals: Vec::new(),
                        uvs: Vec::new(),
                        indices: Vec::new(),
                    });
                    i
                });
            let group = &mut groups[idx];

            let base = group.positions.len() as u32;
            for i in 0..4 {
                let c = face.corners[i];
                group.positions.push([
                    origin[0] + c[0],
                    origin[1] + c[1],
                    origin[2] + c[2],
                ]);
                group.normals.push(face.normal);
                group.uvs.push(face.uvs[i]);
            }
            // Two triangles: 0,1,2 and 0,2,3.
            group
                .indices
                .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    groups.into_iter().filter(|g| !g.is_empty()).collect()
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_voxel_emits_all_six_faces() {
        let mut world = VoxelWorld::default();
        world.place([0, 0, 0], [1.0, 0.0, 0.0]);
        let groups = collect_chunk_faces(&world, &ChunkPos([0, 0, 0]));
        assert_eq!(groups.len(), 1, "one color group");
        // 6 faces × 4 verts each.
        assert_eq!(groups[0].positions.len(), 24);
        assert_eq!(groups[0].indices.len(), 36);
        assert_eq!(groups[0].normals.len(), 24);
        assert_eq!(groups[0].uvs.len(), 24);
    }

    #[test]
    fn two_adjacent_voxels_cull_the_shared_face() {
        let mut world = VoxelWorld::default();
        // Two voxels side by side: the +X face of the left one and the -X face
        // of the right one are both internal and must be culled.
        world.place([0, 0, 0], [1.0, 0.0, 0.0]);
        world.place([1, 0, 0], [1.0, 0.0, 0.0]);
        let groups = collect_chunk_faces(&world, &ChunkPos([0, 0, 0]));
        // 2 voxels unculled = 12 faces = 48 verts; the shared +X/-X pair (8
        // verts) is culled, leaving 10 faces = 40 verts.
        assert_eq!(groups[0].positions.len(), 40);
    }

    #[test]
    fn distinct_colors_form_distinct_groups() {
        let mut world = VoxelWorld::default();
        world.place([0, 0, 0], [1.0, 0.0, 0.0]);
        world.place([3, 0, 0], [0.0, 1.0, 0.0]);
        let groups = collect_chunk_faces(&world, &ChunkPos([0, 0, 0]));
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn cross_chunk_face_is_not_culled() {
        // A voxel at the +x boundary of chunk 0, with no voxel in chunk 1,
        // must still draw its +X face even though the neighbour is in another
        // chunk (world.get returns None across the boundary).
        let mut world = VoxelWorld::default();
        world.place([31, 0, 0], [1.0, 0.0, 0.0]);
        let groups = collect_chunk_faces(&world, &ChunkPos([0, 0, 0]));
        // Full 6 faces since neighbours are all empty.
        assert_eq!(groups[0].positions.len(), 24);
    }

    #[test]
    fn cross_chunk_neighbour_culls_boundary_face() {
        // Voxel at [31,0,0] (chunk 0) and [32,0,0] (chunk 1) are adjacent
        // across the chunk seam; each must cull the shared face.
        let mut world = VoxelWorld::default();
        world.place([31, 0, 0], [1.0, 0.0, 0.0]);
        world.place([32, 0, 0], [1.0, 0.0, 0.0]);
        let g0 = collect_chunk_faces(&world, &ChunkPos([0, 0, 0]));
        let g1 = collect_chunk_faces(&world, &ChunkPos([1, 0, 0]));
        // Chunk 0: its voxel culls +X (neighbour in chunk 1). 5 faces = 20.
        assert_eq!(g0[0].positions.len(), 20);
        // Chunk 1: its voxel culls -X (neighbour in chunk 0). 5 faces = 20.
        assert_eq!(g1[0].positions.len(), 20);
    }
}
