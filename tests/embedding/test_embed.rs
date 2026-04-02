use std::collections::HashMap;

fn main() {
    let text = "test note content";

    // Simple FNV-1a hash
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET;
    for byte in text.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    println!("Hash: {}", hash);
    println!("Hash as f64: {}", hash as f64);

    // Test the embedding generation
    let dim = 384;
    let mut embedding = Vec::with_capacity(dim);

    let seed = hash as f64;
    for i in 0..dim {
        let x = ((seed * (i as f64 + 1.0)).sin() + 1.0) / 2.0;
        embedding.push((x * 2.0 - 1.0) as f32);
    }

    println!("First 10 values: {:?}", &embedding[0..10]);
    println!("Last 10 values: {:?}", &embedding[embedding.len() - 10..]);

    // Calculate norm
    let norm: f32 = embedding.iter().map(|x| x * x).sum();
    println!("Norm: {}", norm.sqrt());
}
