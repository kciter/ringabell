mod utils;
mod wav;
mod spectrogram;
mod fingerprint;

use wasm_bindgen::prelude::*;
use fingerprint::{create_fingerprints, search_fingerprints};
use spectrogram::{create_spectrogram, extract_peaks};
use wav::{bytes_to_samples, get_wav_duration_from_bytes};

static mut MUSIC_FINGERPRINTS: Vec<(String, Vec<u64>)> = Vec::new();

#[wasm_bindgen]
pub async fn register(name: String, data: Vec<u8>) {
    // 1. get duration and samples from the wav data
    let duration = get_wav_duration_from_bytes(&data);
    let samples = bytes_to_samples(data);

    // 2. create a spectrogram from the samples
    let spectrogram = create_spectrogram(samples);
    
    // 3. extract peaks from the spectrogram
    let peaks = extract_peaks(spectrogram, duration as f32);

    // 4. create a fingerprints from the peaks
    let fingerprints = create_fingerprints(peaks);

    // 5. save the fingerprints to memory
    unsafe {
        MUSIC_FINGERPRINTS.push((name, fingerprints));
    }
}

#[wasm_bindgen]
pub async fn search(data: Vec<u8>) -> String {
    // 1. get duratino and samples from the wav data
    let duration = get_wav_duration_from_bytes(&data);
    let samples = bytes_to_samples(data);

    // 2. create a spectrogram from the samples
    let spectrogram = create_spectrogram(samples);

    // 3. extract peaks from the spectrogram
    let peaks = extract_peaks(spectrogram, duration as f32);

    // 4. create a fingerprints from the peaks
    let fingerprints = create_fingerprints(peaks);

    // 5. search the fingerprints in memory and return the best match
    unsafe {
        search_fingerprints(MUSIC_FINGERPRINTS.clone(), fingerprints)
    }
}
