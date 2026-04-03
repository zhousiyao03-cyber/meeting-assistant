use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use log::info;

use super::buffer::SharedBuffer;

/// Lists available audio input devices (microphones + virtual devices like BlackHole).
pub fn list_input_devices() -> Result<Vec<(String, String)>> {
    let host = cpal::default_host();
    let devices: Vec<(String, String)> = host
        .input_devices()?
        .filter_map(|d| {
            let name = d.name().ok()?;
            Some((name.clone(), name))
        })
        .collect();
    Ok(devices)
}

/// Start capturing from a named input device, resampling to 16kHz mono f32.
pub fn start_capture(device_name: &str, buffer: SharedBuffer) -> Result<Stream> {
    let host = cpal::default_host();
    let device = host
        .input_devices()?
        .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
        .ok_or_else(|| anyhow!("Device '{}' not found", device_name))?;

    let config = device.default_input_config()?;
    info!(
        "Capturing from '{}': {} Hz, {} channels, {:?}",
        device_name,
        config.sample_rate().0,
        config.channels(),
        config.sample_format()
    );

    let source_rate = config.sample_rate().0 as f64;
    let target_rate = 16000.0;
    let channels = config.channels() as usize;

    let stream_config: StreamConfig = config.clone().into();

    let buf = buffer.clone();
    let stream = device.build_input_stream(
        &stream_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // Convert to mono and resample to 16kHz
            let mono: Vec<f32> = data
                .chunks(channels)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                .collect();

            // Simple linear resampling (good enough for speech)
            let ratio = target_rate / source_rate;
            let resampled_len = (mono.len() as f64 * ratio) as usize;
            let mut resampled = Vec::with_capacity(resampled_len);
            for i in 0..resampled_len {
                let src_idx = i as f64 / ratio;
                let idx = src_idx as usize;
                let frac = src_idx - idx as f64;
                let sample = if idx + 1 < mono.len() {
                    mono[idx] * (1.0 - frac as f32) + mono[idx + 1] * frac as f32
                } else if idx < mono.len() {
                    mono[idx]
                } else {
                    0.0
                };
                resampled.push(sample);
            }

            if let Ok(mut b) = buf.lock() {
                b.push(&resampled);
            }
        },
        |err| {
            log::error!("Audio stream error: {}", err);
        },
        None,
    )?;

    stream.play()?;
    Ok(stream)
}
