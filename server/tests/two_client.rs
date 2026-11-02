//! Integration test: two-client VoxelEdit sync via server logic.
//!
//! Uses `ServerState` directly (no network required) to verify that:
//! 1. Two peers can join
//! 2. VoxelEdits from one peer are applied to the shared world
//! 3. Roster state is correct after both joins

use warren_protocol::{ClientMsg, PeerInfo};
use warren_server::server::ServerState;

#[test]
fn two_client_voxel_edit_sync() {
    let mut state = ServerState::new(2048);
    assert_eq!(state.voxels.len(), 0);
    assert_eq!(state.peers.len(), 0);

    // Client 1 joins
    let join1 = ClientMsg::Join {
        name: "Alice".to_string(),
        color_rgb: [1.0, 0.0, 0.0],
    };
    state.process_message("client1", &join1);
    assert_eq!(state.peers.len(), 1, "state should have 1 peer after first join");
    eprintln!("[test] client1 (Alice) joined, peers={}", state.peers.len());

    // Client 2 joins
    let join2 = ClientMsg::Join {
        name: "Bob".to_string(),
        color_rgb: [0.0, 1.0, 0.0],
    };
    state.process_message("client2", &join2);
    assert_eq!(state.peers.len(), 2, "state should have 2 peers after second join");
    eprintln!("[test] client2 (Bob) joined, peers={}", state.peers.len());

    // Client 1 places a voxel at [5, 0, 5]
    let place1 = ClientMsg::VoxelEdit {
        position: [5, 0, 5],
        color: [1.0, 0.0, 0.0],
        erase: false,
    };
    state.process_message("client1", &place1);
    assert_eq!(state.voxels.len(), 1, "state should have 1 voxel after place");
    eprintln!("[test] client1 placed voxel at [5,0,5], voxels={}", state.voxels.len());

    // Verify the voxel is in the world
    let voxel = state.voxels.get(&[5, 0, 5]);
    assert!(voxel.is_some(), "voxel at [5,0,5] should exist");
    assert_eq!(voxel.unwrap().color, [1.0, 0.0, 0.0]);

    // Client 2 places another voxel
    let place2 = ClientMsg::VoxelEdit {
        position: [0, 0, 0],
        color: [0.0, 1.0, 0.0],
        erase: false,
    };
    state.process_message("client2", &place2);
    assert_eq!(state.voxels.len(), 2, "state should have 2 voxels after two places");
    eprintln!("[test] client2 placed voxel at [0,0,0], voxels={}", state.voxels.len());

    // Client 1 erases the voxel at [5, 0, 5]
    let erase1 = ClientMsg::VoxelEdit {
        position: [5, 0, 5],
        color: [0.0, 0.0, 0.0],
        erase: true,
    };
    state.process_message("client1", &erase1);
    assert_eq!(state.voxels.len(), 1, "state should have 1 voxel after erase");
    assert_eq!(state.peers.len(), 2, "both peers should still be connected");
    eprintln!("[test] client1 erased [5,0,5], voxels={}", state.voxels.len());

    // Peers list is intact
    let _ = state.peers.get("client1").expect("client1 should still be in roster");
    let _ = state.peers.get("client2").expect("client2 should still be in roster");

    eprintln!("[test] ✅ two-client VoxelEdit sync test passed");
}

#[test]
fn peer_join_update_roster() {
    let mut state = ServerState::new(2048);

    state.process_message("a", &ClientMsg::Join { name: "A".into(), color_rgb: [0.0, 0.0, 0.0] });
    state.process_message("b", &ClientMsg::Join { name: "B".into(), color_rgb: [0.0, 0.0, 0.0] });
    assert_eq!(state.peers.len(), 2, "two peers should be registered after two joins");

    eprintln!("[test] ✅ peer join roster test passed");
}
