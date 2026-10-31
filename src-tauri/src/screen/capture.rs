use crate::protocol::TAG_VIDEO;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

use super::encoder::{H264Encoder, VideoQuality};

#[cfg(target_os = "macos")]
use super::capture_macos as backend;
#[cfg(windows)]
use super::capture_windows as backend;

const CHUNK_SIZE: usize = 1200;

pub struct ScreenCapture {
    pub is_sharing: Arc<AtomicBool>,
    pub stop_flag: Arc<AtomicBool>,
}

pub(super) struct CaptureFlags {
    is_sharing: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    packet_tx: mpsc::Sender<Vec<u8>>,
    stream_id: u32,
    frame_counter: Arc<AtomicU32>,
    next_frame_time: Mutex<Instant>,
    quality: VideoQuality,
    encoder: Mutex<Option<H264Encoder>>,
    preview_tx: broadcast::Sender<Vec<u8>>,
}

impl CaptureFlags {
    pub(super) fn should_stop(&self) -> bool {
        self.stop_flag.load(Ordering::Relaxed)
    }

    /// True when sharing is on and the selected frame rate allows a frame.
    pub(super) fn ready_for_frame(&self) -> bool {
        self.is_sharing.load(Ordering::Relaxed)
            && Instant::now() >= *self.next_frame_time.lock().unwrap()
    }

    /// Queues one tightly packed RGBA frame for the H.264 encoder.
    pub(super) fn send_rgba(&self, raw_pixels: &[u8], width: usize, height: usize) {
        *self.next_frame_time.lock().unwrap() =
            Instant::now() + Duration::from_millis(1000 / self.quality.fps as u64);

        // Halving large captures first keeps the pipe to the encoder affordable
        let factor = (height / self.quality.height as usize).max(1);
        let scaled = (factor > 1).then(|| downscale_rgba(raw_pixels, width, height, factor));
        let (pixels, out_width, out_height) = match scaled {
            Some(pixels) => (pixels, width / factor, height / factor),
            None => (raw_pixels.to_vec(), width, height),
        };

        let mut encoder = self.encoder.lock().unwrap();
        if !encoder
            .as_ref()
            .is_some_and(|e| e.matches(out_width, out_height))
        {
            *encoder = H264Encoder::start(
                out_width,
                out_height,
                self.quality,
                self.stop_flag.clone(),
                self.frame_sink(),
            )
            .map_err(|e| eprintln!("Falha ao iniciar o encoder: {}", e))
            .ok();
        }
        if let Some(encoder) = encoder.as_ref() {
            encoder.push_frame(pixels);
        }
    }

    /// Splits encoded frames into packets and mirrors them to the local preview.
    fn frame_sink(&self) -> impl FnMut(Vec<u8>) + Send + 'static {
        let packet_tx = self.packet_tx.clone();
        let preview_tx = self.preview_tx.clone();
        let stream_id = self.stream_id;
        let frame_counter = self.frame_counter.clone();
        move |frame: Vec<u8>| {
            let frame_id = frame_counter.fetch_add(1, Ordering::Relaxed);
            let total_chunks = frame.len().div_ceil(CHUNK_SIZE) as u16;

            for (chunk_idx, chunk) in frame.chunks(CHUNK_SIZE).enumerate() {
                let mut packet = Vec::with_capacity(13 + chunk.len());
                packet.push(TAG_VIDEO);
                packet.extend_from_slice(&stream_id.to_be_bytes());
                packet.extend_from_slice(&frame_id.to_be_bytes());
                packet.extend_from_slice(&(chunk_idx as u16).to_be_bytes());
                packet.extend_from_slice(&total_chunks.to_be_bytes());
                packet.extend_from_slice(chunk);

                let _ = packet_tx.try_send(packet);
            }

            let _ = preview_tx.send(frame);
        }
    }
}

/// Shrinks an RGBA image by an integer factor, averaging each pixel block.
fn downscale_rgba(pixels: &[u8], width: usize, height: usize, factor: usize) -> Vec<u8> {
    let out_width = width / factor;
    let out_height = height / factor;
    let block_area = (factor * factor) as u32;
    let mut output = Vec::with_capacity(out_width * out_height * 4);
    for out_y in 0..out_height {
        for out_x in 0..out_width {
            let mut sum = [0u32; 4];
            for y in 0..factor {
                let row = (out_y * factor + y) * width * 4;
                for x in 0..factor {
                    let offset = row + (out_x * factor + x) * 4;
                    for (total, value) in sum.iter_mut().zip(&pixels[offset..offset + 4]) {
                        *total += *value as u32;
                    }
                }
            }
            output.extend(sum.map(|total| (total / block_area) as u8));
        }
    }
    output
}

impl ScreenCapture {
    pub fn enumerate_monitors() -> Vec<String> {
        backend::monitor_names()
    }

    pub fn start(
        monitor_index: usize,
        quality: VideoQuality,
        packet_tx: mpsc::Sender<Vec<u8>>,
        preview_tx: broadcast::Sender<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let is_sharing = Arc::new(AtomicBool::new(true));
        let stop_flag = Arc::new(AtomicBool::new(false));

        let flags = Arc::new(CaptureFlags {
            is_sharing: is_sharing.clone(),
            stop_flag: stop_flag.clone(),
            packet_tx,
            stream_id: fastrand::u32(1..),
            frame_counter: Arc::new(AtomicU32::new(1)),
            next_frame_time: Mutex::new(Instant::now()),
            quality,
            encoder: Mutex::new(None),
            preview_tx,
        });

        backend::start(monitor_index, flags)?;

        Ok(Self {
            is_sharing,
            stop_flag,
        })
    }

    pub fn stop(&self) {
        self.is_sharing.store(false, Ordering::Relaxed);
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downscale_averages_each_block() {
        let pixels = [0, 0, 0, 255, 100, 100, 100, 255];
        assert_eq!(downscale_rgba(&pixels, 2, 1, 1).len(), 8);

        let square = [
            0, 0, 0, 255, 40, 40, 40, 255, //
            80, 80, 80, 255, 120, 120, 120, 255,
        ];
        assert_eq!(downscale_rgba(&square, 2, 2, 2), vec![60, 60, 60, 255]);
    }
}
