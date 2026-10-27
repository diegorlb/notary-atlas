pub fn string_similarity(left: &str, right: &str) -> f64 {
    if left.is_empty() && right.is_empty() {
        return 1.0;
    }

    let base_score = strsim::jaro_winkler(left, right);

    let left_tokens: Vec<_> = left.split_whitespace().collect();
    let right_tokens: Vec<_> = right.split_whitespace().collect();

    let is_subset = if left_tokens.len() <= right_tokens.len() {
        left_tokens.iter().all(|w| right_tokens.contains(w))
    } else {
        right_tokens.iter().all(|w| left_tokens.contains(w))
    };

    if is_subset && !left_tokens.is_empty() {
        #[expect(clippy::cast_precision_loss)]
        let min_len = left_tokens.len().min(right_tokens.len()) as f64;

        #[expect(clippy::cast_precision_loss)]
        let max_len = left_tokens.len().max(right_tokens.len()) as f64;

        return f64::mul_add(0.1, min_len / max_len, 0.9);
    }

    let shared_tokens = left_tokens
        .iter()
        .filter(|w| right_tokens.contains(w))
        .count();

    let partial_boost = if shared_tokens > 0 { 0.2 } else { 0.0 };

    f64::min(1.0, base_score + partial_boost)
}

pub fn contains(normalized_source: &str, normalized_pattern: &str) -> bool {
    if normalized_pattern.is_empty() {
        return true;
    }

    normalized_source.contains(normalized_pattern)
}
