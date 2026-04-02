use ndarray::Array3;

fn main() {
    let seq_len = 4;
    let num_heads = 2;
    let head_size = 8;

    // Create test data: flat array of (seq * num_heads * head_size) = 64 elements
    let query: Vec<f32> = (0..seq_len * num_heads * head_size)
        .map(|i| i as f32)
        .collect();

    println!("Query length: {}", query.len());

    // Method 1: Direct 3D reshape (seq, num_heads, head_size)
    let query_3d = Array3::from_shape_vec((seq_len, num_heads, head_size), query.clone()).unwrap();
    println!("Direct reshape shape: {:?}", query_3d.shape());

    // Check value at [i, h, k]
    for i in 0..seq_len {
        for h in 0..num_heads {
            let expected_idx = (i * num_heads + h) * head_size;
            let actual_val = query_3d[[i, h, 0]]; // First head_size position
            let expected_val = query[expected_idx] as f32;
            println!(
                "[{}, {}, 0]: actual={}, expected={}",
                i, h, actual_val, expected_val
            );
        }
    }

    // Method 2: Check if permuted version matches reference indexing
    // Reference: query[(i * head_size) + (h * head_size) + k]
    // This is: query[(i + h) * head_size + k]
    // Which is different from (seq * num_heads + head) * head_size + k

    println!("\nComparing indexing:");
    for i in 0..seq_len {
        for h in 0..num_heads {
            // Reference indexing
            let ref_idx = (i * head_size) + (h * head_size);
            // Our reshape indexing
            let our_idx = (i * num_heads + h) * head_size;
            println!("i={}, h={}: ref_idx={}, our_idx={}", i, h, ref_idx, our_idx);
        }
    }
}
