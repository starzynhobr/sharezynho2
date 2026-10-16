use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct AudioCapture {
    pub is_muted: Arc<AtomicBool>,
    _stream: cpal::Stream,
}

impl AudioCapture {
    pub fn start(
        audio_packet_tx: mpsc::Sender<Vec<u8>>,
        speaking_status_tx: mpsc::Sender<bool>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("Nenhum dispositivo de entrada de áudio encontrado")?;

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;

        let is_muted = Arc::new(AtomicBool::new(false));
        let muted_clone = is_muted.clone();

        // Accumulator for 20ms of audio at 16kHz
        let mut sample_accumulator = Vec::<i16>::with_capacity(320);
        let mut last_speaking_time = Instant::now() - Duration::from_secs(10);
        let mut was_speaking = false;

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let err_fn = |err| eprintln!("Erro no stream de microfone: {}", err);
                device.build_input_stream(
                    config.into(),
                    move |data: &[f32], _: &_| {
                        if muted_clone.load(Ordering::Relaxed) {
                            if was_speaking {
                                was_speaking = false;
                                let _ = speaking_status_tx.try_send(false);
                            }
                            return;
                        }

                        // Convert multi-channel input to mono f32
                        let mono_samples: Vec<f32> = data
                            .chunks(channels)
                            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                            .collect();

                        // Downsample if sample_rate is ~48000 to ~16000
                        let step = (sample_rate / 16000).max(1) as usize;
                        for chunk in mono_samples.chunks(step) {
                            let avg: f32 = chunk.iter().sum::<f32>() / chunk.len() as f32;
                            // Clamp and convert to i16
                            let clamped = (avg.clamp(-1.0, 1.0) * 32767.0) as i16;
                            sample_accumulator.push(clamped);

                            // When we reach ~320 samples (20ms at 16kHz)
                            if sample_accumulator.len() >= 320 {
                                let mut sum_sq = 0.0f64;
                                for &s in &sample_accumulator {
                                    let norm = s as f64 / 32768.0;
                                    sum_sq += norm * norm;
                                }
                                let rms = (sum_sq / sample_accumulator.len() as f64).sqrt();

                                let is_loud = rms > 0.015;
                                if is_loud {
                                    last_speaking_time = Instant::now();
                                    if !was_speaking {
                                        was_speaking = true;
                                        let _ = speaking_status_tx.try_send(true);
                                    }
                                } else if was_speaking
                                    && last_speaking_time.elapsed() > Duration::from_millis(350)
                                {
                                    was_speaking = false;
                                    let _ = speaking_status_tx.try_send(false);
                                }

                                if is_loud
                                    || last_speaking_time.elapsed() < Duration::from_millis(300)
                                {
                                    // Pack as little-endian bytes
                                    let mut packet =
                                        Vec::with_capacity(sample_accumulator.len() * 2);
                                    for &s in &sample_accumulator {
                                        packet.extend_from_slice(&s.to_le_bytes());
                                    }
                                    let _ = audio_packet_tx.try_send(packet);
                                }

                                sample_accumulator.clear();
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            _ => return Err("Formato de áudio não suportado (apenas F32)".into()),
        };

        stream.play()?;

        Ok(Self {
            is_muted,
            _stream: stream,
        })
    }
}
