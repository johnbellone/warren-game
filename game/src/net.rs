//! WebSocket client layer for collaborative voxel editing.
//!
//! The WebSocket connection runs on a dedicated background thread.
//! Messaging with the Bevy main thread uses:
//!   - `std::sync::mpsc::Sender<String>` for Bevy → network (outgoing)
//!   - `Arc<Mutex<VecDeque<String>>>` for network → Bevy (incoming inbox)

use std::collections::VecDeque;
use std::sync::mpsc::{self, Sender as MpscSender};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::rendering::VoxelWorldResource;
use warren_protocol::{ClientMsg, PeerInfo, ServerMsg, VoxelEdit};

// --------------------------------------------------------------------------
// Resources
// --------------------------------------------------------------------------

/// Shared inbox: messages pushed by the network thread, drained by Bevy.
type SharedInbox = Arc<Mutex<VecDeque<String>>>;

/// Client connection state managed by Bevy.
#[derive(Resource)]
pub struct ClientConnection {
    /// Send JSON strings to the background network thread.
    pub tx_to_server: MpscSender<String>,
    /// Read JSON strings forwarded from the server.
    pub inbox: SharedInbox,
    /// The client_id assigned by the server.
    pub client_id: String,
    /// True while the WebSocket is open.
    pub connected: bool,
    /// Latest roster.
    pub roster: Vec<PeerInfo>,
    /// UI-facing status string.
    pub status_msg: String,
}

impl Default for ClientConnection {
    fn default() -> Self {
        let (tx, _rx): (MpscSender<String>, _) = mpsc::channel();
        Self {
            tx_to_server: tx,
            inbox: Arc::new(Mutex::new(VecDeque::new())),
            client_id: String::new(),
            connected: false,
            roster: Vec::new(),
            status_msg: "disconnected".to_string(),
        }
    }
}

/// Voxel edits waiting to be sent to the server.
#[derive(Resource, Default)]
pub struct OutgoingEdits {
    pub queue: Vec<VoxelEdit>,
}

/// Tracks when the last heartbeat was sent.
#[derive(Resource, Default)]
pub struct HeartbeatTimer {
    pub last: f32,
}

// --------------------------------------------------------------------------
// Background network thread
// -------------------------------------------------------------------

/// Called on Startup. Spawns a thread that connects and runs the message pump.
pub fn init_net_connection(mut conn: ResMut<ClientConnection>) {
    let (tx, rx): (MpscSender<String>, _) = mpsc::channel();
    let inbox: SharedInbox = Arc::new(Mutex::new(VecDeque::new()));

    conn.tx_to_server = tx.clone();
    conn.inbox = inbox.clone();
    conn.connected = false;
    conn.status_msg = "connecting...".to_string();

    let tx_net = tx;
    std::thread::spawn(move || {
        run_network_thread(tx_net, rx, inbox, "127.0.0.1", 8080, "Player1", [1.0, 0.0, 0.0]);
    });
}

fn run_network_thread(
    _tx_to_server: MpscSender<String>,
    rx: mpsc::Receiver<String>,
    inbox: SharedInbox,
    host: &str,
    port: u16,
    peer_name: &str,
    peer_color: [f32; 3],
) {
    use futures::SinkExt;
    use futures::StreamExt;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime build failed");

        runtime.block_on(async move {
            let addr = format!("{host}:{port}");
            let ws_url = format!("ws://{addr}");
            eprintln!("[net] connecting to {ws_url} as {peer_name}");

            // connect_async returns Result<(WebSocket, Response), Error>
            let ws_stream = match tokio_tungstenite::connect_async(&ws_url).await {
                Ok((ws, _)) => ws,
                Err(ref e) => {
                    eprintln!("[net] connect error: {e}");
                    return;
                }
            };
            let (mut ws_tx, mut ws_rx) = ws_stream.split();

            // Send Join
            let join = ClientMsg::Join {
                name: peer_name.to_string(),
                color_rgb: peer_color,
            };
            let join_json = match serde_json::to_string(&join) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("[net] serialize join failed: {e}");
                    return;
                }
            };
            if ws_tx
                .send(tokio_tungstenite::tungstenite::Message::Text(join_json))
                .await
                .is_err()
            {
                eprintln!("[net] send join failed");
                return;
            }
            eprintln!("[net] sent Join");

            // Message pump: forward both directions
            loop {
                // Drain outgoing messages (Bevy → server)
                loop {
                    match rx.try_recv() {
                        Ok(json) => {
                            if ws_tx
                                .send(tokio_tungstenite::tungstenite::Message::Text(json))
                                .await
                                .is_err()
                            {
                                eprintln!("[net] ws send failed");
                                return;
                            }
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            eprintln!("[net] outgoing channel disconnected");
                            return;
                        }
                    }
                }

                // Await next incoming WebSocket message
                match ws_rx.next().await {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(json))) => {
                        if let Ok(mut inbox_guard) = inbox.lock() {
                            inbox_guard.push_back(json);
                        }
                    }
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(b))) => {
                        if let Ok(s) = String::from_utf8(b) {
                            if let Ok(mut inbox_guard) = inbox.lock() {
                                inbox_guard.push_back(s);
                            }
                        }
                    }
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => {
                        eprintln!("[net] server closed connection");
                        break;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        eprintln!("[net] ws read error: {e}");
                        break;
                    }
                    None => {
                        eprintln!("[net] ws stream ended");
                        break;
                    }
                }
            }

            eprintln!("[net] message pump stopped");
        })
    }));

    if let Err(ref e) = result {
        eprintln!("[net] thread panicked: {e:?}");
    }
}

// --------------------------------------------------------------------------
// Bevy systems
// -------------------------------------------------------------------

/// Drains the inbox and applies server messages to the voxel world.
pub fn process_inbox(
    mut conn: ResMut<ClientConnection>,
    mut world_res: ResMut<VoxelWorldResource>,
) {
    let inbox = conn.inbox.clone();
    let queue: Vec<String> = match inbox.lock() {
        Ok(mut guard) => guard.drain(..).collect(),
        Err(_) => return,
    };

    for json_str in queue {
        let server_msg: ServerMsg = match serde_json::from_str(&json_str) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[net] parse error for message {json_str}: {e}");
                continue;
            }
        };

        match server_msg {
            ServerMsg::Welcome {
                client_id,
                snapshot,
                roster,
            } => {
                conn.client_id = client_id.clone();
                conn.connected = true;
                conn.roster = roster.clone();
                conn.status_msg = "connected".to_string();
                for voxel in &snapshot {
                    world_res.world.place(voxel.position, voxel.color);
                }
                eprintln!(
                    "[bevy] Received snapshot: {} voxels, {} peers",
                    snapshot.len(),
                    roster.len()
                );
            }
            ServerMsg::Roster { users } => {
                let count = users.len();
                conn.roster = users;
                eprintln!("[bevy] Roster: {} peers", count);
            }
            ServerMsg::VoxelEdit {
                client_id,
                position,
                color,
                erase,
            } => {
                if client_id != conn.client_id {
                    let edit = VoxelEdit {
                        position,
                        color,
                        erase,
                    };
                    world_res.world.apply_edit(&edit);
                    eprintln!(
                        "[bevy] Remote edit from {client_id}: pos={position:?} erase={erase}"
                    );
                }
            }
            ServerMsg::HeartbeatAck => {
                log::info!("[net] received heartbeat ack");
            }
            ServerMsg::Error { message } => {
                eprintln!("[net] server error: {message}");
            }
        }
    }
}

/// Send all queued VoxelEdits to the server via the network channel.
pub fn send_queued_edits(conn: Res<ClientConnection>, mut outgoing: ResMut<OutgoingEdits>) {
    for edit in outgoing.queue.drain(..) {
        let msg = ClientMsg::VoxelEdit {
            position: edit.position,
            color: edit.color,
            erase: edit.erase,
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = conn.tx_to_server.send(json);
        }
    }
}

/// Handle local click → place/erase + queue for network.
pub fn handle_click_net(
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    input: Res<ButtonInput<MouseButton>>,
    mut world_res: ResMut<VoxelWorldResource>,
    mut outgoing: ResMut<OutgoingEdits>,
) {
    let (camera, camera_tf) = camera.into_inner();
    let window = *window;

    let just_left = input.just_pressed(MouseButton::Left);
    let just_right = input.just_pressed(MouseButton::Right);
    if !just_left && !just_right {
        return;
    }

    if let Some(cursor) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_tf, cursor) {
            if let Some(point) = ray.plane_intersection_point(
                Vec3::new(0.0, 0.5, 0.0),
                InfinitePlane3d::new(Vec3::Y),
            ) {
                let x = point.x.round() as i32;
                let z = point.z.round() as i32;
                let pos = [x, 0, z];
                let brush_color = world_res.brush_color;
                if just_left {
                    world_res.world.place(pos, brush_color);
                    outgoing.queue.push(VoxelEdit {
                        position: pos,
                        color: brush_color,
                        erase: false,
                    });
                } else {
                    world_res.world.erase(pos);
                    outgoing.queue.push(VoxelEdit {
                        position: pos,
                        color: [0.0, 0.0, 0.0],
                        erase: true,
                    });
                }
            }
        }
    }
}

/// Sends a heartbeat every 30 seconds.
pub fn heartbeat(time: Res<Time>, mut timer: ResMut<HeartbeatTimer>, conn: Res<ClientConnection>) {
    if !conn.connected {
        return;
    }
    let now = time.elapsed_secs();
    if now - timer.last >= 30.0 || timer.last == 0.0 {
        timer.last = now;
        let msg = ClientMsg::Heartbeat;
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = conn.tx_to_server.send(json);
        }
    }
}
// --------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_connection_default() {
        let c = ClientConnection::default();
        assert_eq!(c.status_msg, "disconnected");
        assert!(!c.connected);
        assert_eq!(c.roster, Vec::<PeerInfo>::new());
    }

    #[test]
    fn test_outgoing_edits_default() {
        let o = OutgoingEdits::default();
        assert_eq!(o.queue.len(), 0);
    }

    #[test]
    fn test_heartbeat_timer_default() {
        let t = HeartbeatTimer::default();
        assert_eq!(t.last, 0.0);
    }
}
