use sonora::config::{EchoCanceller, GainController2, NoiseSuppression};
use sonora::{AudioProcessing, Config, StreamConfig};
use std::sync::{Arc, Mutex};

pub const PROCESSING_SAMPLE_RATE: u32 = 16000;
const FRAME_SAMPLES: usize = 160;

pub type SharedProcessor = Arc<Mutex<VoiceProcessor>>;

/// Echo cancellation, noise suppression and gain control for 16kHz mono voice.
pub struct VoiceProcessor {
    apm: AudioProcessing,
}

impl VoiceProcessor {
    pub fn new() -> Self {
        let config = Config {
            echo_canceller: Some(EchoCanceller::default()),
            noise_suppression: Some(NoiseSuppression::default()),
            gain_controller2: Some(GainController2::default()),
            ..Default::default()
        };
        let stream = StreamConfig::new(PROCESSING_SAMPLE_RATE as _, 1);
        Self {
            apm: AudioProcessing::builder()
                .config(config)
                .capture_config(stream)
                .render_config(stream)
                .build(),
        }
    }

    pub fn shared() -> SharedProcessor {
        Arc::new(Mutex::new(Self::new()))
    }

    /// Cleans microphone samples in place, in 10ms frames.
    pub fn process_capture(&mut self, samples: &mut [i16]) {
        for frame in samples.as_chunks_mut::<FRAME_SAMPLES>().0 {
            let source = *frame;
            let _ = self.apm.process_capture_i16(&source, frame);
        }
    }

    /// Feeds the audio being played, which the echo canceller removes from the mic.
    pub fn process_render(&mut self, samples: &[i16]) {
        let mut output = [0i16; FRAME_SAMPLES];
        for frame in samples.as_chunks::<FRAME_SAMPLES>().0 {
            let _ = self.apm.process_render_i16(frame, &mut output);
        }
    }
}

impl Default for VoiceProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn energy(samples: &[i16]) -> f64 {
        samples.iter().map(|s| (*s as f64).powi(2)).sum()
    }

    #[test]
    fn played_audio_is_removed_from_the_microphone() {
        let mut processor = VoiceProcessor::new();
        let mut first = 0.0;
        let mut last = 0.0;

        // Speech-like noise bursts played back and captured 30ms later
        let mut history = vec![vec![0i16; FRAME_SAMPLES]; 3];
        for frame_index in 0..1000 {
            let active = frame_index % 40 < 25;
            let played: Vec<i16> = (0..FRAME_SAMPLES)
                .map(|_| {
                    if active {
                        fastrand::i16(-9000..9000)
                    } else {
                        0
                    }
                })
                .collect();
            processor.process_render(&played);
            history.push(played);
            let mut captured: Vec<i16> = history.remove(0).iter().map(|s| s / 2).collect();
            processor.process_capture(&mut captured);
            if (100..300).contains(&frame_index) {
                first += energy(&captured);
            }
            if frame_index >= 800 {
                last += energy(&captured);
            }
        }

        assert!(last < first / 4.0, "first={first} last={last}");
    }
}
