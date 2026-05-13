pub fn embed_with_hash_fallback(text: &str, dimensions: usize) -> Vec<f32> {
    let mut vector = vec![0.0_f32; dimensions.max(1)];
    for token in text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        let mut hash = 2166136261_u32;
        for byte in token.bytes() {
            hash ^= u32::from(byte.to_ascii_lowercase());
            hash = hash.wrapping_mul(16777619);
        }
        vector[(hash as usize) % dimensions.max(1)] += 1.0;
    }
    vector
}
