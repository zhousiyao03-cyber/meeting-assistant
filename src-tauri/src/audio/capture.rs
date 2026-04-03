use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};

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

fn resample_mono(data: &[f32], channels: usize, source_rate: f64, target_rate: f64) -> Vec<f32> {
    // Convert to mono
    let mono: Vec<f32> = data
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect();

    // Resample if needed
    if (source_rate - target_rate).abs() < 1.0 {
        return mono;
    }

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
    resampled
}

/// Start capturing from a named input device, resampling to 16kHz mono f32.
pub fn start_capture(device_name: &str, buffer: SharedBuffer) -> Result<Stream> {
    let host = cpal::default_host();
    let device = host
        .input_devices()?
        .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
        .ok_or_else(|| anyhow!("Device '{}' not found", device_name))?;

    let config = device.default_input_config()?;
    let sample_format = config.sample_format();
    let source_rate = config.sample_rate().0 as f64;
    let target_rate = 16000.0;
    let channels = config.channels() as usize;
    let stream_config: StreamConfig = config.into();

    eprintln!(
        "[audio] Capturing from '{}': {:.0} Hz, {} ch, {:?}",
        device_name, source_rate, channels, sample_format
    );

    let stream = match sample_format {
        SampleFormat::F32 => {
            let buf = buffer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let resampled = resample_mono(data, channels, source_rate, target_rate);
                    if let Ok(mut b) = buf.lock() {
                        b.push(&resampled);
                    }
                },
                |err| eprintln!("[audio] Stream error (f32): {}", err),
                None,
            )?
        }
        SampleFormat::I16 => {
            let buf = buffer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    // Convert i16 to f32 [-1.0, 1.0]
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    let resampled = resample_mono(&f32_data, channels, source_rate, target_rate);
                    if let Ok(mut b) = buf.lock() {
                        b.push(&resampled);
                    }
                },
                |err| eprintln!("[audio] Stream error (i16): {}", err),
                None,
            )?
        }
        SampleFormat::I32 => {
            let buf = buffer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[i32], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 2147483648.0).collect();
                    let resampled = resample_mono(&f32_data, channels, source_rate, target_rate);
                    if let Ok(mut b) = buf.lock() {
                        b.push(&resampled);
                    }
                },
                |err| eprintln!("[audio] Stream error (i32): {}", err),
                None,
            )?
        }
        _ => return Err(anyhow!("Unsupported sample format: {:?}", sample_format)),
    };

    stream.play()?;
    Ok(stream)
}
