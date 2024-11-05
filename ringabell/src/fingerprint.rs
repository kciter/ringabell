use std::collections::HashMap;
use serde_json::json;

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

pub fn search_fingerprints(registered: Vec<(String, Vec<u64>)>, fingerprints: Vec<u64>) -> String {
    let mut score_map: HashMap<usize, usize> = HashMap::new();

    // 저장된 각 노래와 비교하여 일치도 계산
    for (i, (_, existing_fingerprints)) in registered.iter().enumerate() {
        for fingerprint in &fingerprints {
            if existing_fingerprints.contains(fingerprint) {
                *score_map.entry(i).or_insert(0) += 1;
            }
        }
    }

    // 최고 점수를 가진 항목 찾기
    let (best_index, best_score) = score_map.iter().max_by_key(|&(_, count)| count).map(|(&i, &count)| (i, count)).unwrap_or((0, 0));

    // 일정 임계값 이상일 때만 결과 반환, 그렇지 않으면 "Not found" 반환
    const MIN_SCORE_THRESHOLD: usize = 5;
    if best_score >= MIN_SCORE_THRESHOLD {
        json!({
            "songName": registered[best_index].0.clone(),
            "score": best_score
        }).to_string()
    } else {
        json!({
            "songName": "Not found",
            "score": 0
        }).to_string()
    }
}
