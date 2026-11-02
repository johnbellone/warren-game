//! Warren Game Server — WebSocket server binary for collaborative voxel editing.
//!
//! Usage:
//!   cargo run -p warren-server -- --port 3000
//!   cargo run -p warren-server -- --host 0.0.0.0 --port 3000

use std::sync::Arc;
use tokio::net::TcpListener;
use log::info;

use warren_server::server::{ServerState, handle_client};

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let mut port: u16 = 3000;
    let mut host = String::from("127.0.0.1");

    // Simple arg parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                i += 1;
                if i < args.len() {
                    port = args[i].parse().unwrap_or(3000);
                }
            }
            "--host" => {
                i += 1;
                if i < args.len() {
                    host = args[i].clone();
                }
            }
            _ => {}
        }
        i += 1;
    }

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind to port");
    info!("Warren Game Server listening on ws://{} (broadcast capacity: 4096)", addr);

    let state = Arc::new(tokio::sync::Mutex::new(ServerState::new(4096)));

    loop {
        let (stream, _req) = listener.accept().await.expect("Failed to accept connection");
        let new_state = state.clone();
        let client_num = {
            let mut s = new_state.lock().await;
            let n = s.next_id;
            s.next_id += 1;
            n
        };
        let client_id = format!("client-{}", client_num);
        let broadcast_rx = {
            let s = state.lock().await;
            s.broadcast.subscribe()
        };

        // Convert TcpStream into a WebSocket stream
        let ws_stream = tokio_tungstenite::accept_async(stream).await;
        match ws_stream {
            Ok(ws) => {
                tokio::spawn(handle_client(new_state, ws, client_id, broadcast_rx));
            }
            Err(e) => {
                log::warn!("WebSocket upgrade failed: {}", e);
            }
        }
    }
}
