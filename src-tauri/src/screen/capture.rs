use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
};

use crate::protocol::TAG_VIDEO;

const CHUNK_SIZE: usize = 1200;
const TARGET_FPS: u64 = 30;

pub struct ScreenCapture {
    pub is_sharing: Arc<AtomicBool>,
    pub stop_flag: Arc<AtomicBool>,
}

struct CaptureFlags {
    is_sharing: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    packet_tx: mpsc::Sender<Vec<u8>>,
    stream_id: u32,
    frame_counter: std::sync::atomic::AtomicU32,
    last_frame_time: std::sync::Mutex<Instant>,
    temp_buf: std::sync::Mutex<Vec<u8>>,
}

struct CaptureHandler {
    flags: Arc<CaptureFlags>,
}

impl GraphicsCaptureApiHandler for CaptureHandler {
    type Flags = Arc<CaptureFlags>;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { flags: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        if self.flags.stop_flag.load(Ordering::Relaxed) {
            capture_control.stop();
            return Ok(());
        }

        if !self.flags.is_sharing.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Rate limit to target FPS (~30 FPS = ~33ms)
        {
            let mut last_time = self.flags.last_frame_time.lock().unwrap();
            if last_time.elapsed() < Duration::from_millis(1000 / TARGET_FPS) {
                return Ok(());
            }
            *last_time = Instant::now();
        }

        let width = frame.width();
        let height = frame.height();
        let buffer = match frame.buffer() {
            Ok(b) => b,
            Err(_) => return Ok(()),
        };

        let mut temp_guard = self.flags.temp_buf.lock().unwrap();
        let raw_pixels = buffer.as_nopadding_buffer(&mut temp_guard);

        // Encode to JPEG
        let mut jpeg_bytes = Vec::with_capacity(64 * 1024);
        let encoder = jpeg_encoder::Encoder::new(&mut jpeg_bytes, 65);
        if encoder
            .encode(
                raw_pixels,
                width as u16,
                height as u16,
                jpeg_encoder::ColorType::Rgba,
            )
            .is_err()
        {
            return Ok(());
        }

        let frame_id = self.flags.frame_counter.fetch_add(1, Ordering::Relaxed);

        let total_chunks = jpeg_bytes.len().div_ceil(CHUNK_SIZE) as u16;

        for (chunk_idx, chunk) in jpeg_bytes.chunks(CHUNK_SIZE).enumerate() {
            let mut packet = Vec::with_capacity(13 + chunk.len());
            packet.push(TAG_VIDEO);
            packet.extend_from_slice(&self.flags.stream_id.to_be_bytes());
            packet.extend_from_slice(&frame_id.to_be_bytes());
            packet.extend_from_slice(&(chunk_idx as u16).to_be_bytes());
            packet.extend_from_slice(&total_chunks.to_be_bytes());
            packet.extend_from_slice(chunk);

            let _ = self.flags.packet_tx.try_send(packet);
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl ScreenCapture {
    pub fn enumerate_monitors() -> Vec<String> {
        Monitor::enumerate()
            .map(|list| {
                list.into_iter()
                    .map(|m| m.name().unwrap_or_else(|_| "Monitor".to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn start(
        monitor_index: usize,
        packet_tx: mpsc::Sender<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let monitors = Monitor::enumerate()?;
        let monitor = monitors
            .into_iter()
            .nth(monitor_index)
            .ok_or("Monitor não encontrado")?;

        let is_sharing = Arc::new(AtomicBool::new(true));
        let stop_flag = Arc::new(AtomicBool::new(false));

        let flags = Arc::new(CaptureFlags {
            is_sharing: is_sharing.clone(),
            stop_flag: stop_flag.clone(),
            packet_tx,
            stream_id: fastrand::u32(1..),
            frame_counter: std::sync::atomic::AtomicU32::new(1),
            last_frame_time: std::sync::Mutex::new(Instant::now()),
            temp_buf: std::sync::Mutex::new(Vec::new()),
        });

        let settings = Settings::new(
            monitor,
            CursorCaptureSettings::Default,
            DrawBorderSettings::Default,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Rgba8,
            flags,
        );

        // start() blocks while capturing, so only an early failure arrives here
        let (error_tx, error_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            if let Err(e) = CaptureHandler::start(settings) {
                let _ = error_tx.send(format!("{:?}", e));
            }
        });

        if let Ok(e) = error_rx.recv_timeout(Duration::from_millis(500)) {
            return Err(e.into());
        }

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
