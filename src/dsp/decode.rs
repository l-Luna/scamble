//! Utilities for decoding samples from a plugin, for data inputs.

use hound::{SampleFormat, WavReader};

/// Decode a data buffer into mono samples, guessing the correct format.
pub fn decode_into(data: &[u8], output: &mut Vec<f32>) -> Option<()> {
    let mut reader = WavReader::new(data).ok()?;
    let channels = reader.spec().channels;
    match reader.spec().sample_format {
        SampleFormat::Float => {
            for f in reader.samples::<f32>().step_by(channels as usize) {
                output.push(f.ok()?)
            }
        }
        SampleFormat::Int => {
            let max = 2usize.pow(reader.spec().bits_per_sample as u32) as f64;
            for f in reader.samples::<i32>().step_by(channels as usize) {
                output.push((f.ok()? as f64 / max) as f32)
            }
        }
    }
    Some(())
}