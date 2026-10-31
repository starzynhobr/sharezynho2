use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub type BufferPusher = Arc<dyn Fn(u32, &[u8]) + Send + Sync>;

pub struct AudioPlayback {
    pub is_deafened: Arc<AtomicBool>,
    buffers: Arc<Mutex<HashMap<u32, VecDeque<f32>>>>,
    _stream: cpal::Stream,
}

impl AudioPlayback {
    pub fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("Nenhum dispositivo de saída de áudio encontrado")?;

        let config = device.default_output_config()?;
        let channels = config.channels() as usize;

        let is_deafened = Arc::new(AtomicBool::new(false));
        let deafened_clone = is_deafened.clone();

        let buffers = Arc::new(Mutex::new(HashMap::<u32, VecDeque<f32>>::new()));
        let buf_clone = buffers.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let err_fn = |err| eprintln!("Erro no stream de saída de áudio: {}", err);
                device.build_output_stream(
                    config.into(),
                    move |data: &mut [f32], _: &_| {
                        if deafened_clone.load(Ordering::Relaxed) {
                            data.fill(0.0);
                            return;
                        }

                        let mut queues = buf_clone.lock().unwrap();
                        for frame in data.chunks_mut(channels) {
                            // Mix one sample from each active speaker
                            let sample: f32 = queues
                                .values_mut()
                                .map(|q| q.pop_front().unwrap_or(0.0))
                                .sum();
                            for ch in frame.iter_mut() {
                                *ch = sample.clamp(-1.0, 1.0);
                            }
                        }
                        queues.retain(|_, q| !q.is_empty());
                    },
                    err_fn,
                    None,
                )?
            }
            _ => return Err("Formato de saída de áudio não suportado (apenas F32)".into()),
        };

        stream.play()?;

        Ok(Self {
            is_deafened,
            buffers,
            _stream: stream,
        })
    }

    /// Returns a closure that enqueues a speaker's 16kHz mono 16-bit PCM samples,
    /// upsampling to 48kHz for playback.
    pub fn clone_buffer_pusher(&self) -> BufferPusher {
        let buffers = self.buffers.clone();
        let is_deafened = self.is_deafened.clone();
        Arc::new(move |sender_id: u32, raw_bytes: &[u8]| {
            if is_deafened.load(Ordering::Relaxed) {
                return;
            }
            let mut samples_48k = Vec::with_capacity(raw_bytes.len() / 2 * 3);
            for chunk in raw_bytes.as_chunks::<2>().0 {
                let s_i16 = i16::from_le_bytes([chunk[0], chunk[1]]);
                let s_f32 = (s_i16 as f32) / 32768.0;
                samples_48k.push(s_f32);
                samples_48k.push(s_f32);
                samples_48k.push(s_f32);
            }
            let mut queues = buffers.lock().unwrap();
            let q = queues.entry(sender_id).or_default();
            // Limit queue length to prevent unbounded lag
            let len = q.len();
            if len > 14400 {
                q.drain(..len - 7200);
            }
            q.extend(samples_48k);
        })
    }
}
