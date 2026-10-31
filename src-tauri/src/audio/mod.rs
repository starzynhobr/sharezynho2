pub mod capture;
pub mod playback;

pub use capture::AudioCapture;
pub use playback::AudioPlayback;

use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::Mutex;

static PREFERRED_INPUT: Mutex<Option<String>> = Mutex::new(None);
static PREFERRED_OUTPUT: Mutex<Option<String>> = Mutex::new(None);

/// Sets the devices used by the next session; None means the system default.
pub fn set_preferred_devices(input: Option<String>, output: Option<String>) {
    *PREFERRED_INPUT.lock().unwrap() = input;
    *PREFERRED_OUTPUT.lock().unwrap() = output;
}

pub fn input_device_names() -> Vec<String> {
    cpal::default_host()
        .input_devices()
        .map(|devices| devices.filter_map(|d| device_name(&d)).collect())
        .unwrap_or_default()
}

pub fn output_device_names() -> Vec<String> {
    cpal::default_host()
        .output_devices()
        .map(|devices| devices.filter_map(|d| device_name(&d)).collect())
        .unwrap_or_default()
}

fn selected_input_device() -> Option<cpal::Device> {
    let host = cpal::default_host();
    let preferred = PREFERRED_INPUT.lock().unwrap().clone();
    preferred
        .and_then(|name| {
            host.input_devices()
                .ok()?
                .find(|d| device_name(d).as_deref() == Some(name.as_str()))
        })
        .or_else(|| host.default_input_device())
}

fn selected_output_device() -> Option<cpal::Device> {
    let host = cpal::default_host();
    let preferred = PREFERRED_OUTPUT.lock().unwrap().clone();
    preferred
        .and_then(|name| {
            host.output_devices()
                .ok()?
                .find(|d| device_name(d).as_deref() == Some(name.as_str()))
        })
        .or_else(|| host.default_output_device())
}

fn device_name(device: &cpal::Device) -> Option<String> {
    device.description().ok().map(|d| d.name().to_string())
}
