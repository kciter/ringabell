use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;

/// 해밍 윈도우를 적용해 각 프레임을 생성
fn hamming_window(size: usize) -> Vec<f32> {
    (0..size).map(|i| 0.54 - 0.46 * ((2.0 * PI * i as f32) / (size as f32 - 1.0)).cos()).collect()
}

/// 간단한 저역 통과 필터 적용
fn low_pass_filter(samples: &[f32], max_freq: f32, sample_rate: f32) -> Vec<f32> {
    samples.iter().map(|&s| if s < max_freq / sample_rate { s } else { 0.0 }).collect()
}

/// 다운샘플링을 수행하여 샘플 수 줄임
fn downsample(input: &[f32], original_sample_rate: usize, target_sample_rate: usize) -> Vec<f32> {
    let ratio = original_sample_rate / target_sample_rate;
    input.iter().step_by(ratio).copied().collect()
}

/// 스펙트로그램 생성 함수
pub fn create_spectrogram(samples: Vec<f32>) -> Vec<Vec<f32>> {
    let sample_rate: usize = 44100;
    let max_freq: f32 = 5000.0;
    let freq_bin_size: usize = 1024;
    let hop_size: usize = 512;
    let dsp_ratio: usize = 4;
    
    // 저역 통과 필터 적용 및 다운샘플링
    let filtered_samples = low_pass_filter(&samples, max_freq, sample_rate as f32);
    let downsampled_samples = downsample(&filtered_samples, sample_rate, sample_rate / dsp_ratio);

    let num_windows = downsampled_samples.len() / (freq_bin_size - hop_size);
    let mut spectrogram = Vec::with_capacity(num_windows);

    // 해밍 윈도우 생성
    let window = hamming_window(freq_bin_size);

    // FFT 설정
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(freq_bin_size);

    for i in 0..num_windows {
        let start = i * hop_size;
        let end = start + freq_bin_size;
        let mut bin = vec![Complex::new(0.0, 0.0); freq_bin_size];

        // 윈도우링 적용
        for j in 0..freq_bin_size {
            if start + j < downsampled_samples.len() {
                bin[j] = Complex::new(downsampled_samples[start + j] * window[j], 0.0);
            }
        }

        // FFT 수행
        fft.process(&mut bin);

        // 스펙트럼 강도 계산 후 저장
        let spectrum_magnitudes: Vec<f32> = bin.iter().map(|c| c.norm()).collect();
        spectrogram.push(spectrum_magnitudes);
    }

    spectrogram
}

/// 스펙트로그램을 분석하여 주파수 대역에서 주요 피크를 추출하는 함수
pub fn extract_peaks(spectrogram: Vec<Vec<f32>>, audio_duration: f32) -> Vec<(usize, usize)> {
    if spectrogram.is_empty() {
        return Vec::new();
    }

    let bands = vec![
        (0, 10),
        (10, 20),
        (20, 40),
        (40, 80),
        (80, 160),
        (160, 512),
    ];

    let bin_duration = audio_duration / spectrogram.len() as f32;
    let mut peaks = Vec::new();

    for (bin_idx, bin) in spectrogram.iter().enumerate() {
        let mut max_mags = Vec::new();
        let mut max_freqs = Vec::new();
        let mut freq_indices = Vec::new();

        for (min, max) in &bands {
            let mut max_mag = 0.0;
            let mut max_freq_idx = *min;

            for freq_idx in *min..*max {
                if freq_idx < bin.len() {
                    let magnitude = bin[freq_idx];
                    if magnitude > max_mag {
                        max_mag = magnitude;
                        max_freq_idx = freq_idx;
                    }
                }
            }

            if max_mag > 0.0 {
                max_mags.push(max_mag);
                max_freqs.push(Complex::new(bin[max_freq_idx], 0.0));
                freq_indices.push(max_freq_idx as f32);
            }
        }

        // 평균 진폭 계산
        let avg_magnitude = max_mags.iter().sum::<f32>() / max_mags.len() as f32;

        // 평균보다 큰 피크만 추출하여 추가
        for (i, &value) in max_mags.iter().enumerate() {
            if value > avg_magnitude {
                let peak_time_in_bin = freq_indices[i] * bin_duration / bin.len() as f32;
                let peak_time_index = (bin_idx as f32 * bin_duration + peak_time_in_bin) as usize;

                // (시간 인덱스, 주파수 인덱스) 튜플을 추가
                peaks.push((peak_time_index, freq_indices[i] as usize));
            }
        }
    }

    peaks
}
