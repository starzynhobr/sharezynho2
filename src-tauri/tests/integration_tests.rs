use std::sync::mpsc;
use std::time::Duration;
use tokio::time::sleep;

use sharezynho::client::{ClientEvent, ClientSession};
use sharezynho::network::tailscale::preferred_host;
use sharezynho::protocol::{
    ClientMessage, ParticipantInfo, ServerMessage, TAG_AUDIO, serialize_client_msg,
    serialize_server_msg,
};
use sharezynho::screen::VideoQuality;
use sharezynho::screen::receiver::is_keyframe;
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

    // Wrong room code is refused
    let (wrong_tx, _wrong_rx) = mpsc::channel();
    let wrong_room = ClientSession::connect(&server_addr, "OTHER", "Eve", false, wrong_tx).await;
    assert!(wrong_room.is_err());

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

async fn join_raw(tcp_port: u16, room: &str, udp_port: u16) -> tokio::net::TcpStream {
    use tokio::io::AsyncWriteExt;
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", tcp_port))
        .await
        .unwrap();
    let join = ClientMessage::Join {
        name: "Raw".to_string(),
        room: room.to_string(),
        udp_port,
    };
    stream
        .write_all(serialize_client_msg(&join).as_bytes())
        .await
        .unwrap();
    sleep(Duration::from_millis(50)).await;
    stream
}

#[tokio::test]
async fn test_relay_only_serves_admitted_participants() {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let (tcp_port, udp_port) = (28775, 28776);
    let server = RoomServer::start(tcp_port, udp_port, shutdown_rx)
        .await
        .unwrap();
    sleep(Duration::from_millis(50)).await;

    let member_a = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let member_b = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let outsider = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port_a = member_a.local_addr().unwrap().port();
    let port_b = member_b.local_addr().unwrap().port();
    let _tcp_a = join_raw(tcp_port, "ROOM", port_a).await;
    let _tcp_b = join_raw(tcp_port, "ROOM", port_b).await;

    let relay = ("127.0.0.1", udp_port);
    let mut buf = [0u8; 16];

    outsider.send_to(&[TAG_AUDIO, 9], relay).await.unwrap();
    let blocked =
        tokio::time::timeout(Duration::from_millis(200), member_b.recv_from(&mut buf)).await;
    assert!(blocked.is_err(), "outsider packet must not be relayed");

    member_a.send_to(&[TAG_AUDIO, 7], relay).await.unwrap();
    let (len, _) = tokio::time::timeout(Duration::from_secs(1), member_b.recv_from(&mut buf))
        .await
        .expect("member packet must be relayed")
        .unwrap();
    assert_eq!(&buf[..len], &[TAG_AUDIO, 7]);

    let _ = shutdown_tx.send(true);
    let _ = server.await;
}

#[tokio::test]
async fn test_room_shutdown_closes_guest_connections() {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let tcp_port = 28785;
    let server = RoomServer::start(tcp_port, 28786, shutdown_rx)
        .await
        .unwrap();
    sleep(Duration::from_millis(50)).await;

    let (guest_tx, guest_rx) = mpsc::channel();
    let addr = format!("127.0.0.1:{}", tcp_port);
    let _guest = ClientSession::connect(&addr, "ROOM", "Guest", false, guest_tx)
        .await
        .unwrap();

    let _ = shutdown_tx.send(true);
    let _ = server.await;
    sleep(Duration::from_millis(200)).await;

    let disconnected = guest_rx
        .try_iter()
        .any(|event| matches!(event, ClientEvent::Disconnected));
    assert!(disconnected);
}

#[tokio::test]
async fn test_guest_detects_unresponsive_host_and_hostname_works() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:28795")
        .await
        .unwrap();
    let silent_host = tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let (mut socket, _) = listener.accept().await.unwrap();
        let admitted = ServerMessage::Admitted {
            participant_id: "guest".to_string(),
            room: "ROOM".to_string(),
            udp_relay_port: 28796,
        };
        socket
            .write_all(serialize_server_msg(&admitted).as_bytes())
            .await
            .unwrap();
        sleep(Duration::from_secs(20)).await;
    });

    let (guest_tx, guest_rx) = mpsc::channel();
    let _guest = ClientSession::connect("localhost:28795", "ROOM", "Guest", false, guest_tx)
        .await
        .unwrap();

    sleep(Duration::from_secs(16)).await;
    let disconnected = guest_rx
        .try_iter()
        .any(|event| matches!(event, ClientEvent::Disconnected));
    assert!(disconnected);
    silent_host.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_real_screen_capture_reaches_viewer_and_can_restart() {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let tcp_port = 28805;
    let server = RoomServer::start(tcp_port, 28806, shutdown_rx)
        .await
        .unwrap();
    sleep(Duration::from_millis(50)).await;

    let addr = format!("127.0.0.1:{}", tcp_port);
    let (host_tx, _host_rx) = mpsc::channel();
    let (viewer_tx, _viewer_rx) = mpsc::channel();
    let mut host = ClientSession::connect(&addr, "ROOM", "Host", true, host_tx)
        .await
        .unwrap();
    let viewer = ClientSession::connect(&addr, "ROOM", "Viewer", false, viewer_tx)
        .await
        .unwrap();

    for attempt in 0..2 {
        let mut frames = viewer.video_frame_tx.subscribe();
        let quality = VideoQuality {
            height: 720,
            fps: 30,
        };
        tokio::task::block_in_place(|| host.start_screen_share(0, quality)).unwrap();

        let frame = tokio::time::timeout(Duration::from_secs(10), frames.recv()).await;
        host.stop_screen_share();

        let frame = frame
            .unwrap_or_else(|_| panic!("no frame on attempt {attempt}"))
            .unwrap();
        assert!(is_keyframe(&frame), "stream must start on a keyframe");
        sleep(Duration::from_millis(300)).await;
    }

    let _ = shutdown_tx.send(true);
    let _ = server.await;
}
