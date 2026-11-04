//! CPU path-tracing pass for the voxel scene.
//!
//! Bevy 0.20.1 ships no GPU path tracer, so this is a hand-rolled CPU "set of
//! path tracing": each frame the current voxel world is ray-marched (Amanatides–
//! Woo DDA) from a camera with a one-term sky-bounce global illumination
//! approximation (diffuse from a directional sun plus an ambient sky fill).
//! The result is an RGBA image, kept as a `bevy` `Image` asset for an in-scene
//! preview and dumped to a PPM file on the first frame.
//!
//! The core [`trace_frame`] is pure (no Bevy, no window), so it is fully
//! unit-testable. `bevy_image` is reached through the `bevy` re-export
//! (`bevy::image::Image`, `bevy::render::render_resource::…`), matching
//! `rendering.rs` — no extra direct dependency.

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::rendering::VoxelWorldResource;
use crate::world::VoxelWorld;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

/// A single trace request handed to the background worker thread. The world is a
/// cloned snapshot; the worker owns it for the duration of the trace. Cloning a
/// few-MB world on the main thread per request is cheap enough that it never
/// competes with the render frame — the heavy ray-march stays off-thread.
struct TraceJob {
    camera: PathCamera,
    world: VoxelWorld,
    width: u32,
    height: u32,
}

/// The sky color (missed primary rays + the ambient bounce term).
const SKY: [f32; 3] = [0.16, 0.30, 0.60];
/// A directional sun for the diffuse term.
const SUN_DIR: [f32; 3] = [0.4, 0.9, 0.3];
/// Ambient sky fill for indirect light.
const AMBIENT: f32 = 0.25;

// --------------------------------------------------------------------------
// Pure core (no Bevy)
// --------------------------------------------------------------------------

/// A camera for the path tracer: an eye position + a look-at target.
#[derive(Clone, Copy, Debug)]
pub struct PathCamera {
    pub pos: [f32; 3],
    pub target: [f32; 3],
}

impl Default for PathCamera {
    /// A default 3/4 high orbit looking down at the Capitol at the origin.
    fn default() -> Self {
        Self { pos: [60.0, 70.0, 60.0], target: [0.0, 10.0, 0.0] }
    }
}

/// March a ray through the voxel grid via the Amanatides–Woo DDA and return the
/// first solid voxel, or `None` when it escapes or misses the occupied scene.
/// Voxels are integer cubes. `max_steps` is a hard safety cap; the real work is
/// bounded by the [`ray_aabb_interval`] hit length, so sky rays (the common
/// case for a high camera) are rejected in O(1) instead of walking the cap.
fn march_voxels(world: &VoxelWorld, origin: [f32; 3], dir: [f32; 3], max_steps: i32, extent: Option<([i32; 3], [i32; 3])>) -> Option<[i32; 3]> {
    // Reject rays that never enter the occupied scene (sky). This is the single
    // biggest speed-up: sky pixels cost O(1) instead of `max_steps` lookups.
    let interval = match extent {
        Some((lo, hi)) => ray_aabb_interval(origin, dir, lo, hi)?,
        None => return None, // empty world: everything is sky
    };
    // `t_enter` may be slightly negative when the camera starts inside the scene;
    // the DDA walks forward from `origin`, so clamp the start there.
    let t_enter = interval.0.max(0.0);
    // Bound the loop to the in-scene interval rather than the large `max_steps`
    // cap, in units of `|dir|` (the DDA advances by that each iteration).
    let span = interval.1.max(t_enter) - t_enter;
    let capped = (1.0 + span * t_delta(dir)[0].min(t_delta(dir)[1]).min(t_delta(dir)[2]))
        .ceil() as i32;
    let max_steps = max_steps.min(capped);

    let sx = dir[0].signum();
    let sy = dir[1].signum();
    let sz = dir[2].signum();
    let tdx = t_delta(dir)[0];
    let tdy = t_delta(dir)[1];
    let tdz = t_delta(dir)[2];

    let mut ix = origin[0].floor() as i32;
    let mut iy = origin[1].floor() as i32;
    let mut iz = origin[2].floor() as i32;

    let mut tmx = distance_to_next(origin[0], sx, tdx);
    let mut tmy = distance_to_next(origin[1], sy, tdy);
    let mut tmz = distance_to_next(origin[2], sz, tdz);

    for _ in 0..max_steps {
        if world.contains([ix, iy, iz]) {
            return Some([ix, iy, iz]);
        }
        if tmx < tmy && tmx < tmz {
            tmx += tdx;
            ix += sx as i32;
        } else if tmy < tmz {
            tmy += tdy;
            iy += sy as i32;
        } else {
            tmz += tdz;
            iz += sz as i32;
        }
    }
    None
}

/// In units of `|dir|`, the distance along a ray from `origin` to the next
/// integer voxel boundary on this axis (0 ⇒ travel is infinite).
fn distance_to_next(origin: f32, step: f32, t_delta: f32) -> f32 {
    if step == 0.0 {
        return f32::INFINITY;
    }
    let next = if step > 0.0 { origin.floor() + 1.0 } else { origin.floor() };
    (next - origin) * t_delta.max(1e-30)
}

/// The per-voxel `t` increment `1/|dir.axis|` for each axis (∞ when the ray is
/// parallel to that axis). Used by the DDA and to bound its step count.
fn t_delta(dir: [f32; 3]) -> [f32; 3] {
    [1.0 / dir[0].abs().max(1e-30), 1.0 / dir[1].abs().max(1e-30), 1.0 / dir[2].abs().max(1e-30)]
}

/// Slab test: the continuous-space `t` interval over which the ray from
/// `origin` along `dir` (both normalized-ish) is inside the axis-aligned solid
/// box `[lo, hi+1]` (voxel-int corners `lo`, `hi`; a voxel `i` occupies `[i,
/// i+1]`). Returns `None` on a miss, which is the key early-out: sky rays
/// that never touch the occupied scene cost O(1).
fn ray_aabb_interval(origin: [f32; 3], dir: [f32; 3], lo: [i32; 3], hi: [i32; 3]) -> Option<(f32, f32)> {
    let lb = [lo[0] as f32, lo[1] as f32, lo[2] as f32];
    let hb = [hi[0] as f32 + 1.0, hi[1] as f32 + 1.0, hi[2] as f32 + 1.0];
    let mut t0 = 0.0_f32;       // nearest enter
    let mut t1 = f32::INFINITY; // nearest exit
    for a in 0..3 {
        let o = origin[a];
        let d = dir[a];
        if d.abs() < 1e-8 {
            // Ray parallel to this axis: it is inside the slab only if `o` is
            // within `lb..hb`, otherwise it misses.
            if o < lb[a] || o > hb[a] {
                return None;
            }
            continue;
        }
        let ta = (lb[a] - o) / d;
        let tb = (hb[a] - o) / d;
        // `ta`/`tb` may cross; normalize so `ta < tb`.
        let (ta, tb) = if ta < tb { (ta, tb) } else { (tb, ta) };
        t0 = t0.max(ta);
        t1 = t1.min(tb);
        if t0 > t1 {
            return None;
        }
    }
    Some((t0, t1))
}

/// Trace a primary ray to a color with one diffuse sun term + ambient sky.
/// The approximated surface normal is `-dir` (good enough for a CPU preview).
/// `max_steps` caps the DDA depth so escaping (sky) rays stop quickly instead
/// of running to a large fixed bound — this is what keeps the trace cheap.
fn trace_ray(world: &VoxelWorld, origin: [f32; 3], dir: [f32; 3], max_steps: i32, extent: Option<([i32; 3], [i32; 3])>) -> [f32; 3] {
    let hit = match march_voxels(world, origin, dir, max_steps, extent) {
        Some(h) => h,
        None => return SKY, // missed everything: sky
    };
    let col = match world.get(hit) {
        Some(v) => v.color,
        None => return SKY,
    };
    // Normal ≈ opposite the incident ray; diffuse from a directional sun plus
    // an ambient fill (a one-bounce global-illumination approximation).
    let n = normalize([-dir[0], -dir[1], -dir[2]]);
    let sun = normalize([SUN_DIR[0], SUN_DIR[1], SUN_DIR[2]]);
    let ndotl = dot(n, sun).max(0.0);
    let light = AMBIENT + (1.0 - AMBIENT) * ndotl;
    [col[0] * light, col[1] * light, col[2] * light]
}

/// The capped DDA depth. A miss only ever needs to travel to the far corner of
/// a small scene; 256 steps covers the ~±75-voxel Capitol with margin while
/// keeping the per-ray and per-frame cost flat. (Was 4096 — the source of a
/// periodic 16M-iteration burst every 12 frames that caused visible hitching.)
const MAX_MARCH: i32 = 256;

/// Trace a band of rows `[start_row, end_row)` of the camera image into the
/// reusable `pixels` buffer (`w*h*4` bytes, pre-allocated). This is the
/// progressive work unit: by tracing a few rows per frame instead of the whole
/// image at once, the CPU cost is spread flat across frames rather than bursting
/// on a single frame.
pub fn trace_band(
    world: &VoxelWorld,
    camera: &PathCamera,
    w: u32,
    h: u32,
    pixels: &mut [u8],
    start_row: u32,
    end_row: u32,
    max_steps: i32,
) {
    if w == 0 || h == 0 {
        return;
    }
    let fov = 0.9_f32;
    let tan_h = (fov * 0.5).tan();
    let aspect = w as f32 / h as f32;

    let fwd = normalize(sub(camera.target, camera.pos));
    let right = normalize(cross(fwd, [0.0, 1.0, 0.0]));
    let up = normalize(cross(right, fwd));
    // Occupied extent is per-image, not per-ray: compute it once here so the
    // per-ray DDA can reject sky in O(1) without re-walking the chunk map.
    let extent = world.occupied_extent();

    for py in start_row..end_row {
        if py >= h {
            break;
        }
        for px in 0..w {
            let nx = (2.0 * (px as f32 + 0.5) / w as f32 - 1.0) * aspect * tan_h;
            let ny = (1.0 - 2.0 * (py as f32 + 0.5) / h as f32) * tan_h;
            let dir = normalize([
                fwd[0] + right[0] * nx + up[0] * ny,
                fwd[1] + right[1] * nx + up[1] * ny,
                fwd[2] + right[2] * nx + up[2] * ny,
            ]);
            let rgb = trace_ray(world, camera.pos, dir, max_steps, extent);
            let i = ((py * w + px) * 4) as usize;
            pixels[i] = (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            pixels[i + 1] = (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8;
            pixels[i + 2] = (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8;
            pixels[i + 3] = 255;
        }
    }
}

/// Trace the whole voxel scene from a camera to an RGBA8 image of size `w*h`.
/// Returns `w*h*4` bytes (zero-length for `0` dimensions). Convenience wrapper
/// over [`trace_band`] for one-shot / test use.
pub fn trace_frame(world: &VoxelWorld, camera: &PathCamera, w: u32, h: u32) -> Vec<u8> {
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let mut pixels: Vec<u8> = vec![0u8; (w * h * 4) as usize];
    trace_band(world, camera, w, h, &mut pixels, 0, h, MAX_MARCH);
    pixels
}

/// Normalize — returns the zero vector on the zero input.
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len == 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Dot product of two 3-vectors.
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product of two 3-vectors.
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// Element-wise subtraction of two 3-vectors.
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Write an RGBA8 frame to a binary PPM (P6) file. Returns true on success.
pub fn dump_ppm(path: &std::path::Path, pixels: &[u8], w: u32, h: u32) -> bool {
    let mut f = match std::fs::File::create(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buf: Vec<u8> = Vec::with_capacity((w * h * 3) as usize);
    buf.extend_from_slice(b"P6\n");
    buf.extend_from_slice(format!("{w} {h}\n255\n").as_bytes());
    for p in pixels.chunks(4) {
        buf.push(p[0]);
        buf.push(p[1]);
        buf.push(p[2]);
    }
    matches!(std::io::Write::write_all(&mut f, &buf), Ok(_))
}

// --------------------------------------------------------------------------
// Bevy wiring: a background worker thread + a swap/request system
// --------------------------------------------------------------------------

/// The in-scene path-traced preview. The heavy ray-march runs on a spawned
/// [`std::thread`]; the main-thread [`render_path_trace`] system only swaps in
/// completed results and periodically kicks new requests. This keeps per-frame
/// work on the render thread to a cheap world-clone + channel poll — which is
/// what removes the periodic hitching (the work is off the main thread).
#[derive(Resource)]
pub struct PathTraceResource {
    pub camera: PathCamera,
    pub width: u32,
    pub height: u32,
    pub frame: u32,
    pub image: Handle<Image>,
    /// A first-frame PPM dump has already happened.
    pub dumped: bool,
    /// Whether the preview image has been added to the asset store yet.
    pub resolved: bool,
    /// Sender to the worker; `None` when the thread has stopped (after drop).
    /// `Sender` is `Send + Sync`, so this keeps the resource `Sync`.
    request_tx: Option<Sender<TraceJob>>,
    /// Results delivered by the worker. `std::sync::mpsc::Receiver` is `Send`
    /// but NOT `Sync`, so it is wrapped in a `Mutex` to keep the Bevy resource
    /// (which must be `Sync`) valid. Drained each frame in `drain_results`.
    result_rx: Mutex<Receiver<Vec<u8>>>,
    /// Min frames between trace requests (cadence; the trace itself is off-thread).
    trace_every: u32,
    /// Frames since the last request was issued.
    since_trace: u32,
    /// The world `revision` the in-flight / last-traced preview was built from.
    /// `render_path_trace` issues a new trace only when the live world's
    /// `revision` differs from this (i.e. the world actually changed), so a
    /// static world is never re-cloned — the whole point of the optimization.
    last_traced_revision: u64,
    /// The worker join handle; joined in `Drop` after the request channel
    /// closes, so the thread always exits cleanly at app shutdown.
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Default for PathTraceResource {
    /// An inactive resource with no worker thread, for tests/hand-rolled setup.
    /// [`PathTraceResource::spawn`] should be used to get a live worker.
    fn default() -> Self {
        Self {
            camera: PathCamera::default(),
            width: 64,
            height: 64,
            frame: 0,
            image: Handle::default(),
            dumped: false,
            resolved: false,
            request_tx: None,
            result_rx: Mutex::new(channel().1),
            trace_every: 30,
            since_trace: 0,
            last_traced_revision: 0,
            worker: None,
        }
    }
}

impl PathTraceResource {
    /// Build a live resource and spawn the background worker thread.
    pub fn spawn(width: u32, height: u32) -> Self {
        let (req_tx, req_rx) = channel();
        let (res_tx, res_rx) = channel();
        let worker = thread::spawn(move || trace_worker(req_rx, res_tx));
        Self {
            camera: PathCamera::default(),
            width,
            height,
            frame: 0,
            image: Handle::default(),
            dumped: false,
            resolved: false,
            request_tx: Some(req_tx),
            result_rx: Mutex::new(res_rx),
            trace_every: 30,
            since_trace: 0,
            last_traced_revision: 0,
            worker: Some(worker),
        }
    }

    /// Collect every completed-result pixel buffer currently sitting in the
    /// result queue, releasing the `Mutex` borrow and moving the data out. This
    /// is what lets the system read `frame.*` fields afterward without a
    /// disjoint-borrow fight through `ResMut`'s `Deref`.
    fn drain_results(&mut self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        if let Ok(guard) = self.result_rx.lock() {
            while let Ok(pixels) = guard.try_recv() {
                out.push(pixels);
            }
        }
        out
    }
}

/// Startup system that installs the live preview resource + worker thread.
/// Replaces `init_resource::<PathTraceResource>()` because the constructor
/// must spawn a thread (which `Default`/`init_resource` cannot do cleanly).
pub fn start_path_trace(world: &mut World) {
    let (width, height) = (64u32, 64u32);
    world.insert_resource(PathTraceResource::spawn(width, height));
}

/// The worker loop: pull trace requests and push results. It never blocks the
/// main thread. When the request channel is closed (the resource is dropped at
/// app exit) the blocking `recv()` returns `Err`, the loop ends, and the thread
/// exits — the handle is then joined in `Drop`.
fn trace_worker(
    req_rx: Receiver<TraceJob>,
    res_tx: Sender<Vec<u8>>,
) {
    while let Ok(job) = req_rx.recv() {
        let pixels = trace_frame(&job.world, &job.camera, job.width, job.height);
        if res_tx.send(pixels).is_err() {
            // Main thread dropped its receiver: stop.
            break;
        }
    }
}

/// Main-thread system: swap in any completed results, then issue the next
/// request on a coarse cadence. Per-frame work here is a world clone + a couple
/// of non-blocking `try_recv`/`send` calls — the trace cost is borne by the
/// worker thread, so frames stay smooth.
pub fn render_path_trace(
    mut frame: ResMut<PathTraceResource>,
    world_res: Res<VoxelWorldResource>,
    mut images: ResMut<Assets<Image>>,
) {
    frame.frame += 1;

    // 1. Swap in any completed results. A result may arrive between cadence
    //    ticks (the worker is async), so drain everything that is ready now into
    //    owned data, which releases the borrow before we touch `frame.*` again.
    let pending = frame.drain_results();
    for pixels in pending {
        let width = frame.width;
        let height = frame.height;
        let img = Image::new(
            Extent3d { width, height, depth_or_array_layers: 1 },
            TextureDimension::D2,
            pixels.clone(),
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        if frame.resolved {
            let _ = images.insert(frame.image.id(), img);
        } else {
            frame.image = images.add(img);
            frame.resolved = true;
            // One-time PPM snapshot of the first completed frame.
            let path = std::path::Path::new("path_traced_preview.ppm");
            if dump_ppm(&path, &pixels, width, height) {
                info!(
                    "path-trace preview dumped to path_traced_preview.ppm ({}x{})",
                    width, height
                );
                frame.dumped = true;
            }
        }
    }

    // 2. Issue a fresh trace ONLY when the world actually changed since the
    //    in-flight trace, and on a coarse cadence otherwise. A static world
    //    (the common case — the city rarely changes) is never re-cloned; the
    //    heavy clone + DDA stays off the render thread *and* off-cadence for a
    //    no-change frame. A single 64-bit `revision` read is the whole gate;
    //    the very first frame bypasses the cadence so the preview appears fast.
    let world_rev = world_res.world.revision();
    frame.since_trace += 1;
    let due = frame.since_trace >= frame.trace_every || !frame.resolved;
    let changed = world_rev != frame.last_traced_revision;
    if due && changed {
        frame.since_trace = 0;
        let camera = frame.camera;
        let width = frame.width;
        let height = frame.height;
        let world = world_res.world.clone();
        // Record the revision we're about to ask about. Set it before borrowing
        // `request_tx` so the mutable and immutable borrows don't overlap. If
        // the world changes again before this result lands, the next eligible
        // frame re-traces the newer revision; a static world keeps its revision,
        // so it is never re-cloned again.
        frame.last_traced_revision = world_rev;
        if let Some(tx) = &frame.request_tx {
            let job = TraceJob { camera, world, width, height };
            // `send` blocks only if the worker is mid-trace; it drains promptly,
            // so this is effectively non-blocking in practice.
            let _ = tx.send(job);
        }
    }
}

impl Drop for PathTraceResource {
    fn drop(&mut self) {
        // Dropping the request sender closes the channel; the worker's blocking
        // `recv()` returns `Err` and the loop ends. Then join so the thread
        // actually exits at app shutdown instead of lingering.
        self.request_tx = None;
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

// --------------------------------------------------------------------------
// Tests (pure — no Bevy, no window)
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn one_block() -> VoxelWorld {
        let mut w = VoxelWorld::new([16, 32, 16]);
        w.place([0, 0, 0], [0.90, 0.45, 0.20]);
        w
    }

    #[test]
    fn empty_world_renders_uniform_sky() {
        let w = VoxelWorld::new([16, 32, 16]);
        let frame = trace_frame(&w, &PathCamera::default(), 4, 4);
        assert_eq!(frame.len(), 4 * 4 * 4);
        // Every pixel should be the same sky color (all rays miss).
        for i in (0..frame.len()).step_by(4) {
            assert_eq!((frame[i], frame[i + 1], frame[i + 2]), (frame[0], frame[1], frame[2]), "all pixels must be the uniform sky color");
        }
    }

    #[test]
    fn a_single_block_hits_and_color() {
        let w = one_block();
        let cam = PathCamera { pos: [0.0, 8.0, 0.0], target: [0.0, 0.0, 0.0] };
        let frame = trace_frame(&w, &cam, 3, 3);
        // The center pixel looks down at the block: it should read the block's
        // color (a warm red), not the blue sky.
        let ci = 4 * 4; // row 1, col 1 of a 3x3 grid (center)
        let (r, g) = (frame[ci], frame[ci + 1]);
        assert!(r > g, "block color should be red-ish (r {r} > g {g})");
        assert!(r > 80, "center should read a lit block, got r={r}");
    }

    #[test]
    fn dump_ppm_writes_a_p6_header() {
        let pixels = vec![255u8, 128, 0, 255, 0, 128, 255, 255]; // 2x2 RGBA
        let path = std::env::temp_dir().join("warren_path_trace_test.ppm");
        assert!(dump_ppm(&path, &pixels, 2, 2));
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"P6\n2 2\n255\n"));
        assert_eq!(bytes.len(), 11 + 6, "11-byte header + 6 RGB bytes (2 pixels x 3)");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn trace_band_composes_into_a_full_frame() {
        // Tracing the image row-band by row (the progressive sweep) must equal
        // tracing the whole image at once: this is the invariant that makes the
        // per-frame band tracing produce the same picture without a burst.
        let w = one_block();
        let cam = PathCamera { pos: [0.0, 8.0, 0.0], target: [0.0, 0.0, 0.0] };
        let (width, height) = (8u32, 8u32);
        let full = trace_frame(&w, &cam, width, height);

        let mut band = vec![0u8; (width * height * 4) as usize];
        // Two non-overlapping bands covering all rows.
        trace_band(&w, &cam, width, height, &mut band, 0, 4, MAX_MARCH);
        trace_band(&w, &cam, width, height, &mut band, 4, 8, MAX_MARCH);
        assert_eq!(band, full, "composed bands must equal a single full trace");
    }

    #[test]
    fn empty_dimensions_trace_nothing() {
        let w = VoxelWorld::new([16, 32, 16]);
        let mut pixels = vec![1u8; 16];
        trace_band(&w, &PathCamera::default(), 0, 4, &mut pixels, 0, 4, MAX_MARCH);
        // Untouched: 0 width writes nothing.
        assert_eq!(pixels, vec![1u8; 16]);
    }

    #[test]
    fn worker_roundtrips_a_traced_frame() {
        // The heavy trace runs off the main thread: verify the worker pipeline
        // (spawn -> request -> result -> join) delivers a full RGBA frame and
        // shuts down cleanly when the request channel closes.
        let (req_tx, req_rx) = std::sync::mpsc::channel();
        let (res_tx, res_rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || trace_worker(req_rx, res_tx));

        let world = crate::world::VoxelWorld::new([16, 32, 16]);
        let job = TraceJob {
            camera: PathCamera::default(),
            world: world.clone(),
            width: 16,
            height: 16,
        };
        req_tx.send(job).unwrap();
        let out = res_rx.recv().expect("worker should deliver a frame");
        assert_eq!(out.len(), 16 * 16 * 4, "worker must return a full RGBA frame");

        // Drop the request sender -> the worker's blocking recv() ends -> join.
        drop(req_tx);
        worker.join().expect("worker thread should exit cleanly");
    }
}
