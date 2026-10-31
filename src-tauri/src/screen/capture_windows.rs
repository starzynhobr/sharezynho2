use std::sync::Arc;
use std::time::Duration;
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

use super::capture::CaptureFlags;

struct CaptureHandler {
    flags: Arc<CaptureFlags>,
    temp_buf: Vec<u8>,
}

impl GraphicsCaptureApiHandler for CaptureHandler {
    type Flags = Arc<CaptureFlags>;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self {
            flags: ctx.flags,
            temp_buf: Vec::new(),
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        if self.flags.should_stop() {
            capture_control.stop();
            return Ok(());
        }

        if !self.flags.ready_for_frame() {
            return Ok(());
        }

        let width = frame.width() as usize;
        let height = frame.height() as usize;
        let buffer = match frame.buffer() {
            Ok(b) => b,
            Err(_) => return Ok(()),
        };

        let raw_pixels = buffer.as_nopadding_buffer(&mut self.temp_buf);
        self.flags.send_rgba(raw_pixels, width, height);
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn monitor_names() -> Vec<String> {
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
    flags: Arc<CaptureFlags>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let monitors = Monitor::enumerate()?;
    let monitor = monitors
        .into_iter()
        .nth(monitor_index)
        .ok_or("Monitor não encontrado")?;

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
    Ok(())
}
