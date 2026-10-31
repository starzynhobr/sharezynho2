use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast::{self, error::RecvError};

use crate::screen::receiver::is_keyframe;

pub const DEFAULT_STREAM_PORT: u16 = 9768;

/// Serves encoded frames to the webview, each prefixed with its big-endian length.
pub async fn start_stream_server(port: u16, frame_tx: broadcast::Sender<Vec<u8>>) {
    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind video stream server on {}: {}", addr, e);
            return;
        }
    };

    println!(
        "H.264 video stream server listening on http://{}/video",
        addr
    );

    loop {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut frame_rx = frame_tx.subscribe();
            tokio::spawn(async move {
                let header = "HTTP/1.1 200 OK\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Content-Type: application/octet-stream\r\n\
                    Cache-Control: no-store\r\n\
                    Connection: close\r\n\r\n";

                if socket.write_all(header.as_bytes()).await.is_err() {
                    return;
                }
                let _ = socket.set_nodelay(true);

                let mut needs_keyframe = true;
                loop {
                    let frame = match frame_rx.recv().await {
                        Ok(frame) => frame,
                        Err(RecvError::Lagged(_)) => {
                            needs_keyframe = true;
                            continue;
                        }
                        Err(RecvError::Closed) => break,
                    };
                    if needs_keyframe && !is_keyframe(&frame) {
                        continue;
                    }
                    needs_keyframe = false;

                    let length = (frame.len() as u32).to_be_bytes();
                    if socket.write_all(&length).await.is_err()
                        || socket.write_all(&frame).await.is_err()
                    {
                        break;
                    }
                }
            });
        }
    }
}
