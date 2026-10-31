use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::broadcast;

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
    last_decodable_id: u32,
    waiting_for_keyframe: bool,
    frame_tx: broadcast::Sender<Vec<u8>>,
}

impl ScreenReceiver {
    pub fn new(frame_tx: broadcast::Sender<Vec<u8>>) -> Self {
        Self {
            assemblies: HashMap::new(),
            stream_id: 0,
            latest_frame_id: 0,
            last_decodable_id: 0,
            waiting_for_keyframe: true,
            frame_tx,
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
            self.last_decodable_id = 0;
            self.waiting_for_keyframe = true;
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

        // When all chunks are received, release the encoded frame
        if assembly.chunks_received == assembly.total_chunks {
            let mut frame_bytes = Vec::new();
            for bytes in assembly.chunks.iter().flatten() {
                frame_bytes.extend_from_slice(bytes);
            }

            self.assemblies.remove(&frame_id);

            self.release_frame(frame_id, frame_bytes);
        }
    }

    /// A lost frame breaks every later one, so playback resumes at the next keyframe.
    fn release_frame(&mut self, frame_id: u32, frame: Vec<u8>) {
        if frame_id <= self.last_decodable_id {
            return;
        }
        if frame_id != self.last_decodable_id + 1 {
            self.waiting_for_keyframe = true;
        }
        if self.waiting_for_keyframe && !is_keyframe(&frame) {
            return;
        }
        self.waiting_for_keyframe = false;
        self.last_decodable_id = frame_id;
        let _ = self.frame_tx.send(frame);
    }
}

/// True when the H.264 access unit holds an IDR slice.
pub fn is_keyframe(frame: &[u8]) -> bool {
    frame
        .windows(4)
        .any(|w| w[0] == 0 && w[1] == 0 && w[2] == 1 && w[3] & 0x1F == 5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::TAG_VIDEO;

    fn packet(stream_id: u32, frame_id: u32, nal_type: u8) -> Vec<u8> {
        let mut data = vec![TAG_VIDEO];
        data.extend_from_slice(&stream_id.to_be_bytes());
        data.extend_from_slice(&frame_id.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&1u16.to_be_bytes());
        data.extend_from_slice(&[0, 0, 1, nal_type]);
        data
    }

    #[test]
    fn new_stream_is_accepted_after_frame_counter_restarts() {
        let (tx, mut rx) = broadcast::channel(8);
        let mut receiver = ScreenReceiver::new(tx);

        receiver.handle_packet(&packet(1, 9000, 0x65));
        receiver.handle_packet(&packet(2, 1, 0x65));

        assert_eq!(rx.try_recv().unwrap(), vec![0, 0, 1, 0x65]);
        assert_eq!(rx.try_recv().unwrap(), vec![0, 0, 1, 0x65]);
    }

    #[test]
    fn lost_frame_waits_for_next_keyframe() {
        let (tx, mut rx) = broadcast::channel(8);
        let mut receiver = ScreenReceiver::new(tx);

        receiver.handle_packet(&packet(1, 1, 0x65));
        receiver.handle_packet(&packet(1, 3, 0x41));
        receiver.handle_packet(&packet(1, 4, 0x65));

        assert_eq!(rx.try_recv().unwrap(), vec![0, 0, 1, 0x65]);
        assert_eq!(rx.try_recv().unwrap(), vec![0, 0, 1, 0x65]);
        assert!(rx.try_recv().is_err());
    }
}
