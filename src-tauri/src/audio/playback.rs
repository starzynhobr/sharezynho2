use cpal::traits::{DeviceTrait, StreamTrait};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::processing::{PROCESSING_SAMPLE_RATE, SharedProcessor};

pub type BufferPusher = Arc<dyn Fn(u32, &[u8]) + Send + Sync>;

pub struct AudioPlayback {
    pub is_deafened: Arc<AtomicBool>,
    output_rate: u32,
    buffers: Arc<Mutex<HashMap<u32, VecDeque<f32>>>>,
    _stream: cpal::Stream,
}

/// Sums one sample from each active speaker.
fn mix_next_sample(queues: &mut HashMap<u32, VecDeque<f32>>) -> f32 {
    queues
        .values_mut()
        .map(|q| q.pop_front().unwrap_or(0.0))
        .sum::<f32>()
        .clamp(-1.0, 1.0)
}

/// Converts 16kHz PCM16 bytes to the output rate with linear interpolation.
fn decode_and_resample(raw_bytes: &[u8], output_rate: usize) -> Vec<f32> {
    let samples: Vec<f32> = raw_bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|chunk| i16::from_le_bytes(*chunk) as f32 / 32768.0)
        .collect();
    if samples.is_empty() {
        return Vec::new();
    }
    let output_len = samples.len() * output_rate / PROCESSING_SAMPLE_RATE as usize;
    (0..output_len)
        .map(|i| {
            let position = i as f32 * PROCESSING_SAMPLE_RATE as f32 / output_rate as f32;
            let index = position as usize;
            let current = samples[index.min(samples.len() - 1)];
            let next = samples[(index + 1).min(samples.len() - 1)];
            current + (next - current) * position.fract()
        })
        .collect()
}

impl AudioPlayback {
    pub fn start(
        processor: SharedProcessor,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let device = super::selected_output_device()
            .ok_or("Nenhum dispositivo de saída de áudio encontrado")?;

        let config = device.default_output_config()?;
        let channels = config.channels() as usize;
        let output_rate = config.sample_rate();

        let is_deafened = Arc::new(AtomicBool::new(false));
        let deafened_clone = is_deafened.clone();

        let buffers = Arc::new(Mutex::new(HashMap::<u32, VecDeque<f32>>::new()));
        let buf_clone = buffers.clone();

        // Reference signal for the echo canceller, resampled to 16kHz
        let mut reference = Vec::<i16>::with_capacity(160);
        let mut reference_sum = 0.0f32;
        let mut reference_count = 0u32;
        let mut reference_phase = 0u32;

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
                            let sample = mix_next_sample(&mut queues);
                            frame.fill(sample);

                            reference_sum += sample;
                            reference_count += 1;
                            reference_phase += PROCESSING_SAMPLE_RATE;
                            if reference_phase >= output_rate {
                                reference_phase -= output_rate;
                                let average = reference_sum / reference_count as f32;
                                reference.push((average * 32767.0) as i16);
                                reference_sum = 0.0;
                                reference_count = 0;
                            }
                            if reference.len() >= 160 {
                                processor.lock().unwrap().process_render(&reference);
                                reference.clear();
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
            output_rate,
            buffers,
            _stream: stream,
        })
    }

    /// Returns a closure that enqueues a speaker's 16kHz mono 16-bit PCM samples,
    /// resampling to the output device rate.
    pub fn clone_buffer_pusher(&self) -> BufferPusher {
        let buffers = self.buffers.clone();
        let is_deafened = self.is_deafened.clone();
        let output_rate = self.output_rate as usize;
        Arc::new(move |sender_id: u32, raw_bytes: &[u8]| {
            if is_deafened.load(Ordering::Relaxed) {
                return;
            }
            let resampled = decode_and_resample(raw_bytes, output_rate);
            let mut queues = buffers.lock().unwrap();
            let q = queues.entry(sender_id).or_default();
            // Limit queue length to prevent unbounded lag
            let len = q.len();
            if len > output_rate * 3 / 10 {
                q.drain(..len - output_rate * 15 / 100);
            }
            q.extend(resampled);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simultaneous_speakers_are_summed() {
        let mut queues = HashMap::from([
            (1, VecDeque::from([0.25, 0.5])),
            (2, VecDeque::from([0.25])),
        ]);
        assert_eq!(mix_next_sample(&mut queues), 0.5);
        assert_eq!(mix_next_sample(&mut queues), 0.5);
    }

    #[test]
    fn mix_is_limited_to_full_scale() {
        let mut queues = HashMap::from([(1, VecDeque::from([0.8])), (2, VecDeque::from([0.8]))]);
        assert_eq!(mix_next_sample(&mut queues), 1.0);
    }

    #[test]
    fn resampling_keeps_duration_at_any_output_rate() {
        let packet = vec![0u8; 640];
        assert_eq!(decode_and_resample(&packet, 48000).len(), 960);
        assert_eq!(decode_and_resample(&packet, 44100).len(), 882);
    }
}
