use std::convert::TryInto;

// WAV data to samples.
pub fn bytes_to_samples(data: Vec<u8>) -> Vec<f32> {
    if data.len() % 2 != 0 {
        panic!("Invalid WAV data");
    }

    let mut samples = Vec::new();
    for i in 0..data.len() / 2 {
        let sample = i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
        samples.push(sample as f32 / i16::MAX as f32); // sample / 32768.0
    }

    samples
}
