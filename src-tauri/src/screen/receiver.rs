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
    stream_id: u32,
    latest_frame_id: u32,
    latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
}

impl ScreenReceiver {
    pub fn new(latest_frame: Arc<Mutex<Option<Vec<u8>>>>) -> Self {
        Self {
            assemblies: HashMap::new(),
            stream_id: 0,
            latest_frame_id: 0,
            latest_frame,
        }
    }

    pub fn handle_packet(&mut self, data: &[u8]) {
        if data.len() < 13 {
            return;
        }

        let stream_id = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
        let frame_id = u32::from_be_bytes([data[5], data[6], data[7], data[8]]);
        let chunk_index = u16::from_be_bytes([data[9], data[10]]);
        let total_chunks = u16::from_be_bytes([data[11], data[12]]);
        let chunk_data = &data[13..];

        // Each capture restarts its frame counter, so a new stream resets tracking
        if stream_id != self.stream_id {
            self.stream_id = stream_id;
            self.latest_frame_id = 0;
            self.assemblies.clear();
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::TAG_VIDEO;

    fn packet(stream_id: u32, frame_id: u32, payload: u8) -> Vec<u8> {
        let mut data = vec![TAG_VIDEO];
        data.extend_from_slice(&stream_id.to_be_bytes());
        data.extend_from_slice(&frame_id.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&1u16.to_be_bytes());
        data.push(payload);
        data
    }

    #[test]
    fn new_stream_is_accepted_after_frame_counter_restarts() {
        let latest = Arc::new(Mutex::new(None));
        let mut receiver = ScreenReceiver::new(latest.clone());

        receiver.handle_packet(&packet(1, 9000, 1));
        receiver.handle_packet(&packet(2, 1, 2));

        assert_eq!(*latest.lock().unwrap(), Some(vec![2]));
    }
}
