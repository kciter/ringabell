mod utils;

use wasm_bindgen::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

static mut FINGERPRINTS: Vec<(String, Vec<u64>)> = Vec::new();

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn register(name: String, data: Vec<u8>) {
    // 1. load the data into a wav file
    // 2. get samples from the wav file
    let samples = bytes_to_samples(data);
    console_log!("samples");

    // 3. create a spectogram from the samples
    let spectogram = create_spectogram(samples);
    console_log!("spectogram");
    
    // 4. extract peaks from the spectogram
    let peaks = extract_peaks(spectogram);
    console_log!("peaks");

    // 5. create a fingerprints from the peaks
    let fingerprints = create_fingerprints(peaks);
    console_log!("fingerprints");

    // 6. save the fingerprints to memory
    unsafe {
        FINGERPRINTS.push((name, fingerprints));
    }
}

#[wasm_bindgen]
pub fn search(data: Vec<u8>) -> String {
    // 1. load the data into a wav file
    // 2. get samples from the wav file
    let samples = bytes_to_samples(data);
    console_log!("samples");

    // 3. create a spectogram from the samples
    let spectogram = create_spectogram(samples);
    console_log!("spectogram");

    // 4. extract peaks from the spectogram
    let peaks = extract_peaks(spectogram);
    console_log!("peaks");

    // 5. create a fingerprints from the peaks
    let fingerprints = create_fingerprints(peaks);
    console_log!("fingerprints");

    // 6. search the fingerprints in memory and return the best match
    search_fingerprints(fingerprints)
}

//WAV data to samples.
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

// Create a spectogram from samples.
pub fn create_spectogram(samples: Vec<f32>) -> Vec<Vec<f32>> {
    let mut spectogram = Vec::new();
    let mut window = Vec::new();
    let window_size = 1024;
    let window_step = 512;
    let mut window_function = vec![0.0; window_size];

    for i in 0..window_size {
        window_function[i] = 0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / window_size as f32).cos();
    }
    for i in 0..samples.len() {
        window.push(samples[i]);
        if window.len() == window_size {
            let mut windowed = Vec::new();
            for j in 0..window_size {
                windowed.push(window[j] * window_function[j]);
            }
            let mut fft = FftPlanner::<f32>::new().plan_fft_forward(window_size);
            let mut input: Vec<Complex<f32>> = windowed.iter().map(|&x| Complex::new(x, 0.0)).collect();
            let mut output: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); window_size];
            
            // 수정된 process_with_scratch 호출
            let mut scratch = vec![Complex::new(0.0, 0.0); fft.get_inplace_scratch_len()];
            fft.process_with_scratch(&mut input, &mut scratch);
            
            let mut magnitudes = Vec::new();
            for j in 0..window_size / 2 {
                let magnitude = (input[j].re.powi(2) + input[j].im.powi(2)).sqrt();
                magnitudes.push(magnitude);
            }
            spectogram.push(magnitudes);
            window.clear();
            for j in 0..window_step {
                if i + j < samples.len() {
                    window.push(samples[i + j]);
                }
            }
        }
    }

    spectogram
}

// Extract peaks from spectogram.
pub fn extract_peaks(spectrogram: Vec<Vec<f32>>) -> Vec<(usize, usize)> {
    let mut peaks = Vec::new();
    let bands = vec![(0, 10), (10, 20), (20, 40), (40, 80), (80, 160), (160, 512)];

    for i in 0..spectrogram.len() {
        let mut max_mags = vec![0.0; bands.len()];
        let mut max_freqs = vec![0.0; bands.len()];
        let mut freq_indices = vec![0.0; bands.len()];

        for j in 0..bands.len() {
            let mut max_mag = 0.0;
            let mut max_freq = 0.0;
            let mut freq_idx = 0;

            for k in bands[j].0..bands[j].1 {
                let magnitude = spectrogram[i][k];
                if magnitude > max_mag {
                    max_mag = magnitude;
                    max_freq = k as f32;
                    freq_idx = k;
                }
            }

            max_mags[j] = max_mag;
            max_freqs[j] = max_freq;
            freq_indices[j] = freq_idx as f32;
        }

        let mut max_mags_sum = 0.0;
        for j in 0..max_mags.len() {
            max_mags_sum += max_mags[j];
        }
        let avg = max_mags_sum / max_mags.len() as f32;

        for j in 0..max_mags.len() {
            if max_mags[j] > avg {
                let peak_time_in_bin = freq_indices[j] * 1000.0 / spectrogram[i].len() as f32;
                let peak_time = i as f32 * 1000.0 + peak_time_in_bin;
                peaks.push((peak_time as usize, max_freqs[j] as usize));
            }
        }
    }

    peaks
}

// pub fn extract_peaks(spectrogram: Vec<Vec<f32>>) -> Vec<(usize, usize)> {
//     let mut peaks = Vec::new();
//     let bands = vec![(0, 10), (10, 20), (20, 40), (40, 80), (80, 160), (160, 512)];

//     for i in 0..spectrogram.len() {
//         for j in 1..spectrogram[i].len() - 1 {
//             // 임계값 이상의 값만 피크로 간주
//             if spectrogram[i][j] > spectrogram[i][j - 1]
//                 && spectrogram[i][j] > spectrogram[i][j + 1]
//                 && spectrogram[i][j] > threshold
//             {
//                 peaks.push((i, j));
//             }
//         }
//     }

//     peaks
// }

// pub fn create_fingerprints(peaks: Vec<(usize, usize)>) -> Vec<u64> {
//     let mut fingerprints = Vec::new();

//     for i in 0..peaks.len() {
//         for j in i + 1..peaks.len() {
//             let time_delta = peaks[j].0 as i64 - peaks[i].0 as i64;
//             let frequency_delta = peaks[j].1 as i64 - peaks[i].1 as i64;
//             // let fingerprint = (time_delta << 32) | frequency_delta as u64;
//             let fingerprint = ((time_delta as u64) << 32) | frequency_delta as u64;
//             fingerprints.push(fingerprint);
//         }
//     }

//     fingerprints
// }

// Create fingerprints from peaks.
pub fn create_fingerprints(peaks: Vec<(usize, usize)>) -> Vec<u64> {
    let mut fingerprints = Vec::new();
    let target_zone_size = 3;

    for i in 0..peaks.len() {
        for j in i + 1..peaks.len() {
            if j <= i + target_zone_size {
                let anchor = peaks[i];
                let target = peaks[j];
                let address = create_address(anchor, target);
                let anchor_time_ms = (anchor.0 as f32 * 1000.0) as u32;
                fingerprints.push((address as u64) << 32 | anchor_time_ms as u64);
            }
        }
    }

    fingerprints
}

pub fn create_address(anchor: (usize, usize), target: (usize, usize)) -> u32 {
    let anchor_freq = anchor.1;
    let target_freq = target.1;
    let delta_ms = ((target.0 - anchor.0) as f32 * 1000.0) as u32;

    (anchor_freq as u32) << 23 | (target_freq as u32) << 14 | delta_ms
}

// Search fingerprints in memory and return the best match.
// pub fn search_fingerprints(fingerprints: Vec<u64>) -> String {
//     let mut score_map: HashMap<usize, usize> = HashMap::new();

//     for (i, (_, existing_fingerprints)) in unsafe { FINGERPRINTS.iter().enumerate() } {
//         for fingerprint in &fingerprints {
//             if existing_fingerprints.contains(fingerprint) {
//                 *score_map.entry(i).or_insert(0) += 1;
//             }
//         }
//     }

//     let best_index = score_map.iter().max_by_key(|&(_, count)| count).map(|(&i, _)| i).unwrap_or(0);
//     unsafe { FINGERPRINTS[best_index].0.clone() }
// }


pub fn search_fingerprints(fingerprints: Vec<u64>) -> String {
    const MIN_SCORE_THRESHOLD: usize = 10;
    let mut score_map: HashMap<usize, usize> = HashMap::new();

    for (i, (_, existing_fingerprints)) in unsafe { FINGERPRINTS.iter().enumerate() } {
        for fingerprint in &fingerprints {
            if existing_fingerprints.contains(fingerprint) {
                *score_map.entry(i).or_insert(0) += 1;
            }
        }
    }

    // 최고 점수를 가진 항목 찾기
    if let Some((&best_index, &best_score)) = score_map.iter().max_by_key(|&(_, &count)| count) {
        // 스코어가 임계값 이상인지 확인
        if best_score >= MIN_SCORE_THRESHOLD {
            let result = json!({
                "songName": unsafe { FINGERPRINTS[best_index].0.clone() },
                "score": best_score
            });
            return result.to_string();
        }
    }

    // 임계값 이하일 경우 "Not found" 반환
    json!({
        "songName": "Not found",
        "score": 0
    }).to_string()
}
