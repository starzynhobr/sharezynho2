use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

struct FrameAssembly {
    total_chunks: u16,
    chunks_received: u16,
    chunks: Vec<Option<Vec<u8>>>,
    created_at: Instant,
}

pub struct ScreenReceiver {
    assemblies: HashMap<u32, FrameAssembly>,
    latest_frame_id: u32,
    latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
}

impl ScreenReceiver {
    pub fn new(latest_frame: Arc<Mutex<Option<Vec<u8>>>>) -> Self {
        Self {
            assemblies: HashMap::new(),
            latest_frame_id: 0,
            latest_frame,
        }
    }

    pub fn handle_packet(&mut self, data: &[u8]) {
        if data.len() < 9 {
            return;
        }

        let frame_id = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
        let chunk_index = u16::from_be_bytes([data[5], data[6]]);
        let total_chunks = u16::from_be_bytes([data[7], data[8]]);
        let chunk_data = &data[9..];

        // If this frame is much older than our current known frame, discard
        if frame_id + 5 < self.latest_frame_id {
            return;
        }
        if frame_id > self.latest_frame_id {
            self.latest_frame_id = frame_id;
        }

        // Clean up stale assemblies older than 500ms
        self.assemblies
            .retain(|_, a| a.created_at.elapsed().as_millis() < 500);

        let assembly = self.assemblies.entry(frame_id).or_insert_with(|| {
            let mut chunks = Vec::with_capacity(total_chunks as usize);
            chunks.resize(total_chunks as usize, None);
            FrameAssembly {
                total_chunks,
                chunks_received: 0,
                chunks,
                created_at: Instant::now(),
            }
        });

        if (chunk_index as usize) < assembly.chunks.len()
            && assembly.chunks[chunk_index as usize].is_none()
        {
            assembly.chunks[chunk_index as usize] = Some(chunk_data.to_vec());
            assembly.chunks_received += 1;
        }

        // When all chunks are received, store complete JPEG bytes
        if assembly.chunks_received == assembly.total_chunks {
            let mut jpeg_bytes = Vec::new();
            for bytes in assembly.chunks.iter().flatten() {
                jpeg_bytes.extend_from_slice(bytes);
            }

            self.assemblies.remove(&frame_id);

            let mut guard = self.latest_frame.lock().unwrap();
            *guard = Some(jpeg_bytes);
        }
    }
}
