use serde::Deserialize;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Fastest first; the last two run on the CPU when no GPU encoder exists.
const ENCODERS: &[(&str, &[&str])] = &[
    (
        "h264_nvenc",
        &[
            "-preset",
            "p3",
            "-tune",
            "ull",
            "-rc",
            "cbr",
            "-zerolatency",
            "1",
        ],
    ),
    ("h264_amf", &["-usage", "ultralowlatency", "-rc", "cbr"]),
    ("h264_qsv", &["-preset", "veryfast"]),
    ("h264_mf", &["-rate_control", "cbr"]),
    ("libopenh264", &[]),
];

// A static screen produces no captures, so the last one repeats to keep the stream alive
const KEEPALIVE_INTERVAL: Duration = Duration::from_millis(200);
const ACCESS_UNIT_DELIMITER: [u8; 5] = [0, 0, 0, 1, 9];

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct VideoQuality {
    pub height: u32,
    pub fps: u32,
}

impl VideoQuality {
    fn bitrate(&self) -> u32 {
        let base = match self.height {
            0..=480 => 2_500_000,
            481..=720 => 5_000_000,
            _ => 8_000_000,
        };
        if self.fps > 30 { base * 3 / 2 } else { base }
    }
}

pub struct H264Encoder {
    width: usize,
    height: usize,
    frame_tx: SyncSender<Vec<u8>>,
    child: Child,
}

fn ffmpeg_path() -> PathBuf {
    let dev_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries/ffmpeg.exe");
    let beside_exe = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("ffmpeg.exe")));
    beside_exe
        .filter(|path| path.is_file())
        .or_else(|| dev_binary.is_file().then_some(dev_binary))
        .unwrap_or_else(|| PathBuf::from("ffmpeg"))
}

fn ffmpeg_command() -> Command {
    let mut command = Command::new(ffmpeg_path());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/// Picks the first encoder this machine can really open, once per run.
fn selected_encoder() -> Option<&'static (&'static str, &'static [&'static str])> {
    static SELECTED: OnceLock<Option<&'static (&'static str, &'static [&'static str])>> =
        OnceLock::new();
    *SELECTED.get_or_init(|| {
        ENCODERS.iter().find(|(name, _)| {
            ffmpeg_command()
                .args(["-hide_banner", "-loglevel", "quiet", "-f", "lavfi", "-i"])
                .args(["color=black:s=640x480:r=30", "-frames:v", "5", "-c:v", name])
                .args(["-f", "null", "-"])
                .stdin(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        })
    })
}

impl H264Encoder {
    pub fn start(
        width: usize,
        height: usize,
        quality: VideoQuality,
        stop_flag: Arc<AtomicBool>,
        mut on_frame: impl FnMut(Vec<u8>) + Send + 'static,
    ) -> Result<Self, String> {
        let (name, options) = selected_encoder().ok_or("Nenhum encoder H.264 disponível")?;
        let bitrate = quality.bitrate().to_string();
        let fps = quality.fps.to_string();

        let mut child = ffmpeg_command()
            .args(["-hide_banner", "-loglevel", "error", "-fflags", "nobuffer"])
            .args(["-f", "rawvideo", "-pix_fmt", "rgba"])
            .args([
                "-video_size",
                &format!("{width}x{height}"),
                "-framerate",
                &fps,
            ])
            .args(["-i", "pipe:0", "-an"])
            .args([
                "-vf",
                &format!("scale=-2:'min({},ih)',format=yuv420p", quality.height),
            ])
            .args(["-c:v", name])
            .args(*options)
            .args(["-b:v", &bitrate, "-maxrate", &bitrate, "-bufsize", &bitrate])
            .args(["-g", &fps, "-bf", "0"])
            .args(["-bsf:v", "h264_metadata=aud=insert"])
            .args(["-flush_packets", "1", "-f", "h264", "pipe:1"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Não foi possível abrir o FFmpeg: {}", e))?;

        let mut stdin = child.stdin.take().ok_or("FFmpeg sem entrada")?;
        let mut stdout = child.stdout.take().ok_or("FFmpeg sem saída")?;

        let (frame_tx, frame_rx) = sync_channel::<Vec<u8>>(1);
        std::thread::spawn(move || write_frames(&mut stdin, frame_rx, stop_flag));
        std::thread::spawn(move || read_access_units(&mut stdout, &mut on_frame));

        Ok(Self {
            width,
            height,
            frame_tx,
            child,
        })
    }

    pub fn matches(&self, width: usize, height: usize) -> bool {
        self.width == width && self.height == height
    }

    /// Drops the frame when the encoder is busy, so latency never builds up.
    pub fn push_frame(&self, pixels: Vec<u8>) {
        let _ = self.frame_tx.try_send(pixels);
    }
}

impl Drop for H264Encoder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn write_frames(stdin: &mut impl Write, frames: Receiver<Vec<u8>>, stop_flag: Arc<AtomicBool>) {
    let mut last_frame = Vec::new();
    while !stop_flag.load(Ordering::Relaxed) {
        match frames.recv_timeout(KEEPALIVE_INTERVAL) {
            Ok(frame) => last_frame = frame,
            Err(RecvTimeoutError::Timeout) if !last_frame.is_empty() => {}
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return,
        }
        if stdin.write_all(&last_frame).is_err() {
            return;
        }
    }
}

fn read_access_units(stdout: &mut impl Read, on_frame: &mut impl FnMut(Vec<u8>)) {
    let mut chunk = vec![0u8; 64 * 1024];
    let mut buffer = Vec::new();
    while let Ok(read) = stdout.read(&mut chunk) {
        if read == 0 {
            return;
        }
        buffer.extend_from_slice(&chunk[..read]);
        for frame in drain_access_units(&mut buffer) {
            on_frame(frame);
        }
    }
}

/// Removes every complete frame; a frame ends where the next delimiter starts.
fn drain_access_units(buffer: &mut Vec<u8>) -> Vec<Vec<u8>> {
    let mut starts: Vec<usize> = buffer
        .windows(ACCESS_UNIT_DELIMITER.len())
        .enumerate()
        .filter(|(_, window)| *window == ACCESS_UNIT_DELIMITER)
        .map(|(index, _)| index)
        .collect();
    let Some(last) = starts.pop() else {
        return Vec::new();
    };
    let rest = buffer.split_off(last);
    let complete = std::mem::replace(buffer, rest);
    let mut frames = Vec::with_capacity(starts.len());
    let mut bounds = starts.into_iter().chain([complete.len()]).peekable();
    let mut begin = bounds.next().unwrap_or(0);
    if begin == complete.len() {
        begin = 0;
    }
    for end in bounds {
        frames.push(complete[begin..end].to_vec());
        begin = end;
    }
    if frames.is_empty() && !complete.is_empty() {
        frames.push(complete);
    }
    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_stream_on_access_unit_delimiters() {
        let mut buffer = vec![
            0, 0, 0, 1, 9, 0xAA, 0, 0, 0, 1, 9, 0xBB, 0, 0, 0, 1, 9, 0xCC,
        ];
        let frames = drain_access_units(&mut buffer);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], vec![0, 0, 0, 1, 9, 0xAA]);
        assert_eq!(frames[1], vec![0, 0, 0, 1, 9, 0xBB]);
        assert_eq!(buffer, vec![0, 0, 0, 1, 9, 0xCC]);
    }
}
