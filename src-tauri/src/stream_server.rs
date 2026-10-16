use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

pub const DEFAULT_STREAM_PORT: u16 = 9768;

// Minimal 1x1 black JPEG fallback frame
const PLACEHOLDER_JPEG: &[u8] = &[
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01, 0x00, 0x48,
    0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08,
    0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
    0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20, 0x24, 0x2E, 0x27, 0x20,
    0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29, 0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27,
    0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01,
    0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x1F, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04,
    0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F,
    0x00, 0xBF, 0x00, 0xFF, 0xD9,
];

pub async fn start_stream_server(port: u16, latest_frame: Arc<Mutex<Option<Vec<u8>>>>) {
    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind video stream server on {}: {}", addr, e);
            return;
        }
    };

    println!(
        "MJPEG video stream server listening on http://{}/video",
        addr
    );

    loop {
        if let Ok((mut socket, _)) = listener.accept().await {
            let frame_arc = latest_frame.clone();
            tokio::spawn(async move {
                let header = "HTTP/1.1 200 OK\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Content-Type: multipart/x-mixed-replace; boundary=frame\r\n\
                    Cache-Control: no-cache, no-store, must-revalidate\r\n\
                    Pragma: no-cache\r\n\
                    Expires: 0\r\n\
                    Connection: close\r\n\r\n";

                if socket.write_all(header.as_bytes()).await.is_err() {
                    return;
                }

                let mut interval = tokio::time::interval(Duration::from_millis(33)); // ~30 FPS
                loop {
                    interval.tick().await;

                    let frame_data = {
                        let guard = frame_arc.lock().unwrap();
                        guard.clone()
                    };

                    let bytes_to_send = frame_data.as_deref().unwrap_or(PLACEHOLDER_JPEG);

                    let part_header = format!(
                        "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        bytes_to_send.len()
                    );

                    if socket.write_all(part_header.as_bytes()).await.is_err() {
                        break;
                    }
                    if socket.write_all(bytes_to_send).await.is_err() {
                        break;
                    }
                    if socket.write_all(b"\r\n").await.is_err() {
                        break;
                    }
                }
            });
        }
    }
}
