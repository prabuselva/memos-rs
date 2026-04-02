use ndarray::{Array2, Array3};

// BERT Configuration for all-MiniLM-L6-v2
const HIDDEN_SIZE: usize = 384;
const NUM_HIDDEN_LAYERS: usize = 6;
const NUM_ATTENTION_HEADS: usize = 12;
const HEAD_SIZE: usize = HIDDEN_SIZE / NUM_ATTENTION_HEADS; // 32

// Small test configuration for debugging
const TEST_SEQ_LEN: usize = 4;
const TEST_NUM_HEADS: usize = 2;
const TEST_HEAD_SIZE: usize = 8;
const TEST_HIDDEN_SIZE: usize = TEST_NUM_HEADS * TEST_HEAD_SIZE;

// ============== REFERENCE IMPLEMENTATION (Nested Loops) ==============
// Fixed version with correct indexing: i * hidden_size + h * head_size

pub fn reference_compute_attention_scores(
    query: &[f32],
    key: &[f32],
    sequence_length: usize,
    head_size: usize,
    num_heads: usize,
) -> Vec<f32> {
    let mut scores = vec![0.0; sequence_length * sequence_length * num_heads];
    let hidden_size = num_heads * head_size;

    for h in 0..num_heads {
        let head_offset = h * head_size;
        for i in 0..sequence_length {
            for j in 0..sequence_length {
                let q_offset = (i * hidden_size) + head_offset;
                let k_offset = (j * hidden_size) + head_offset;

                let mut score = 0.0;
                for k in 0..head_size {
                    score += query[q_offset + k] * key[k_offset + k];
                }

                let score_idx = (i * sequence_length + j) * num_heads + h;
                scores[score_idx] = score / (head_size as f32).sqrt();
            }
        }
    }

    scores
}

pub fn reference_softmax(scores: &[f32], sequence_length: usize, _head_size: usize) -> Vec<f32> {
    let num_heads = scores.len() / (sequence_length * sequence_length);
    let mut probs = scores.to_vec();

    for h in 0..num_heads {
        for i in 0..sequence_length {
            let start = (i * sequence_length * num_heads) + h;

            let mut max_score = f32::NEG_INFINITY;
            for j in (0..sequence_length).map(|j| start + j * num_heads) {
                if probs[j] > max_score {
                    max_score = probs[j];
                }
            }

            let mut sum_exp = 0.0;
            for j in (0..sequence_length).map(|j| start + j * num_heads) {
                probs[j] = (probs[j] - max_score).exp();
                sum_exp += probs[j];
            }

            for j in (0..sequence_length).map(|j| start + j * num_heads) {
                probs[j] /= sum_exp;
            }
        }
    }

    probs
}

pub fn reference_apply_attention(
    attention_probs: &[f32],
    value: &[f32],
    sequence_length: usize,
    head_size: usize,
    num_heads: usize,
) -> Vec<f32> {
    let hidden_size = num_heads * head_size;
    let mut context = vec![0.0; sequence_length * hidden_size];

    for h in 0..num_heads {
        let head_offset = h * head_size;
        for i in 0..sequence_length {
            for j in 0..sequence_length {
                let prob_idx = (i * sequence_length + j) * num_heads + h;
                let val_offset = (j * hidden_size) + head_offset;
                let ctx_offset = (i * hidden_size) + head_offset;

                let prob = attention_probs[prob_idx];
                for k in 0..head_size {
                    context[ctx_offset + k] += prob * value[val_offset + k];
                }
            }
        }
    }

    context
}

// ============== NDARRAY IMPLEMENTATION ==============

pub fn ndarray_compute_attention_scores(
    query: &[f32],
    key: &[f32],
    sequence_length: usize,
    head_size: usize,
    num_heads: usize,
) -> Vec<f32> {
    eprintln!("\n=== ndarray_compute_attention_scores ===");
    eprintln!(
        "[INPUT] query.len()={}, key.len()={}",
        query.len(),
        key.len()
    );
    eprintln!(
        "[PARAMS] seq_len={}, head_size={}, num_heads={}",
        sequence_length, head_size, num_heads
    );

    // Reshape to 3D: (seq_len, num_heads, head_size)
    let query_3d = Array3::from_shape_vec((sequence_length, num_heads, head_size), query.to_vec())
        .expect("Failed to reshape query to 3D");
    let key_3d = Array3::from_shape_vec((sequence_length, num_heads, head_size), key.to_vec())
        .expect("Failed to reshape key to 3D");

    eprintln!(
        "[RESHAPE] query_3d.shape={:?}, key_3d.shape={:?}",
        query_3d.shape(),
        key_3d.shape()
    );

    // Permute to (num_heads, seq_len, head_size) for batched operations
    let query_permuted = query_3d.permuted_axes([1, 0, 2]).to_owned();
    let key_permuted = key_3d.permuted_axes([1, 0, 2]).to_owned();

    eprintln!(
        "[PERMUTE] query_permuted.shape={:?}, key_permuted.shape={:?}",
        query_permuted.shape(),
        key_permuted.shape()
    );

    // Transpose key to (num_heads, head_size, seq_len)
    let key_transposed = key_permuted.permuted_axes([0, 2, 1]).to_owned();

    eprintln!(
        "[TRANSPOSE] key_transposed.shape={:?}",
        key_transposed.shape()
    );

    // Batch matrix multiplication: (num_heads, seq_len, head_size) × (num_heads, head_size, seq_len)
    let mut scores = Array3::zeros((num_heads, sequence_length, sequence_length));

    for h in 0..num_heads {
        let q_h = query_permuted.index_axis(ndarray::Axis(0), h);
        let k_h = key_transposed.index_axis(ndarray::Axis(0), h);

        eprintln!(
            "[HEAD {}] q_h.shape={:?}, k_h.shape={:?}",
            h,
            q_h.shape(),
            k_h.shape()
        );

        let scores_h = q_h.dot(&k_h);
        eprintln!("[HEAD {}] scores_h.shape={:?}", h, scores_h.shape());

        for i in 0..sequence_length {
            for j in 0..sequence_length {
                scores[[h, i, j]] = scores_h[[i, j]];
            }
        }
    }

    eprintln!("[BEFORE SCALE] scores.shape={:?}", scores.shape());

    // Scale by sqrt(head_size)
    let scale = (head_size as f32).sqrt();
    scores /= scale;

    eprintln!(
        "[AFTER SCALE] scores.shape={:?}, first_value={:.6}",
        scores.shape(),
        scores[[0, 0, 0]]
    );

    // Convert to flat vector: (num_heads, seq_len, seq_len) -> (seq_len, seq_len, num_heads)
    let mut flat_scores = vec![0.0; sequence_length * sequence_length * num_heads];

    for i in 0..sequence_length {
        for j in 0..sequence_length {
            for h in 0..num_heads {
                let flat_idx = (i * sequence_length + j) * num_heads + h;
                flat_scores[flat_idx] = scores[[h, i, j]];
            }
        }
    }

    eprintln!(
        "[OUTPUT] flat_scores.len()={}, first_5=[{:.6}, {:.6}, {:.6}, {:.6}, {:.6}]",
        flat_scores.len(),
        flat_scores[0],
        flat_scores[1],
        flat_scores[2],
        flat_scores[3],
        flat_scores[4]
    );

    flat_scores
}

pub fn ndarray_softmax(scores: &[f32], sequence_length: usize, _head_size: usize) -> Vec<f32> {
    let num_heads = scores.len() / (sequence_length * sequence_length);
    let mut probs = scores.to_vec();

    eprintln!("\n=== ndarray_softmax ===");
    eprintln!(
        "[INPUT] scores.len()={}, num_heads={}, seq_len={}",
        scores.len(),
        num_heads,
        sequence_length
    );

    for i in 0..sequence_length {
        for h in 0..num_heads {
            let start = i * sequence_length * num_heads + h;

            let mut max_score = f32::NEG_INFINITY;
            for j in 0..sequence_length {
                let idx = start + j * num_heads;
                if probs[idx] > max_score {
                    max_score = probs[idx];
                }
            }

            eprintln!("[POS {}, HEAD {}] max_score={:.6}", i, h, max_score);

            let mut sum_exp = 0.0;
            for j in 0..sequence_length {
                let idx = start + j * num_heads;
                probs[idx] = (probs[idx] - max_score).exp();
                sum_exp += probs[idx];
            }

            for j in 0..sequence_length {
                let idx = start + j * num_heads;
                probs[idx] /= sum_exp;
            }

            eprintln!(
                "[POS {}, HEAD {}] sum_exp={:.6}, probs[start]={:.6}",
                i, h, sum_exp, probs[start]
            );
        }
    }

    eprintln!(
        "[OUTPUT] probs.len()={}, first_5=[{:.6}, {:.6}, {:.6}, {:.6}, {:.6}]",
        probs.len(),
        probs[0],
        probs[1],
        probs[2],
        probs[3],
        probs[4]
    );

    probs
}

pub fn ndarray_apply_attention(
    attention_probs: &[f32],
    value: &[f32],
    sequence_length: usize,
    head_size: usize,
    num_heads: usize,
) -> Vec<f32> {
    let hidden_size = num_heads * head_size;

    eprintln!("\n=== ndarray_apply_attention ===");
    eprintln!(
        "[INPUT] probs.len()={}, value.len()={}",
        attention_probs.len(),
        value.len()
    );
    eprintln!(
        "[PARAMS] seq_len={}, head_size={}, num_heads={}, hidden_size={}",
        sequence_length, head_size, num_heads, hidden_size
    );

    // Reshape value to 3D: (seq_len, num_heads, head_size)
    let value_3d = Array3::from_shape_vec((sequence_length, num_heads, head_size), value.to_vec())
        .expect("Failed to reshape value to 3D");

    eprintln!("[RESHAPE] value_3d.shape={:?}", value_3d.shape());

    // Permute to (num_heads, seq_len, head_size)
    let value_permuted = value_3d.permuted_axes([1, 0, 2]).to_owned();

    eprintln!(
        "[PERMUTE] value_permuted.shape={:?}",
        value_permuted.shape()
    );

    // Reshape attention probs to 3D: (num_heads, seq_len, seq_len)
    let mut probs_3d = Array3::zeros((num_heads, sequence_length, sequence_length));

    for h in 0..num_heads {
        for i in 0..sequence_length {
            for j in 0..sequence_length {
                let flat_idx = (i * sequence_length + j) * num_heads + h;
                probs_3d[[h, i, j]] = attention_probs[flat_idx];
            }
        }
    }

    eprintln!("[RESHAPE] probs_3d.shape={:?}", probs_3d.shape());

    // Batch matrix multiplication: (num_heads, seq_len, seq_len) × (num_heads, seq_len, head_size)
    let mut context = Array3::zeros((num_heads, sequence_length, head_size));

    for h in 0..num_heads {
        let probs_h = probs_3d.index_axis(ndarray::Axis(0), h);
        let value_h = value_permuted.index_axis(ndarray::Axis(0), h);

        eprintln!(
            "[HEAD {}] probs_h.shape={:?}, value_h.shape={:?}",
            h,
            probs_h.shape(),
            value_h.shape()
        );

        let context_h = probs_h.dot(&value_h);
        eprintln!("[HEAD {}] context_h.shape={:?}", h, context_h.shape());

        for i in 0..sequence_length {
            for k in 0..head_size {
                context[[h, i, k]] = context_h[[i, k]];
            }
        }
    }

    eprintln!("[BEFORE PERMUTE] context.shape={:?}", context.shape());

    // Permute back to (seq_len, num_heads, head_size) and make contiguous
    let context_permuted = context.permuted_axes([1, 0, 2]).to_owned();

    eprintln!(
        "[PERMUTE] context_permuted.shape={:?}",
        context_permuted.shape()
    );

    // Manually flatten to 2D to ensure correct memory layout
    let mut context_flat_data = vec![0.0; sequence_length * hidden_size];
    for i in 0..sequence_length {
        for h in 0..num_heads {
            for k in 0..head_size {
                let flat_idx = (i * hidden_size) + (h * head_size) + k;
                context_flat_data[flat_idx] = context_permuted[[i, h, k]];
            }
        }
    }

    let context_flat = Array2::from_shape_vec((sequence_length, hidden_size), context_flat_data)
        .expect("Failed to create 2D array");

    eprintln!("[FLATTEN] context_flat.shape={:?}", context_flat.shape());
    eprintln!(
        "[OUTPUT] context_flat.len()={}, first_5=[{:.6}, {:.6}, {:.6}, {:.6}, {:.6}]",
        context_flat.len(),
        context_flat[[0, 0]],
        context_flat[[0, 1]],
        context_flat[[0, 2]],
        context_flat[[0, 3]],
        context_flat[[0, 4]]
    );

    context_flat.into_raw_vec()
}

pub fn ndarray_linear_transform(
    hidden_states: &[f32],
    weights: &[f32],
    input_size: usize,
    output_size: usize,
) -> Vec<f32> {
    let sequence_length = hidden_states.len() / input_size;

    eprintln!("\n=== ndarray_linear_transform ===");
    eprintln!(
        "[INPUT] hidden_states.len()={}, weights.len()={}",
        hidden_states.len(),
        weights.len()
    );
    eprintln!(
        "[PARAMS] input_size={}, output_size={}, seq_len={}",
        input_size, output_size, sequence_length
    );

    let input_arr = Array2::from_shape_vec((sequence_length, input_size), hidden_states.to_vec())
        .expect("Failed to reshape input to 2D");

    let weight_arr = Array2::from_shape_vec((output_size, input_size), weights.to_vec())
        .expect("Failed to reshape weights to 2D");

    eprintln!(
        "[RESHAPE] input_arr.shape={:?}, weight_arr.shape={:?}",
        input_arr.shape(),
        weight_arr.shape()
    );

    let result = input_arr.dot(&weight_arr.t());

    eprintln!("[RESULT] result.shape={:?}", result.shape());
    eprintln!(
        "[OUTPUT] result.len()={}, first_5=[{:.6}, {:.6}, {:.6}, {:.6}, {:.6}]",
        result.len(),
        result[[0, 0]],
        result[[0, 1]],
        result[[0, 2]],
        result[[0, 3]],
        result[[0, 4]]
    );

    result.into_raw_vec()
}

// ============== UNIT TESTS ==============

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data(
        seq_len: usize,
        hidden_size: usize,
        num_heads: usize,
    ) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        let query: Vec<f32> = (0..seq_len * hidden_size)
            .map(|i| (i as f32) * 0.01)
            .collect();
        let key: Vec<f32> = (0..seq_len * hidden_size)
            .map(|i| (i as f32) * 0.02)
            .collect();
        let value: Vec<f32> = (0..seq_len * hidden_size)
            .map(|i| (i as f32) * 0.03)
            .collect();
        let weights: Vec<f32> = (0..hidden_size * hidden_size)
            .map(|i| (i as f32) * 0.001)
            .collect();

        (query, key, value, weights)
    }

    #[test]
    fn test_linear_transform_shapes() {
        let seq_len = 4;
        let input_size = 16;
        let output_size = 16;

        let hidden_states: Vec<f32> = (0..seq_len * input_size)
            .map(|i| (i as f32) * 0.01)
            .collect();
        let weights: Vec<f32> = (0..input_size * output_size)
            .map(|i| (i as f32) * 0.001)
            .collect();

        let result = ndarray_linear_transform(&hidden_states, &weights, input_size, output_size);

        assert_eq!(result.len(), seq_len * output_size);
    }

    #[test]
    fn test_attention_scores_shapes() {
        let seq_len = 4;
        let hidden_size = 16;
        let num_heads = 2;
        let head_size = hidden_size / num_heads;

        let (query, key, _value, _weights) = create_test_data(seq_len, hidden_size, num_heads);

        let scores = ndarray_compute_attention_scores(&query, &key, seq_len, head_size, num_heads);

        assert_eq!(scores.len(), seq_len * seq_len * num_heads);
    }

    #[test]
    fn test_softmax_shapes() {
        let seq_len = 4;
        let hidden_size = 16;
        let num_heads = 2;
        let head_size = hidden_size / num_heads;

        let (query, key, _value, _weights) = create_test_data(seq_len, hidden_size, num_heads);
        let scores = ndarray_compute_attention_scores(&query, &key, seq_len, head_size, num_heads);
        let probs = ndarray_softmax(&scores, seq_len, head_size);

        assert_eq!(probs.len(), seq_len * seq_len * num_heads);

        for i in 0..seq_len {
            for h in 0..num_heads {
                let start = i * seq_len * num_heads + h;
                let sum: f32 = (0..seq_len).map(|j| probs[start + j * num_heads]).sum();
                assert!(
                    (sum - 1.0).abs() < 1e-5,
                    "Softmax sum should be 1.0 for i={}, h={}, got {}",
                    i,
                    h,
                    sum
                );
            }
        }
    }

    #[test]
    fn test_softmax_correctness() {
        let seq_len = 4;
        let hidden_size = 16;
        let num_heads = 2;
        let head_size = hidden_size / num_heads;

        let (query, key, _value, _weights) = create_test_data(seq_len, hidden_size, num_heads);
        let scores = ndarray_compute_attention_scores(&query, &key, seq_len, head_size, num_heads);
        let probs = ndarray_softmax(&scores, seq_len, head_size);

        for i in 0..seq_len {
            for h in 0..num_heads {
                let start = i * seq_len * num_heads + h;
                let sum: f32 = (0..seq_len).map(|j| probs[start + j * num_heads]).sum();

                assert!(
                    (sum - 1.0).abs() < 1e-5,
                    "Softmax sum should be 1.0 for i={}, h={}, got {}",
                    i,
                    h,
                    sum
                );

                for j in 0..seq_len {
                    let idx = start + j * num_heads;
                    assert!(
                        probs[idx] >= 0.0 && probs[idx] <= 1.0,
                        "Softmax probability should be in [0, 1], got {} at i={}, j={}, h={}",
                        probs[idx],
                        i,
                        j,
                        h
                    );
                }
            }
        }
    }

    #[test]
    fn test_apply_attention_shapes() {
        let seq_len = 4;
        let hidden_size = 16;
        let num_heads = 2;
        let head_size = hidden_size / num_heads;

        let (query, key, value, _weights) = create_test_data(seq_len, hidden_size, num_heads);
        let scores = ndarray_compute_attention_scores(&query, &key, seq_len, head_size, num_heads);
        let probs = ndarray_softmax(&scores, seq_len, head_size);
        let context = ndarray_apply_attention(&probs, &value, seq_len, head_size, num_heads);

        assert_eq!(context.len(), seq_len * hidden_size);
    }

    #[test]
    fn test_reference_vs_ndarray_scores() {
        let seq_len = 4;
        let hidden_size = 16;
        let num_heads = 2;
        let head_size = hidden_size / num_heads;

        let (query, key, _value, _weights) = create_test_data(seq_len, hidden_size, num_heads);

        let scores_ref =
            reference_compute_attention_scores(&query, &key, seq_len, head_size, num_heads);
        let scores_nd =
            ndarray_compute_attention_scores(&query, &key, seq_len, head_size, num_heads);

        for i in 0..scores_ref.len() {
            let diff = (scores_ref[i] - scores_nd[i]).abs();
            assert!(
                diff < 1e-4,
                "Score mismatch at index {}: ref={}, nd={}, diff={}",
                i,
                scores_ref[i],
                scores_nd[i],
                diff
            );
        }
    }

    #[test]
    fn test_reference_vs_ndarray_probs() {
        let seq_len = 4;
        let hidden_size = 16;
        let num_heads = 2;
        let head_size = hidden_size / num_heads;

        let (query, key, _value, _weights) = create_test_data(seq_len, hidden_size, num_heads);

        let scores =
            reference_compute_attention_scores(&query, &key, seq_len, head_size, num_heads);
        let probs_ref = reference_softmax(&scores, seq_len, head_size);
        let probs_nd = ndarray_softmax(&scores, seq_len, head_size);

        for i in 0..probs_ref.len() {
            let diff = (probs_ref[i] - probs_nd[i]).abs();
            assert!(
                diff < 1e-5,
                "Prob mismatch at index {}: ref={}, nd={}, diff={}",
                i,
                probs_ref[i],
                probs_nd[i],
                diff
            );
        }
    }

    #[test]
    fn test_numerical_stability() {
        let seq_len = 4;
        let hidden_size = 16;
        let num_heads = 2;
        let head_size = hidden_size / num_heads;

        let (query, key, _value, _weights) = create_test_data(seq_len, hidden_size, num_heads);

        let scores = ndarray_compute_attention_scores(&query, &key, seq_len, head_size, num_heads);
        let probs = ndarray_softmax(&scores, seq_len, head_size);

        assert!(
            !probs.iter().any(|&x| x.is_nan()),
            "Probs should not contain NaN"
        );
        assert!(
            !probs.iter().any(|&x| x.is_infinite()),
            "Probs should not contain inf"
        );

        for &p in &probs {
            assert!(p >= 0.0 && p <= 1.0, "Prob should be in [0, 1], got {}", p);
        }
    }

    #[test]
    fn test_full_attention_pipeline() {
        let seq_len = 4;
        let hidden_size = 16;
        let num_heads = 2;
        let head_size = hidden_size / num_heads;

        let (query, key, value, _weights) = create_test_data(seq_len, hidden_size, num_heads);

        let scores = ndarray_compute_attention_scores(&query, &key, seq_len, head_size, num_heads);
        let probs = ndarray_softmax(&scores, seq_len, head_size);
        let context = ndarray_apply_attention(&probs, &value, seq_len, head_size, num_heads);

        assert_eq!(context.len(), seq_len * hidden_size);
        assert!(!context.iter().any(|&x| x.is_nan()));
    }
}
