## 2026-11-03
* **Update**: Updated concept `rendering/weft-renderscene.md` (Added verified Phase 2 per-chunk face-culling mesh decisions: Mesh::new(TriangleList).with_inserted_* builder, VertexFormat per attribute, Indices::U32, missing StandardMaterial vertex-color toggle so one mesh is used per distinct color, and dirty-chunk tracking pattern).
* **Update**: Linked `rendering/weft-renderscene.md` to `components-game.md` (Concrete implementation of the playable character entity and environment rendering described by components-game).
* **Update**: Linked `rendering/weft-renderscene.md` to `weft-analysis.md` (Corrects and extends the weft-analysis Bevy 0.20 translation table with actual verified API (base_color_texture, emissive LinearRgba, single-plane textured grid, auto-walk animation)).
* **Creation**: Documented concept `rendering/weft-renderscene.md` (Warren-Game Bevy 0.20 Rendering & Animation Decisions).
* **Update**: Linked `meshem-analysis.md` to `components-editor.md` (bevy_meshem is the rendering layer for the editor's voxel data model. The Mantine UI triggers voxel edits that feed into meshem's VoxelChange journal and update_mesh().).
* **Update**: Linked `meshem-analysis.md` to `texture-loader-crates.md` (bevy_meshem generates mesh geometry from a voxel grid; texture-loader-crates handles the texture atlas and model data that meshem applies as vertex attributes. Together, they form the full voxel rendering pipeline.).
* **Creation**: Documented concept `meshem-analysis.md` (bevy_meshem — Voxel Grid to Bevy Mesh Generation).

## 2026-11-02
* **Update**: Linked `big-space-analysis.md` to `bevy-water-analysis.md` (Both address world-scale considerations: big_space handles coordinate precision for large worlds; bevy_water handles tileable rendering for large water surfaces. Complementary.).
* **Update**: Linked `bevy-settings-example.md` to `bevy-plugin-example.md` (SettingsPlugin is a Plugin that manages persistent Resources via SettingsGroup. Shows the Plugin trait pattern with config via constructor args.).
* **Update**: Linked `bevy-logs-example.md` to `bevy-plugin-example.md` (Bevy's LogPlugin is itself a Plugin registered via add_plugins(). Demonstrates the Plugin registration pattern in production Bevy code.).
* **Update**: Linked `bevy-console-analysis.md` to `bevy-logs-example.md` (bevy_console can capture Bevy's log! / info! / warn! output and display it in the in-game console.).
* **Update**: Linked `bevy-fps-controller-analysis.md` to `rapier-analysis.md` (bevy_fps_controller also supports Rapier via --features rapier (uses bevy-rapier wrapper, not Rapier directly).).
* **Update**: Linked `bevy-fps-controller-analysis.md` to `avian-analysis.md` (bevy_fps_controller supports Avian via --features avian. Uses Avian for character physics in FPS movement.).
* **Update**: Linked `avian-analysis.md` to `rapier-analysis.md` (Avian is the Bevy-native integration layer for Rapier. Uses Parry (Rapier's collision geometry crate) as its default collision backend.).
* **Creation**: Documented concept `bevy-logs-example.md` (Bevy Logs — LogPlugin, Log Levels, and once! Spam Prevention).
* **Creation**: Documented concept `bevy-plugin-example.md` (Bevy Plugin — Custom Plugin Registration Pattern).
* **Creation**: Documented concept `bevy-settings-example.md` (Bevy Settings — SettingsGroup, SaveSettingsDeferred, and Persistent Config).
* **Creation**: Documented concept `bevy-fps-controller-analysis.md` (bevy_fps_controller — Source-Engine-Inspired FPS Movement for Bevy).
* **Creation**: Documented concept `rapier-analysis.md` (Rapier — Multi-Platform Physics Engine (Deno/Rust)).
* **Creation**: Documented concept `bevy-console-analysis.md` (bevy_console — Half-Life-Inspired Console for Bevy).
* **Creation**: Documented concept `bevy-animation-graph-analysis.md` (bevy_animation_graph — Animation Graphs and State Machines for Bevy).
* **Creation**: Documented concept `bevy-hanabi-analysis.md` (bevy_hanabi — GPU Particle System Plugin for Bevy).
* **Creation**: Documented concept `bevy-third-person-camera-analysis.md` (bevy_third_person_camera — Configurable Third-Person Camera for Bevy).
* **Creation**: Documented concept `avian-analysis.md` (Avian — ECS-Driven 2D/3D Physics Engine for Bevy).

## 2026-11-01
* **Update**: Updated concept `foundation-decision.md`.
* **Update**: Updated concept `components-client.md`.
* **Update**: Updated concept `components-editor.md`.
* **Update**: Updated concept `components-server.md`.
* **Update**: Updated concept `components-game.md`.
* **Update**: Updated concept `collaboration-architecture.md`.
* **Creation**: Documented concept `mvp-plan.md` (MVP Plan: Collaborative Voxel Asset Editor).
* **Creation**: Documented concept `weft-analysis.md` (Weft Voxel Engine Texture Atlas & Resource Pack Analysis).
* **Creation**: Documented concept `pumpkin-analysis.md` (Pumpkin Minecraft Server Analysis).
* **Creation**: Documented concept `texture-loader-crates.md` (Texture Loader Crate Analysis).
* **Creation**: Documented concept `texture-loader-architecture.md` (Texture Loading Architecture for warren-game).
* **Creation**: Documented concept `aeronet-analysis.md` (aeronet I/O foundation analysis).
* **Creation**: Documented concept `bevy-replicon-analysis.md` (bevy_replicon replication core analysis).
* **Creation**: Documented concept `lightyear-analysis.md` (lightyear game framework analysis).
* **Creation**: Documented concept `matchbox-analysis.md` (matchbox P2P WebRTC analysis and Bevy 0.17 incompatibility flag).
* **Creation**: Documented concept `networking-comparison.md` (cross-crate networking comparison and warren-game recommendation).
* **Creation**: Initialized OKF v0.2 knowledge bundle.
