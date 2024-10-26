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

pub fn get_wav_duration_from_bytes(bytes: &[u8]) -> f32 {
    // WAV 파일 헤더 검사
    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        eprintln!("Invalid WAV data");
        return 0.0; // 에러 발생 시 기본값 반환
    }

    let mut cursor = 12;
    let mut sample_rate = None;
    let mut num_samples = None;

    while cursor < bytes.len() {
        let chunk_id = &bytes[cursor..cursor + 4];
        let chunk_size = u32::from_le_bytes(match bytes[cursor + 4..cursor + 8].try_into() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Error reading chunk size");
                return 0.0; // 에러 발생 시 기본값 반환
            }
        });
        cursor += 8;

        if chunk_id == b"fmt " {
            sample_rate = Some(u32::from_le_bytes(match bytes[cursor + 4..cursor + 8].try_into() {
                Ok(val) => val,
                Err(_) => {
                    eprintln!("Error reading sample rate");
                    return 0.0; // 에러 발생 시 기본값 반환
                }
            }));
        } else if chunk_id == b"data" {
            // 샘플 수 계산
            if let Some(rate) = sample_rate {
                let byte_rate = rate as f64;
                num_samples = Some(chunk_size as f64 / (byte_rate / 8.0)); // 샘플 수 계산
            } else {
                eprintln!("Sample rate not found");
                return 0.0; // 에러 발생 시 기본값 반환
            }
            break;
        }

        cursor += chunk_size as usize;
    }

    // 재생 시간 계산, 에러 발생 시 기본값 반환
    match (num_samples, sample_rate) {
        (Some(samples), Some(rate)) => (samples as f32) / rate as f32,
        _ => {
            eprintln!("Data chunk or sample rate not found");
            0.0 // 에러 발생 시 기본값 반환
        }
    }
}
