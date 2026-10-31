use crate::protocol::TAG_VIDEO;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[cfg(target_os = "macos")]
use super::capture_macos as backend;
#[cfg(windows)]
use super::capture_windows as backend;

const CHUNK_SIZE: usize = 1200;
const TARGET_FPS: u64 = 30;
const MAX_WIDTH: usize = 1920;
const MAX_BYTES_PER_SEC: u64 = 4_000_000;
const MAX_FRAME_BYTES: usize = 150 * 1024;
const MIN_FRAME_BYTES: usize = 90 * 1024;
const MIN_QUALITY: u8 = 35;
const MAX_QUALITY: u8 = 65;
// The sender's own preview reuses every Nth encoded frame, about 5 FPS at most
const PREVIEW_FRAME_INTERVAL: u32 = 6;

pub struct ScreenCapture {
    pub is_sharing: Arc<AtomicBool>,
    pub stop_flag: Arc<AtomicBool>,
}

pub(super) struct CaptureFlags {
    is_sharing: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    packet_tx: mpsc::Sender<Vec<u8>>,
    stream_id: u32,
    frame_counter: std::sync::atomic::AtomicU32,
    next_frame_time: std::sync::Mutex<Instant>,
    quality: std::sync::atomic::AtomicU8,
    preview_frame: Arc<std::sync::Mutex<Option<Vec<u8>>>>,
}

impl CaptureFlags {
    pub(super) fn should_stop(&self) -> bool {
        self.stop_flag.load(Ordering::Relaxed)
    }

    /// True when sharing is on and the FPS and bandwidth budget allow a frame.
    pub(super) fn ready_for_frame(&self) -> bool {
        self.is_sharing.load(Ordering::Relaxed)
            && Instant::now() >= *self.next_frame_time.lock().unwrap()
    }

    /// Encodes one tightly packed RGBA frame and queues its packets.
    pub(super) fn send_rgba(&self, raw_pixels: &[u8], width: usize, height: usize) {
        let frame_start = Instant::now();

        // Encode to JPEG
        let mut jpeg_bytes = Vec::with_capacity(64 * 1024);
        let factor = width.div_ceil(MAX_WIDTH).max(1);
        let scaled = (factor > 1).then(|| downscale_rgba(raw_pixels, width, height, factor));
        let (pixels, out_width, out_height) = match &scaled {
            Some(pixels) => (pixels.as_slice(), width / factor, height / factor),
            None => (raw_pixels, width, height),
        };
        let quality = self.quality.load(Ordering::Relaxed);
        let encoder = jpeg_encoder::Encoder::new(&mut jpeg_bytes, quality);
        if encoder
            .encode(
                pixels,
                out_width as u16,
                out_height as u16,
                jpeg_encoder::ColorType::Rgba,
            )
            .is_err()
        {
            return;
        }

        if jpeg_bytes.len() > MAX_FRAME_BYTES {
            self.quality.store(
                quality.saturating_sub(5).max(MIN_QUALITY),
                Ordering::Relaxed,
            );
        } else if jpeg_bytes.len() < MIN_FRAME_BYTES {
            self.quality
                .store((quality + 5).min(MAX_QUALITY), Ordering::Relaxed);
        }

        let budget = Duration::from_millis(jpeg_bytes.len() as u64 * 1000 / MAX_BYTES_PER_SEC);
        *self.next_frame_time.lock().unwrap() =
            frame_start + budget.max(Duration::from_millis(1000 / TARGET_FPS));

        let frame_id = self.frame_counter.fetch_add(1, Ordering::Relaxed);

        let total_chunks = jpeg_bytes.len().div_ceil(CHUNK_SIZE) as u16;

        for (chunk_idx, chunk) in jpeg_bytes.chunks(CHUNK_SIZE).enumerate() {
            let mut packet = Vec::with_capacity(13 + chunk.len());
            packet.push(TAG_VIDEO);
            packet.extend_from_slice(&self.stream_id.to_be_bytes());
            packet.extend_from_slice(&frame_id.to_be_bytes());
            packet.extend_from_slice(&(chunk_idx as u16).to_be_bytes());
            packet.extend_from_slice(&total_chunks.to_be_bytes());
            packet.extend_from_slice(chunk);

            let _ = self.packet_tx.try_send(packet);
        }

        if frame_id % PREVIEW_FRAME_INTERVAL == 1 {
            *self.preview_frame.lock().unwrap() = Some(jpeg_bytes);
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
        packet_tx: mpsc::Sender<Vec<u8>>,
        preview_frame: Arc<std::sync::Mutex<Option<Vec<u8>>>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let is_sharing = Arc::new(AtomicBool::new(true));
        let stop_flag = Arc::new(AtomicBool::new(false));

        let flags = Arc::new(CaptureFlags {
            is_sharing: is_sharing.clone(),
            stop_flag: stop_flag.clone(),
            packet_tx,
            stream_id: fastrand::u32(1..),
            frame_counter: std::sync::atomic::AtomicU32::new(1),
            next_frame_time: std::sync::Mutex::new(Instant::now()),
            quality: std::sync::atomic::AtomicU8::new(MAX_QUALITY),
            preview_frame,
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
