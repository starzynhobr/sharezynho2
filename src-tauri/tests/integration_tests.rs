use std::sync::mpsc;
use std::time::Duration;
use tokio::time::sleep;

use sharezynho::client::{ClientEvent, ClientSession};
use sharezynho::network::tailscale::preferred_host;
use sharezynho::protocol::{
    ClientMessage, ParticipantInfo, ServerMessage, serialize_client_msg, serialize_server_msg,
};
use sharezynho::server::RoomServer;

#[test]
fn test_preferred_host_returns_valid_ip() {
    let host = preferred_host();
    assert!(!host.is_empty());
    assert!(host.parse::<std::net::Ipv4Addr>().is_ok());
}

#[test]
fn test_protocol_serialization_roundtrip() {
    let client_join = ClientMessage::Join {
        name: "TestUser".to_string(),
        room: "DESIGN".to_string(),
        udp_port: 8766,
    };
    let s = serialize_client_msg(&client_join);
    assert!(s.ends_with('\n'));
    let deserialized: ClientMessage = serde_json::from_str(&s).unwrap();
    match deserialized {
        ClientMessage::Join {
            name,
            room,
            udp_port,
        } => {
            assert_eq!(name, "TestUser");
            assert_eq!(room, "DESIGN");
            assert_eq!(udp_port, 8766);
        }
        _ => panic!("Expected Join variant"),
    }

    let server_state = ServerMessage::RoomState {
        room: "DESIGN".to_string(),
        participants: vec![ParticipantInfo {
            id: "user-123".to_string(),
            name: "Alice".to_string(),
            is_speaking: true,
            is_host: true,
            is_sharing_screen: false,
        }],
        screen_presenter: None,
    };
    let s2 = serialize_server_msg(&server_state);
    assert!(s2.ends_with('\n'));
    let deserialized2: ServerMessage = serde_json::from_str(&s2).unwrap();
    match deserialized2 {
        ServerMessage::RoomState {
            room,
            participants,
            screen_presenter,
        } => {
            assert_eq!(room, "DESIGN");
            assert_eq!(participants.len(), 1);
            assert!(participants[0].is_speaking);
            assert!(screen_presenter.is_none());
        }
        _ => panic!("Expected RoomState variant"),
    }
}

#[tokio::test]
async fn test_room_server_and_client_end_to_end() {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let test_tcp_port = 28765;
    let test_udp_port = 28766;

    // 1. Start Server
    let server_handle = RoomServer::start(test_tcp_port, test_udp_port, shutdown_rx)
        .await
        .expect("Failed to start test room server");

    sleep(Duration::from_millis(50)).await;

    // 2. Connect Alice
    let (alice_tx, alice_rx) = mpsc::channel();
    let server_addr = format!("127.0.0.1:{}", test_tcp_port);
    let mut alice = ClientSession::connect(&server_addr, "RUSTROOM", "Alice", true, alice_tx)
        .await
        .expect("Alice failed to connect");

    assert_eq!(alice.room_name, "RUSTROOM");
    assert_eq!(alice.display_name, "Alice");
    assert!(alice.is_host);

    // 3. Connect Bob
    let (bob_tx, _bob_rx) = mpsc::channel();
    let mut bob = ClientSession::connect(&server_addr, "RUSTROOM", "Bob", false, bob_tx)
        .await
        .expect("Bob failed to connect");

    assert_eq!(bob.room_name, "RUSTROOM");
    assert_eq!(bob.display_name, "Bob");
    assert!(!bob.is_host);

    // 4. Verify Alice receives RoomState with both participants
    sleep(Duration::from_millis(100)).await;

    let mut found_both = false;
    while let Ok(evt) = alice_rx.try_recv() {
        if let ClientEvent::RoomStateUpdated { participants, .. } = evt
            && participants.len() == 2
        {
            found_both = true;
            break;
        }
    }
    assert!(
        found_both,
        "Alice should receive room state with 2 participants"
    );

    // 5. Clean disconnect
    alice.disconnect();
    bob.disconnect();
    let _ = shutdown_tx.send(true);
    let _ = server_handle.await;
}
