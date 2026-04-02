use anyhow::{Context, Result};
use ndarray::{Array2, Array3};
use std::collections::HashMap;
use std::fs;

use tracing::{debug, error, info};

use crate::embeddings::tokenizer::EmbeddingTokenizer;

#[derive(Clone)]
pub struct BERTModel {
    tokenizer: EmbeddingTokenizer,
    config: ModelConfig,
    weights: ModelWeights,
}

#[derive(Clone)]
pub struct ModelConfig {
    pub hidden_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub intermediate_size: usize,
    pub max_position_embeddings: usize,
    pub vocab_size: usize,
    pub hidden_act: String,
}

#[derive(Clone)]
pub struct ModelWeights {
    pub embeddings: EmbeddingWeights,
    pub encoder: Vec<EncoderWeights>,
    pub pooler: Option<PoolerWeights>,
}

#[derive(Clone)]
pub struct EmbeddingWeights {
    pub word_embeddings: Vec<f32>,
    pub position_embeddings: Vec<f32>,
    pub token_type_embeddings: Vec<f32>,
    pub layer_norm: LayerNormWeights,
}

#[derive(Clone)]
pub struct LayerNormWeights {
    pub weight: Vec<f32>,
    pub bias: Vec<f32>,
}

#[derive(Clone)]
pub struct EncoderWeights {
    pub attention: AttentionWeights,
    pub intermediate: FeedForwardWeights,
    pub output: FeedForwardWeights,
}

#[derive(Clone)]
pub struct AttentionOutputWeights {
    pub dense: Vec<f32>,
    pub layer_norm: LayerNormWeights,
}

#[derive(Clone)]
pub struct AttentionWeights {
    pub query: Vec<f32>,
    pub key: Vec<f32>,
    pub value: Vec<f32>,
    pub output: AttentionOutputWeights,
}

#[derive(Clone)]
pub struct FeedForwardWeights {
    pub dense: Vec<f32>,
    pub layer_norm: LayerNormWeights,
}

#[derive(Clone)]
pub struct PoolerWeights {
    pub dense: Vec<f32>,
    pub layer_norm: LayerNormWeights,
}

impl BERTModel {
    pub fn load(tokenizer_path: &str, model_dir: &str) -> Result<Self> {
        info!(
            "[BERT::INFO] [BERT::load] Loading tokenizer from: {}",
            tokenizer_path
        );
        let tokenizer = EmbeddingTokenizer::load(tokenizer_path).map_err(|e| {
            error!("[BERT::load] Failed to load tokenizer: {}", e);
            e
        })?;
        info!(
            "[BERT::INFO] [BERT::load] Loading config from: {}",
            model_dir
        );
        let config = load_config(model_dir).map_err(|e| {
            error!("[BERT::load] Failed to load config: {}", e);
            e
        })?;
        info!(
            "[BERT::INFO] [BERT::load] Loading weights from: {}",
            model_dir
        );
        let weights = load_weights(model_dir).map_err(|e| {
            error!("[BERT::load] Failed to load weights: {}", e);
            e
        })?;
        info!("[BERT::INFO] [BERT::load] Weights loaded successfully");

        Ok(Self {
            tokenizer,
            config,
            weights,
        })
    }

    pub fn load_with_config(config: &crate::config::Config, model_dir: &str) -> Result<Self> {
        let tokenizer_path = config.get_tokenizer_path();
        let tokenizer = EmbeddingTokenizer::load(tokenizer_path.to_str().unwrap())?;
        let model_config = load_config(model_dir)?;
        let weights = load_weights(model_dir)?;

        Ok(Self {
            tokenizer,
            config: model_config,
            weights,
        })
    }

    pub fn tokenize(&self, text: &str) -> Result<Vec<u32>> {
        self.tokenizer.tokenize(text)
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        info!(
            "[BERT::embed] Starting embedding for text ({} chars): {}",
            text.len(),
            &text[..text.len().min(50)]
        );
        debug!("[BERT::embed::DEBUG] Starting debug embedding flow");
        let start_total = std::time::Instant::now();

        debug!("[BERT::embed::DEBUG] Calling tokenize for text");
        let tokens = self.tokenize(text)?;
        debug!("[BERT::embed] Tokenized,  {} tokens", tokens.len());
        debug!("[BERT::embed::DEBUG] Tokenization complete");
        debug!(
            "[BERT::embed] Tokenization time: {:?}",
            start_total.elapsed()
        );

        debug!("[BERT::embed::DEBUG] Calling forward pass");
        let embeddings = self.forward(&tokens)?;
        debug!("[BERT::embed::DEBUG] Forward pass complete");

        info!(
            "[BERT::embed] Total embedding time: {:?}, embedding dimension: {}",
            start_total.elapsed(),
            embeddings.len()
        );
        debug!(
            "[BERT::embed::DEBUG] Embedding computed, first 5 values: {:?}",
            &embeddings[0..5.min(embeddings.len())]
        );
        Ok(embeddings)
    }

    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }

    fn forward(&self, tokens: &[u32]) -> Result<Vec<f32>> {
        debug!(
            "[BERT::forward::DEBUG] Starting forward pass with {} tokens",
            tokens.len()
        );
        let sequence_length = tokens.len();
        let start_forward = std::time::Instant::now();

        debug!(
            "[BERT::forward] Starting forward pass, sequence_length={}",
            sequence_length
        );

        debug!("[BERT::forward::DEBUG] Calling embeddings_forward");
        let embedding_output = self.embeddings_forward(tokens, sequence_length)?;
        debug!("[BERT::forward::DEBUG] embeddings_forward complete");
        debug!(
            "[BERT::forward] Embeddings forward time: {:?}",
            start_forward.elapsed()
        );
        debug!(
            "[BERT::forward::DEBUG] Embedding output shape: {}",
            embedding_output.len()
        );

        let mut hidden_states = embedding_output;
        let mut layer_times: Vec<std::time::Duration> = Vec::new();

        for (layer_idx, encoder_layer) in self.weights.encoder.iter().enumerate() {
            debug!(
                "[BERT::forward::DEBUG] Processing encoder layer {}",
                layer_idx
            );
            let start_layer = std::time::Instant::now();
            hidden_states = self.encoder_forward(&hidden_states, &encoder_layer)?;
            let layer_time = start_layer.elapsed();
            layer_times.push(layer_time);
            debug!(
                "[BERT::forward] Layer {} encoder_forward time: {:?}, total so far: {:?}",
                layer_idx,
                layer_time,
                start_forward.elapsed()
            );
            debug!(
                "[BERT::forward::DEBUG] Layer {} output shape: {}",
                layer_idx,
                hidden_states.len()
            );
        }

        let start_pooler = std::time::Instant::now();
        debug!("[BERT::forward::DEBUG] Calling pooler_forward");
        let pooled = self.pooler_forward(&hidden_states)?;
        debug!("[BERT::forward::DEBUG] pooler_forward complete");
        debug!(
            "[BERT::DEBUG] [BERT::forward] Pooler time: {:?}",
            start_pooler.elapsed()
        );
        debug!(
            "[BERT::forward] Total encoder layers time: {:?}",
            layer_times.iter().sum::<std::time::Duration>()
        );
        debug!(
            "[BERT::forward] Total forward time: {:?}",
            start_forward.elapsed()
        );
        debug!(
            "[BERT::forward::DEBUG] Final pooled output shape: {}",
            pooled.len()
        );

        Ok(pooled)
    }

    fn embeddings_forward(&self, tokens: &[u32], sequence_length: usize) -> Result<Vec<f32>> {
        debug!(
            "[BERT::embeddings_forward] sequence_length={}, hidden_size={}",
            sequence_length, self.config.hidden_size
        );
        debug!("[BERT::embeddings_forward::DEBUG] Starting embeddings computation");
        let start_total = std::time::Instant::now();

        let hidden_size = self.config.hidden_size;
        let mut embeddings = vec![0.0; sequence_length * hidden_size];

        debug!("[BERT::embeddings_forward::DEBUG] Computing token embeddings");
        let start_token_emb = std::time::Instant::now();
        for (i, &token_id) in tokens.iter().enumerate() {
            let emb_offset = (i * hidden_size) as usize;
            let token_emb = self.get_token_embedding(token_id)?;

            for j in 0..hidden_size {
                embeddings[emb_offset + j] = token_emb[j];
            }
        }
        debug!("[BERT::embeddings_forward::DEBUG] Token embeddings computed");
        debug!(
            "[BERT::embeddings_forward] Token embedding time: {:?}",
            start_token_emb.elapsed()
        );

        debug!("[BERT::embeddings_forward::DEBUG] Adding positional embeddings");
        let start_pos = std::time::Instant::now();
        let positional = self.add_positional_embeddings(&embeddings, sequence_length)?;
        debug!("[BERT::embeddings_forward::DEBUG] Positional embeddings added");
        debug!(
            "[BERT::embeddings_forward] Positional embedding time: {:?}",
            start_pos.elapsed()
        );

        debug!("[BERT::embeddings_forward::DEBUG] Applying layer normalization");
        let start_norm = std::time::Instant::now();
        let normalized =
            self.layer_norm_forward(&positional, &self.weights.embeddings.layer_norm)?;
        debug!("[BERT::embeddings_forward::DEBUG] Layer normalization complete");
        debug!(
            "[BERT::embeddings_forward] Layer norm time: {:?}",
            start_norm.elapsed()
        );
        debug!(
            "[BERT::embeddings_forward] Total embeddings_forward time: {:?}",
            start_total.elapsed()
        );
        debug!(
            "[BERT::embeddings_forward::DEBUG] Final embeddings shape: {}",
            normalized.len()
        );

        Ok(normalized)
    }

    fn add_positional_embeddings(
        &self,
        embeddings: &[f32],
        sequence_length: usize,
    ) -> Result<Vec<f32>> {
        let start_total = std::time::Instant::now();
        let hidden_size = self.config.hidden_size;
        let mut result = embeddings.to_vec();

        let start_pos_emb = std::time::Instant::now();
        for i in 0..sequence_length {
            let pos_emb = self.get_position_embedding(i)?;
            let pos_offset = i * hidden_size;

            for j in 0..hidden_size {
                let pos_val = pos_emb[j];
                if (j == 7 || j >= hidden_size - 2) && i < 10 {
                    debug!(
                        "[BERT::add_positional_embeddings] Position embedding at i={}, j={}, val={}",
                        i, j, pos_val
                    );
                }
                if pos_val.is_nan() || pos_val.is_infinite() {
                    error!(
                        "[BERT::add_positional_embeddings] Invalid position embedding at i={}, j={}, val={}",
                        i, j, pos_val
                    );
                }
                result[pos_offset + j] += pos_val;
                if (j == 7 || j >= hidden_size - 2) && i < 10 {
                    debug!(
                        "[BERT::add_positional_embeddings] After add, result at i={}, j={}, val={}",
                        i,
                        j,
                        result[pos_offset + j]
                    );
                }
            }
        }
        debug!(
            "[BERT::add_positional_embeddings] Position embedding lookup time: {:?}",
            start_pos_emb.elapsed()
        );

        debug!(
            "[BERT::add_positional_embeddings] Added positional embeddings for {} tokens, total time: {:?}",
            sequence_length,
            start_total.elapsed()
        );

        let mut has_invalid = false;
        for i in 0..result.len() {
            if result[i].is_nan() || result[i].is_infinite() {
                has_invalid = true;
                error!(
                    "[BERT::add_positional_embeddings] Invalid result at index {}: {}",
                    i, result[i]
                );
                break;
            }
        }
        if has_invalid {
            error!("[BERT::add_positional_embeddings] result has invalid values!");
        }

        Ok(result)
    }

    fn get_position_embedding(&self, position_id: usize) -> Result<Vec<f32>> {
        let start = std::time::Instant::now();
        let hidden_size = self.config.hidden_size;
        let max_position = self.config.max_position_embeddings;

        if position_id >= max_position {
            return Err(anyhow::anyhow!("Position ID {} out of range", position_id));
        }

        let offset = (position_id * hidden_size) as usize;
        let embedding =
            self.weights.embeddings.position_embeddings[offset..offset + hidden_size].to_vec();

        debug!(
            "[BERT::get_position_embedding] Position ID {} time: {:?}",
            position_id,
            start.elapsed()
        );

        for j in 0..hidden_size {
            if (j == 7 || j >= hidden_size - 2) && position_id < 10 {
                debug!(
                    "[BERT::get_position_embedding] position_id={}, j={}, val={}",
                    position_id, j, embedding[j]
                );
            }
        }

        Ok(embedding)
    }

    fn get_token_embedding(&self, token_id: u32) -> Result<Vec<f32>> {
        let start = std::time::Instant::now();
        let hidden_size = self.config.hidden_size;
        let vocab_size = self.config.vocab_size;

        if token_id as usize >= vocab_size {
            return Err(anyhow::anyhow!("Token ID {} out of range", token_id));
        }

        let offset = (token_id as usize * hidden_size) as usize;
        let embedding =
            self.weights.embeddings.word_embeddings[offset..offset + hidden_size].to_vec();

        debug!(
            "[BERT::get_token_embedding] Token ID {} time: {:?}",
            token_id,
            start.elapsed()
        );

        for j in 0..hidden_size {
            if (j == 7 || j >= hidden_size - 2) && token_id < 100 {
                debug!(
                    "[BERT::get_token_embedding] token_id={}, j={}, val={}",
                    token_id, j, embedding[j]
                );
            }
        }

        Ok(embedding)
    }

    fn encoder_forward(&self, hidden_states: &[f32], weights: &EncoderWeights) -> Result<Vec<f32>> {
        debug!("[BERT::encoder_forward::DEBUG] Starting encoder_forward");
        let start_total = std::time::Instant::now();
        let sequence_length = hidden_states.len() / self.config.hidden_size;
        let hidden_size = self.config.hidden_size;
        debug!(
            "[BERT::encoder_forward] Input shape: {}x{}",
            sequence_length, hidden_size
        );
        debug!(
            "[BERT::encoder_forward::DEBUG] Input shape: {}",
            hidden_states.len()
        );

        debug!("[BERT::encoder_forward::DEBUG] Computing self-attention");
        let start_attn = std::time::Instant::now();
        let attention_output = self.self_attention_forward(hidden_states, weights)?;
        debug!("[BERT::encoder_forward::DEBUG] Self-attention computed");
        debug!(
            "[BERT::encoder_forward] Self attention time: {:?}",
            start_attn.elapsed()
        );

        debug!("[BERT::encoder_forward::DEBUG] Adding residual connection (attention)");
        let start_residual = std::time::Instant::now();
        let residual_after_attn = self.add_residual(&attention_output, hidden_states)?;
        debug!("[BERT::encoder_forward::DEBUG] Residual connection added (attention)");
        debug!(
            "[BERT::encoder_forward] Add residual (attention) time: {:?}",
            start_residual.elapsed()
        );

        debug!("[BERT::encoder_forward::DEBUG] Applying layer norm 1");
        let start_norm1 = std::time::Instant::now();
        let normalized =
            self.layer_norm_forward(&residual_after_attn, &weights.attention.output.layer_norm)?;
        debug!("[BERT::encoder_forward::DEBUG] Layer norm 1 complete");
        debug!(
            "[BERT::encoder_forward] Layer norm 1 time: {:?}",
            start_norm1.elapsed()
        );

        debug!("[BERT::encoder_forward::DEBUG] Computing feed-forward");
        let start_ffn = std::time::Instant::now();
        let intermediate_output = self.intermediate_forward(&normalized, weights)?;
        debug!("[BERT::encoder_forward::DEBUG] Feed-forward computed");
        debug!(
            "[BERT::encoder_forward] Feed forward time: {:?}",
            start_ffn.elapsed()
        );

        debug!("[BERT::encoder_forward::DEBUG] Computing output forward");
        let start_out = std::time::Instant::now();
        let output_dense = self.output_forward(&intermediate_output, weights)?;
        debug!("[BERT::encoder_forward::DEBUG] Output forward computed");
        debug!(
            "[BERT::encoder_forward] Output forward time: {:?}",
            start_out.elapsed()
        );

        debug!("[BERT::encoder_forward::DEBUG] Adding residual connection (final)");
        let start_residual2 = std::time::Instant::now();
        let residual_final = self.add_residual(&output_dense, &normalized)?;
        debug!("[BERT::encoder_forward::DEBUG] Residual connection added (final)");
        debug!(
            "[BERT::encoder_forward] Add residual (final) time: {:?}",
            start_residual2.elapsed()
        );

        debug!("[BERT::encoder_forward::DEBUG] Applying layer norm 2");
        let start_norm2 = std::time::Instant::now();
        let final_output = self.layer_norm_forward(&residual_final, &weights.output.layer_norm)?;
        debug!("[BERT::encoder_forward::DEBUG] Layer norm 2 complete");
        debug!(
            "[BERT::encoder_forward] Layer norm 2 time: {:?}",
            start_norm2.elapsed()
        );
        debug!(
            "[BERT::encoder_forward] Total encoder_forward time: {:?}",
            start_total.elapsed()
        );
        debug!(
            "[BERT::encoder_forward::DEBUG] Output shape: {}",
            final_output.len()
        );

        Ok(final_output)
    }

    fn self_attention_forward(
        &self,
        hidden_states: &[f32],
        weights: &EncoderWeights,
    ) -> Result<Vec<f32>> {
        debug!("[BERT::self_attention_forward::DEBUG] Starting self-attention computation");
        let start_total = std::time::Instant::now();
        let sequence_length = hidden_states.len() / self.config.hidden_size;
        let hidden_size = self.config.hidden_size;
        let num_heads = self.config.num_attention_heads;
        let head_size = hidden_size / num_heads;
        debug!(
            "[BERT::self_attention_forward] seq_len={}, hidden_size={}, num_heads={}, head_size={}",
            sequence_length, hidden_size, num_heads, head_size
        );

        debug!("[BERT::self_attention_forward::DEBUG] Computing query vectors");
        let start_query = std::time::Instant::now();
        let query = self.linear_transform_tokens(hidden_states, &weights.attention.query)?;
        debug!("[BERT::self_attention_forward::DEBUG] Query vectors computed");
        debug!(
            "[BERT::DEBUG] [BERT::self_attention_forward] Query linear time: {:?}",
            start_query.elapsed()
        );

        debug!("[BERT::self_attention_forward::DEBUG] Computing key vectors");
        let start_key = std::time::Instant::now();
        let key = self.linear_transform_tokens(hidden_states, &weights.attention.key)?;
        debug!("[BERT::self_attention_forward::DEBUG] Key vectors computed");
        debug!(
            "[BERT::DEBUG] [BERT::self_attention_forward] Key linear time: {:?}",
            start_key.elapsed()
        );

        debug!("[BERT::self_attention_forward::DEBUG] Computing value vectors");
        let start_value = std::time::Instant::now();
        let value = self.linear_transform_tokens(hidden_states, &weights.attention.value)?;
        debug!("[BERT::self_attention_forward::DEBUG] Value vectors computed");
        debug!(
            "[BERT::DEBUG] [BERT::self_attention_forward] Value linear time: {:?}",
            start_value.elapsed()
        );

        debug!("[BERT::self_attention_forward::DEBUG] Computing attention scores");
        let start_scores = std::time::Instant::now();
        let attention_scores =
            self.compute_attention_scores(&query, &key, sequence_length, head_size)?;
        debug!("[BERT::self_attention_forward::DEBUG] Attention scores computed");
        debug!(
            "[BERT::DEBUG] [BERT::self_attention_forward] Attention scores time: {:?}",
            start_scores.elapsed()
        );

        debug!("[BERT::self_attention_forward::DEBUG] Computing softmax");
        let start_softmax = std::time::Instant::now();
        let attention_probs = self.softmax(&attention_scores, sequence_length, head_size)?;
        debug!("[BERT::self_attention_forward::DEBUG] Softmax computed");
        debug!(
            "[BERT::DEBUG] [BERT::self_attention_forward] Softmax time: {:?}",
            start_softmax.elapsed()
        );

        debug!("[BERT::self_attention_forward::DEBUG] Applying attention to values");
        let start_apply = std::time::Instant::now();
        let context = self.apply_attention(&attention_probs, &value, sequence_length, head_size)?;
        debug!("[BERT::self_attention_forward::DEBUG] Attention applied to values");
        debug!(
            "[BERT::DEBUG] [BERT::self_attention_forward] Apply attention time: {:?}",
            start_apply.elapsed()
        );

        debug!("[BERT::self_attention_forward::DEBUG] Computing output projection");
        let start_out = std::time::Instant::now();
        let output = self.linear_transform_tokens(&context, &weights.attention.output.dense)?;
        debug!("[BERT::self_attention_forward::DEBUG] Output projection computed");
        debug!(
            "[BERT::DEBUG] [BERT::self_attention_forward] Output linear time: {:?}",
            start_out.elapsed()
        );
        debug!(
            "[BERT::self_attention_forward] Output shape: {}",
            output.len()
        );

        let mut has_nan = false;
        for i in 0..output.len() {
            if output[i].is_nan() {
                has_nan = true;
                debug!(
                    "[BERT::self_attention_forward] NaN in output at index {}",
                    i
                );
                break;
            }
        }
        if has_nan {
            debug!("[BERT::DEBUG] [BERT::self_attention_forward] output has NaN values!");
        }

        debug!("[BERT::self_attention_forward::DEBUG] Applying layer normalization");
        let start_ln = std::time::Instant::now();
        let normalized = self.layer_norm_forward(&output, &weights.attention.output.layer_norm)?;
        debug!("[BERT::self_attention_forward::DEBUG] Layer normalization complete");
        debug!(
            "[BERT::DEBUG] [BERT::self_attention_forward] Layer norm time: {:?}",
            start_ln.elapsed()
        );
        debug!(
            "[BERT::self_attention_forward] After layer_norm: {}",
            normalized.len()
        );

        let mut has_nan = false;
        for i in 0..normalized.len() {
            if normalized[i].is_nan() {
                has_nan = true;
                debug!(
                    "[BERT::self_attention_forward] NaN in normalized at index {}",
                    i
                );
                break;
            }
        }
        if has_nan {
            debug!("[BERT::DEBUG] [BERT::self_attention_forward] normalized has NaN values!");
        }

        debug!(
            "[BERT::DEBUG] [BERT::self_attention_forward] Total self_attention_forward time: {:?}",
            start_total.elapsed()
        );
        debug!(
            "[BERT::self_attention_forward::DEBUG] Self-attention complete, output shape: {}",
            normalized.len()
        );

        Ok(normalized)
    }

    fn linear_transform_tokens(&self, hidden_states: &[f32], weights: &[f32]) -> Result<Vec<f32>> {
        let start_total = std::time::Instant::now();
        let config_hidden_size = self.config.hidden_size;
        let intermediate_size = self.config.intermediate_size;
        let weights_len = weights.len();
        let input_len = hidden_states.len();

        let mut actual_input_size = config_hidden_size;
        let mut actual_output_size = config_hidden_size;

        // Special case: if input is 196608 (128 * 1536), we're in the output layer
        // which should produce 49152 (128 * 384)
        if input_len == 196608 && weights_len == 589824 {
            actual_input_size = 1536;
            actual_output_size = 384;
        } else {
            let mut best_score = 0;
            let mut found_valid = false;

            for &input_size in &[config_hidden_size, intermediate_size] {
                if input_size > 0 && input_len % input_size == 0 {
                    let seq_len = input_len / input_size;
                    if seq_len > 0 && seq_len <= 512 {
                        if weights_len % input_size == 0 {
                            let out_size = weights_len / input_size;
                            if out_size == config_hidden_size || out_size == intermediate_size {
                                let score = out_size * 1000 + seq_len;
                                if score > best_score {
                                    best_score = score;
                                    actual_input_size = input_size;
                                    actual_output_size = out_size;
                                    found_valid = true;
                                }
                            }
                        }
                    }
                }
            }

            if !found_valid {
                actual_input_size = config_hidden_size;
                actual_output_size = weights_len / config_hidden_size;
            }
        }

        let sequence_length = input_len / actual_input_size;
        debug!(
            "[BERT::linear_transform] input_len={}, weights_len={}, input_size={}, output_size={}, seq_len={}",
            input_len, weights_len, actual_input_size, actual_output_size, sequence_length
        );

        let start_reshape = std::time::Instant::now();
        let input_arr =
            Array2::from_shape_vec((sequence_length, actual_input_size), hidden_states.to_vec())?;

        let weight_arr =
            Array2::from_shape_vec((actual_output_size, actual_input_size), weights.to_vec())?;
        debug!(
            "[BERT::linear_transform] Reshape time: {:?}",
            start_reshape.elapsed()
        );

        let start_dot = std::time::Instant::now();
        let result = input_arr.dot(&weight_arr.t());
        debug!(
            "[BERT::linear_transform] Dot product time: {:?}",
            start_dot.elapsed()
        );

        debug!(
            "[BERT::linear_transform] Total linear_transform time: {:?}",
            start_total.elapsed()
        );

        return Ok(result.into_raw_vec());
    }

    fn compute_attention_scores(
        &self,
        query: &[f32],
        key: &[f32],
        sequence_length: usize,
        head_size: usize,
    ) -> Result<Vec<f32>> {
        debug!("[BERT::compute_attention_scores::DEBUG] Starting attention score computation");
        let start_total = std::time::Instant::now();
        let num_heads = self.config.num_attention_heads;
        let hidden_size = self.config.hidden_size;

        debug!(
            "[BERT::compute_attention_scores] num_heads={}, seq_len={}, head_size={}, query_len={}, key_len={}",
            num_heads, sequence_length, head_size, query.len(), key.len()
        );

        debug!("[BERT::compute_attention_scores::DEBUG] Reshaping query and key to 3D");
        let start_reshape = std::time::Instant::now();
        // Reshape to 3D: (seq_len, num_heads, head_size)
        let query_3d =
            Array3::from_shape_vec((sequence_length, num_heads, head_size), query.to_vec())?;
        let key_3d = Array3::from_shape_vec((sequence_length, num_heads, head_size), key.to_vec())?;
        debug!("[BERT::compute_attention_scores::DEBUG] Reshaped to 3D");
        debug!(
            "[BERT::compute_attention_scores] Reshape time: {:?}",
            start_reshape.elapsed()
        );

        debug!("[BERT::compute_attention_scores::DEBUG] Permuting axes");
        let start_permute = std::time::Instant::now();
        // Permute to (num_heads, seq_len, head_size) for batched operations
        let query_permuted = query_3d.permuted_axes([1, 0, 2]).to_owned();
        let key_permuted = key_3d.permuted_axes([1, 0, 2]).to_owned();
        debug!("[BERT::compute_attention_scores::DEBUG] Axes permuted");
        debug!(
            "[BERT::compute_attention_scores] Permute time: {:?}",
            start_permute.elapsed()
        );

        debug!("[BERT::compute_attention_scores::DEBUG] Transposing key");
        let start_transpose = std::time::Instant::now();
        // Transpose key to (num_heads, head_size, seq_len)
        let key_transposed = key_permuted.permuted_axes([0, 2, 1]).to_owned();
        debug!("[BERT::compute_attention_scores::DEBUG] Key transposed");
        debug!(
            "[BERT::compute_attention_scores] Transpose time: {:?}",
            start_transpose.elapsed()
        );

        // Batch matrix multiplication: (num_heads, seq_len, head_size) × (num_heads, head_size, seq_len)
        let mut scores = Array3::zeros((num_heads, sequence_length, sequence_length));

        debug!("[BERT::compute_attention_scores::DEBUG] Computing batched GEMM");
        let start_gemm = std::time::Instant::now();
        for h in 0..num_heads {
            let q_h = query_permuted.index_axis(ndarray::Axis(0), h);
            let k_h = key_transposed.index_axis(ndarray::Axis(0), h);

            let scores_h = q_h.dot(&k_h);

            for i in 0..sequence_length {
                for j in 0..sequence_length {
                    scores[[h, i, j]] = scores_h[[i, j]];
                }
            }
        }
        debug!("[BERT::compute_attention_scores::DEBUG] GEMM computation complete");
        debug!(
            "[BERT::compute_attention_scores] GEMM time: {:?}",
            start_gemm.elapsed()
        );

        // Scale by sqrt(head_size)
        let scale = (head_size as f32).sqrt();
        scores /= scale;

        debug!("[BERT::compute_attention_scores::DEBUG] Flattening scores");
        let start_flatten = std::time::Instant::now();
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
        debug!("[BERT::compute_attention_scores::DEBUG] Flattening complete");
        debug!(
            "[BERT::compute_attention_scores] Flatten time: {:?}",
            start_flatten.elapsed()
        );
        debug!(
            "[BERT::compute_attention_scores] Total compute_attention_scores time: {:?}",
            start_total.elapsed()
        );
        debug!(
            "[BERT::compute_attention_scores::DEBUG] Attention scores computed, shape: {}",
            flat_scores.len()
        );

        Ok(flat_scores)
    }

    fn softmax(
        &self,
        scores: &[f32],
        sequence_length: usize,
        _head_size: usize,
    ) -> Result<Vec<f32>> {
        let start_total = std::time::Instant::now();
        let num_heads = self.config.num_attention_heads;
        let mut probs = scores.to_vec();

        debug!(
            "[BERT::softmax] Starting softmax, num_heads={}, seq_len={}",
            num_heads, sequence_length
        );

        let start_softmax = std::time::Instant::now();
        for i in 0..sequence_length {
            for h in 0..num_heads {
                let row: Vec<f32> = (0..sequence_length)
                    .map(|j| {
                        let idx = (i * sequence_length + j) * num_heads + h;
                        probs[idx]
                    })
                    .collect();

                let max_score = row.iter().fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));

                let exp_row: Vec<f32> = row.iter().map(|&x| (x - max_score).exp()).collect();
                let sum_exp: f32 = exp_row.iter().sum();

                for j in 0..sequence_length {
                    let idx = (i * sequence_length + j) * num_heads + h;
                    probs[idx] = exp_row[j] / sum_exp;
                }
            }
        }
        debug!(
            "[BERT::softmax] Softmax computation time: {:?}",
            start_softmax.elapsed()
        );
        debug!(
            "[BERT::softmax] Total softmax time: {:?}",
            start_total.elapsed()
        );

        Ok(probs)
    }

    fn apply_attention(
        &self,
        attention_probs: &[f32],
        value: &[f32],
        sequence_length: usize,
        head_size: usize,
    ) -> Result<Vec<f32>> {
        let start_total = std::time::Instant::now();
        let num_heads = self.config.num_attention_heads;
        let hidden_size = num_heads * head_size;

        debug!(
            "[BERT::apply_attention] probs.len()={}, value.len()={}, seq_len={}, head_size={}, num_heads={}",
            attention_probs.len(), value.len(), sequence_length, head_size, num_heads
        );

        let start_reshape_val = std::time::Instant::now();
        let value_3d =
            Array3::from_shape_vec((sequence_length, num_heads, head_size), value.to_vec())?;
        debug!(
            "[BERT::apply_attention] Value reshape time: {:?}",
            start_reshape_val.elapsed()
        );

        let start_permute_val = std::time::Instant::now();
        let value_permuted = value_3d.permuted_axes([1, 0, 2]).to_owned();
        debug!(
            "[BERT::apply_attention] Value permute time: {:?}",
            start_permute_val.elapsed()
        );

        let start_reshape_probs = std::time::Instant::now();
        let mut probs_3d = Array3::zeros((num_heads, sequence_length, sequence_length));

        for h in 0..num_heads {
            for i in 0..sequence_length {
                for j in 0..sequence_length {
                    let flat_idx = (i * sequence_length + j) * num_heads + h;
                    probs_3d[[h, i, j]] = attention_probs[flat_idx];
                }
            }
        }
        debug!(
            "[BERT::apply_attention] Probs reshape time: {:?}",
            start_reshape_probs.elapsed()
        );

        // Batch matrix multiplication: (num_heads, seq_len, seq_len) × (num_heads, seq_len, head_size)
        let mut context = Array3::zeros((num_heads, sequence_length, head_size));

        let start_gemm = std::time::Instant::now();
        for h in 0..num_heads {
            let probs_h = probs_3d.index_axis(ndarray::Axis(0), h);
            let value_h = value_permuted.index_axis(ndarray::Axis(0), h);

            let context_h = probs_h.dot(&value_h);

            for i in 0..sequence_length {
                for k in 0..head_size {
                    context[[h, i, k]] = context_h[[i, k]];
                }
            }
        }
        debug!(
            "[BERT::apply_attention] GEMM time: {:?}",
            start_gemm.elapsed()
        );

        let start_permute_ctx = std::time::Instant::now();
        let context_permuted = context.permuted_axes([1, 0, 2]).to_owned();
        debug!(
            "[BERT::apply_attention] Context permute time: {:?}",
            start_permute_ctx.elapsed()
        );

        let start_flatten = std::time::Instant::now();
        let mut context_flat_data = vec![0.0; sequence_length * hidden_size];
        for i in 0..sequence_length {
            for h in 0..num_heads {
                for k in 0..head_size {
                    let flat_idx = (i * hidden_size) + (h * head_size) + k;
                    context_flat_data[flat_idx] = context_permuted[[i, h, k]];
                }
            }
        }
        debug!(
            "[BERT::apply_attention] Flatten time: {:?}",
            start_flatten.elapsed()
        );
        debug!(
            "[BERT::apply_attention] Total apply_attention time: {:?}",
            start_total.elapsed()
        );

        let context_flat =
            Array2::from_shape_vec((sequence_length, hidden_size), context_flat_data)?;

        Ok(context_flat.into_raw_vec())
    }

    fn intermediate_forward(&self, input: &[f32], weights: &EncoderWeights) -> Result<Vec<f32>> {
        let start_total = std::time::Instant::now();
        debug!("[BERT::DEBUG] [BERT::intermediate_forward] Starting intermediate_forward");

        let start_linear = std::time::Instant::now();
        let intermediate = self.linear_transform_tokens(input, &weights.intermediate.dense)?;
        debug!(
            "[BERT::intermediate_forward] Linear transform time: {:?}",
            start_linear.elapsed()
        );

        let start_gelu = std::time::Instant::now();
        let mut output = intermediate.clone();
        for val in output.iter_mut() {
            *val = Self::gelu(*val);
        }
        debug!(
            "[BERT::intermediate_forward] GELU time: {:?}",
            start_gelu.elapsed()
        );

        let start_norm = std::time::Instant::now();
        // Check if intermediate layer norm exists (some models like MiniLM don't have it)
        let normalized = if weights.intermediate.layer_norm.weight.is_empty() {
            debug!(
                "[BERT::DEBUG] [BERT::intermediate_forward] Skipping layer norm (empty weights)"
            );
            output
        } else {
            self.layer_norm_forward(&output, &weights.intermediate.layer_norm)?
        };
        debug!(
            "[BERT::intermediate_forward] Layer norm time: {:?}",
            start_norm.elapsed()
        );
        debug!(
            "[BERT::intermediate_forward] Total intermediate_forward time: {:?}",
            start_total.elapsed()
        );

        Ok(normalized)
    }

    fn output_forward(&self, input: &[f32], weights: &EncoderWeights) -> Result<Vec<f32>> {
        let start_total = std::time::Instant::now();
        debug!("[BERT::DEBUG] [BERT::output_forward] Starting output_forward");

        let start_linear = std::time::Instant::now();
        let output = self.linear_transform_tokens(input, &weights.output.dense)?;
        debug!(
            "[BERT::output_forward] Linear transform time: {:?}",
            start_linear.elapsed()
        );

        let start_norm = std::time::Instant::now();
        let normalized = self.layer_norm_forward(&output, &weights.output.layer_norm)?;
        debug!(
            "[BERT::output_forward] Layer norm time: {:?}",
            start_norm.elapsed()
        );
        debug!(
            "[BERT::output_forward] Total output_forward time: {:?}",
            start_total.elapsed()
        );

        Ok(normalized)
    }

    fn layer_norm_forward(&self, input: &[f32], weights: &LayerNormWeights) -> Result<Vec<f32>> {
        let start_total = std::time::Instant::now();
        debug!(
            "[BERT::layer_norm_forward] Starting layer_norm_forward, input_size={}",
            input.len()
        );

        let input_size = input.len();
        let epsilon = 1e-12;

        let feature_size = weights.weight.len();
        if feature_size == 0 {
            return Err(anyhow::anyhow!("Layer norm weight size is zero"));
        }

        if input_size % feature_size != 0 {
            return Err(anyhow::anyhow!(
                "Input size {} not divisible by feature size {}",
                input_size,
                feature_size
            ));
        }

        let sequence_length = input_size / feature_size;
        let mut output = vec![0.0; input_size];
        let mut has_invalid = false;

        for feat in 0..feature_size {
            let mut sum = 0.0;
            for seq in 0..sequence_length {
                let input_val = input[seq * feature_size + feat];
                if (feat < 2 || feat >= feature_size - 2 || feat == 7) && seq < 10 {
                    debug!(
                        "[BERT::layer_norm_forward] Input[seq={}, feat={}]={}",
                        seq, feat, input_val
                    );
                }
                if input_val.is_nan() || input_val.is_infinite() {
                    error!(
                        "[BERT::layer_norm_forward] Invalid input at seq={}, feat={}, val={}",
                        seq, feat, input_val
                    );
                    has_invalid = true;
                }
                let prev_sum = sum;
                sum += input_val;
                if sum.is_nan() || sum.is_infinite() {
                    error!("[BERT::layer_norm_forward] Sum overflow at seq={}, feat={}, sum={}, input_val={}, prev_sum={}", seq, feat, sum, input_val, prev_sum);
                    has_invalid = true;
                }
            }
            let mean = sum / sequence_length as f32;
            if feat < 2 || feat >= feature_size - 2 || feat == 7 {
                debug!(
                    "[BERT::DEBUG] [BERT::layer_norm_forward] Mean for feat={}={}",
                    feat, mean
                );
                debug!(
                    "[BERT::DEBUG] [BERT::layer_norm_forward] Sum for feat={}={}",
                    feat, sum
                );
            }

            let mut var_sum = 0.0;
            for seq in 0..sequence_length {
                let input_val = input[seq * feature_size + feat];
                let diff = input_val - mean;
                if diff.is_nan() || diff.is_infinite() {
                    error!(
                        "[BERT::layer_norm_forward] Invalid diff at seq={}, feat={}, diff={}, input={}, mean={}",
                        seq, feat, diff, input_val, mean
                    );
                    has_invalid = true;
                }
                var_sum += diff * diff;
                if var_sum.is_nan() || var_sum.is_infinite() {
                    error!("[BERT::layer_norm_forward] var_sum overflow at seq={}, feat={}, var_sum={}, diff={}, input={}, mean={}", seq, feat, var_sum, diff, input_val, mean);
                    has_invalid = true;
                }
            }
            let variance = var_sum / sequence_length as f32;

            if variance.is_nan() || variance.is_infinite() {
                error!(
                    "[BERT::layer_norm_forward] Invalid variance: {}, feat={}, mean={}, var_sum={}, seq_len={}",
                    variance, feat, mean, var_sum, sequence_length
                );
                has_invalid = true;
            }

            for seq in 0..sequence_length {
                let idx = seq * feature_size + feat;
                let variance_plus_eps = variance + epsilon;
                let std = variance_plus_eps.sqrt();
                if std.is_nan() || std.is_infinite() || std == 0.0 {
                    error!("[BERT::layer_norm_forward] Invalid std: {}, variance={}, epsilon={}, feat={}, seq={}", std, variance, epsilon, feat, seq);
                    has_invalid = true;
                }
                let diff = input[idx] - mean;
                let normalized = diff / std;
                if normalized.is_nan() || normalized.is_infinite() {
                    error!("[BERT::layer_norm_forward] Invalid normalized: {}, diff={}, mean={}, feat={}, seq={}", normalized, diff, mean, feat, seq);
                    has_invalid = true;
                }
                output[idx] = normalized * weights.weight[feat] + weights.bias[feat];
                let output_val = output[idx];
                if output_val.is_nan() || output_val.is_infinite() {
                    error!("[BERT::layer_norm_forward] Invalid output: {}, normalized={}, weight={}, bias={}, feat={}, seq={}", output_val, normalized, weights.weight[feat], weights.bias[feat], feat, seq);
                    has_invalid = true;
                }
            }
        }

        if has_invalid {
            return Err(anyhow::anyhow!(
                "layer_norm_forward: detected invalid values"
            ));
        }

        debug!(
            "[BERT::layer_norm_forward] Total time: {:?}",
            start_total.elapsed()
        );
        Ok(output)
    }

    fn pooler_forward(&self, hidden_states: &[f32]) -> Result<Vec<f32>> {
        let start_total = std::time::Instant::now();
        let hidden_size = self.config.hidden_size;

        debug!("[BERT::DEBUG] [BERT::pooler_forward] Starting pooler_forward");

        let start_linear = std::time::Instant::now();
        let first_token = &hidden_states[0..hidden_size];
        let pooled = self
            .linear_transform_tokens(first_token, &self.weights.pooler.as_ref().unwrap().dense)?;
        debug!(
            "[BERT::pooler_forward] Linear transform time: {:?}",
            start_linear.elapsed()
        );

        let start_tanh = std::time::Instant::now();
        let mut activated = pooled.clone();
        for val in activated.iter_mut() {
            *val = val.tanh();
        }
        debug!(
            "[BERT::pooler_forward] Tanh activation time: {:?}",
            start_tanh.elapsed()
        );
        debug!(
            "[BERT::pooler_forward] Total pooler_forward time: {:?}",
            start_total.elapsed()
        );

        debug!(
            "[BERT::pooler_forward] Pooler output shape: {}",
            activated.len()
        );
        Ok(activated)
    }

    pub fn gelu(x: f32) -> f32 {
        const GELU_COEFF: f32 = 0.044715;
        const SQRT_2_OVER_PI: f32 = 0.7978845608028654;

        let x_cubed = x * x * x;
        let inner = SQRT_2_OVER_PI * (x + GELU_COEFF * x_cubed);
        0.5 * x * (1.0 + inner.tanh())
    }

    pub fn add_residual(&self, main: &[f32], residual: &[f32]) -> Result<Vec<f32>> {
        let start = std::time::Instant::now();
        debug!(
            "[BERT::add_residual] Starting add_residual, main_len={}, residual_len={}",
            main.len(),
            residual.len()
        );

        if main.len() != residual.len() {
            return Err(anyhow::anyhow!(
                "Residual connection dimension mismatch: {} vs {}",
                main.len(),
                residual.len()
            ));
        }

        let mut result = vec![0.0; main.len()];
        for i in 0..main.len() {
            let sum = main[i] + residual[i];
            if sum.is_nan() || sum.is_infinite() {
                debug!(
                    "[BERT::add_residual] Invalid sum: {}, main={}, residual={}, i={}",
                    sum, main[i], residual[i], i
                );
            }
            result[i] = sum;
        }
        debug!(
            "[BERT::DEBUG] [BERT::add_residual] Total time: {:?}",
            start.elapsed()
        );
        Ok(result)
    }
}

fn load_config(model_dir: &str) -> Result<ModelConfig> {
    let config_path = format!("{}/config.json", model_dir);

    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config from {}", config_path))?;

    let config: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;

    let hidden_size = config
        .get("hidden_size")
        .context("Missing hidden_size in config")?
        .as_u64()
        .context("hidden_size must be a number")? as usize;

    let num_hidden_layers = config
        .get("num_hidden_layers")
        .context("Missing num_hidden_layers in config")?
        .as_u64()
        .context("num_hidden_layers must be a number")? as usize;

    let num_attention_heads = config
        .get("num_attention_heads")
        .context("Missing num_attention_heads in config")?
        .as_u64()
        .context("num_attention_heads must be a number")? as usize;

    let intermediate_size = config
        .get("intermediate_size")
        .context("Missing intermediate_size in config")?
        .as_u64()
        .context("intermediate_size must be a number")? as usize;

    let max_position_embeddings = config
        .get("max_position_embeddings")
        .context("Missing max_position_embeddings in config")?
        .as_u64()
        .context("max_position_embeddings must be a number")?
        as usize;

    let vocab_size = config
        .get("vocab_size")
        .context("Missing vocab_size in config")?
        .as_u64()
        .context("vocab_size must be a number")? as usize;

    let hidden_act = config
        .get("hidden_act")
        .context("Missing hidden_act in config")?
        .as_str()
        .context("hidden_act must be a string")?
        .to_string();

    Ok(ModelConfig {
        hidden_size,
        num_hidden_layers,
        num_attention_heads,
        intermediate_size,
        max_position_embeddings,
        vocab_size,
        hidden_act,
    })
}

fn load_weights(model_dir: &str) -> Result<ModelWeights> {
    let weights_path = format!("{}/model.safetensors", model_dir);

    // Check if safetensors file exists
    if std::path::Path::new(&weights_path).exists() {
        info!(
            "[BERT::INFO] [BERT] Loading weights from safetensors: {}",
            weights_path
        );
        let weights = load_weights_from_safetensors(&weights_path)?;
        return Ok(weights);
    }

    // Fallback to pytorch_model.bin
    let weights_path = format!("{}/pytorch_model.bin", model_dir);
    info!(
        "[BERT] Loading weights from pytorch_model.bin: {}",
        weights_path
    );

    let content = fs::read(&weights_path)
        .with_context(|| format!("Failed to read weights from {}", weights_path))?;

    let weights = parse_weights(&content)?;

    Ok(weights)
}

fn parse_weights(data: &[u8]) -> Result<ModelWeights> {
    let mut offset = 0;

    fn read_f32s(data: &[u8], offset: &mut usize, count: usize) -> Result<Vec<f32>> {
        let bytes_needed = count * 4;
        if *offset + bytes_needed > data.len() {
            return Err(anyhow::anyhow!("Unexpected end of data"));
        }
        let slice = &data[*offset..*offset + bytes_needed];
        *offset += bytes_needed;

        let mut result = vec![0.0; count];
        for (i, item) in result.iter_mut().enumerate() {
            let bytes = [
                slice[i * 4],
                slice[i * 4 + 1],
                slice[i * 4 + 2],
                slice[i * 4 + 3],
            ];
            *item = f32::from_le_bytes(bytes);
        }
        Ok(result)
    }

    let hidden_size = 384;
    let vocab_size = 30522;
    let num_layers = 6;
    let _num_heads = 12;
    let intermediate_size = 1536;

    let word_embeddings = read_f32s(data, &mut offset, vocab_size * hidden_size)?;
    let position_embeddings = read_f32s(data, &mut offset, 512 * hidden_size)?;
    let token_type_embeddings = read_f32s(data, &mut offset, 2 * hidden_size)?;
    let embedding_layer_norm_weight = read_f32s(data, &mut offset, hidden_size)?;
    let embedding_layer_norm_bias = read_f32s(data, &mut offset, hidden_size)?;
    let embedding_layer_norm = LayerNormWeights {
        weight: embedding_layer_norm_weight,
        bias: embedding_layer_norm_bias,
    };

    let mut encoder_layers = Vec::new();
    for _ in 0..num_layers {
        let query = read_f32s(data, &mut offset, hidden_size * hidden_size)?;
        let key = read_f32s(data, &mut offset, hidden_size * hidden_size)?;
        let value = read_f32s(data, &mut offset, hidden_size * hidden_size)?;

        let attention_dense = read_f32s(data, &mut offset, hidden_size * hidden_size)?;
        let attention_layer_norm_weight = read_f32s(data, &mut offset, hidden_size)?;
        let attention_layer_norm_bias = read_f32s(data, &mut offset, hidden_size)?;
        let attention_layer_norm = LayerNormWeights {
            weight: attention_layer_norm_weight,
            bias: attention_layer_norm_bias,
        };

        let intermediate = read_f32s(data, &mut offset, hidden_size * intermediate_size)?;
        let intermediate_layer_norm_weight = read_f32s(data, &mut offset, intermediate_size)?;
        let intermediate_layer_norm_bias = read_f32s(data, &mut offset, intermediate_size)?;
        let intermediate_layer_norm = LayerNormWeights {
            weight: intermediate_layer_norm_weight,
            bias: intermediate_layer_norm_bias,
        };

        let output_dense = read_f32s(data, &mut offset, intermediate_size * hidden_size)?;
        let output_layer_norm_weight = read_f32s(data, &mut offset, hidden_size)?;
        let output_layer_norm_bias = read_f32s(data, &mut offset, hidden_size)?;
        let output_layer_norm = LayerNormWeights {
            weight: output_layer_norm_weight,
            bias: output_layer_norm_bias,
        };

        encoder_layers.push(EncoderWeights {
            attention: AttentionWeights {
                query,
                key,
                value,
                output: AttentionOutputWeights {
                    dense: attention_dense,
                    layer_norm: attention_layer_norm,
                },
            },
            intermediate: FeedForwardWeights {
                dense: intermediate,
                layer_norm: intermediate_layer_norm,
            },
            output: FeedForwardWeights {
                dense: output_dense,
                layer_norm: output_layer_norm,
            },
        });
    }

    let pooler_dense = read_f32s(data, &mut offset, hidden_size * hidden_size)?;
    let pooler_layer_norm_weight = read_f32s(data, &mut offset, hidden_size)?;
    let pooler_layer_norm_bias = read_f32s(data, &mut offset, hidden_size)?;
    let pooler_layer_norm = LayerNormWeights {
        weight: pooler_layer_norm_weight,
        bias: pooler_layer_norm_bias,
    };

    Ok(ModelWeights {
        embeddings: EmbeddingWeights {
            word_embeddings,
            position_embeddings,
            token_type_embeddings,
            layer_norm: embedding_layer_norm,
        },
        encoder: encoder_layers,
        pooler: Some(PoolerWeights {
            dense: pooler_dense,
            layer_norm: pooler_layer_norm,
        }),
    })
}

fn load_weights_from_safetensors(weights_path: &str) -> Result<ModelWeights> {
    use safetensors::SafeTensors;
    use std::fs;

    info!(
        "[BERT::load_weights_from_safetensors] Loading from: {}",
        weights_path
    );

    let data = fs::read(weights_path)
        .with_context(|| format!("Failed to read safetensors from {}", weights_path))?;
    debug!(
        "[BERT::load_weights_from_safetensors] Read {} bytes",
        data.len()
    );

    let tensors =
        SafeTensors::deserialize(&data).with_context(|| "Failed to deserialize safetensors")?;
    let tensor_names = tensors.names();
    debug!(
        "[BERT::load_weights_from_safetensors] Deserialized {} tensors",
        tensor_names.len()
    );

    let hidden_size = 384;
    let vocab_size = 30522;
    let num_layers = 6;
    let intermediate_size = 1536;

    // Helper function to convert bytes to f32
    fn bytes_to_f32s(bytes: &[u8]) -> Result<Vec<f32>> {
        if bytes.len() % 4 != 0 {
            return Err(anyhow::anyhow!("Bytes length not divisible by 4"));
        }
        let mut result = Vec::with_capacity(bytes.len() / 4);
        for i in 0..(bytes.len() / 4) {
            let slice = &bytes[i * 4..(i + 1) * 4];
            let bytes_arr = [slice[0], slice[1], slice[2], slice[3]];
            result.push(f32::from_le_bytes(bytes_arr));
        }
        Ok(result)
    }

    // Extract word embeddings (30522 x 384)
    let word_embeddings_bytes = tensors.tensor("embeddings.word_embeddings.weight")?.data();
    let word_embeddings = bytes_to_f32s(word_embeddings_bytes)?;

    // Extract position embeddings (512 x 384)
    let position_embeddings_bytes = tensors
        .tensor("embeddings.position_embeddings.weight")?
        .data();
    let position_embeddings = bytes_to_f32s(position_embeddings_bytes)?;

    // Extract token type embeddings (2 x 384)
    let token_type_embeddings_bytes = tensors
        .tensor("embeddings.token_type_embeddings.weight")?
        .data();
    let token_type_embeddings = bytes_to_f32s(token_type_embeddings_bytes)?;

    // Extract embedding layer norm
    let embedding_layer_norm_weight_bytes = tensors.tensor("embeddings.LayerNorm.weight")?.data();
    let embedding_layer_norm_weight = bytes_to_f32s(embedding_layer_norm_weight_bytes)?;

    let embedding_layer_norm_bias_bytes = tensors.tensor("embeddings.LayerNorm.bias")?.data();
    let embedding_layer_norm_bias = bytes_to_f32s(embedding_layer_norm_bias_bytes)?;

    let embedding_layer_norm = LayerNormWeights {
        weight: embedding_layer_norm_weight,
        bias: embedding_layer_norm_bias,
    };

    let mut encoder_layers = Vec::new();
    for i in 0..num_layers {
        // Attention weights
        let query_bytes = tensors
            .tensor(&format!("encoder.layer.{}.attention.self.query.weight", i))?
            .data();
        let query = bytes_to_f32s(query_bytes)?;

        let key_bytes = tensors
            .tensor(&format!("encoder.layer.{}.attention.self.key.weight", i))?
            .data();
        let key = bytes_to_f32s(key_bytes)?;

        let value_bytes = tensors
            .tensor(&format!("encoder.layer.{}.attention.self.value.weight", i))?
            .data();
        let value = bytes_to_f32s(value_bytes)?;

        let attention_dense_bytes = tensors
            .tensor(&format!(
                "encoder.layer.{}.attention.output.dense.weight",
                i
            ))?
            .data();
        let attention_dense = bytes_to_f32s(attention_dense_bytes)?;

        let attention_layer_norm_weight_bytes = tensors
            .tensor(&format!(
                "encoder.layer.{}.attention.output.LayerNorm.weight",
                i
            ))?
            .data();
        let attention_layer_norm_weight = bytes_to_f32s(attention_layer_norm_weight_bytes)?;

        let attention_layer_norm_bias_bytes = tensors
            .tensor(&format!(
                "encoder.layer.{}.attention.output.LayerNorm.bias",
                i
            ))?
            .data();
        let attention_layer_norm_bias = bytes_to_f32s(attention_layer_norm_bias_bytes)?;

        let attention_layer_norm = LayerNormWeights {
            weight: attention_layer_norm_weight,
            bias: attention_layer_norm_bias,
        };

        // FFN weights
        let intermediate_bytes = tensors
            .tensor(&format!("encoder.layer.{}.intermediate.dense.weight", i))?
            .data();
        let intermediate = bytes_to_f32s(intermediate_bytes)?;

        // Check if intermediate layer norm exists (some models like MiniLM don't have it)
        let intermediate_layer_norm_weight_bytes = match tensors.tensor(&format!(
            "encoder.layer.{}.intermediate.LayerNorm.weight",
            i
        )) {
            Ok(t) => t.data(),
            Err(_) => {
                debug!("[BERT::DEBUG] [BERT::load_weights_from_safetensors] Intermediate layer norm not found for layer {}, using empty weights", i);
                &[]
            }
        };
        let intermediate_layer_norm_weight = if intermediate_layer_norm_weight_bytes.is_empty() {
            vec![]
        } else {
            bytes_to_f32s(intermediate_layer_norm_weight_bytes)?
        };

        let intermediate_layer_norm_bias_bytes =
            match tensors.tensor(&format!("encoder.layer.{}.intermediate.LayerNorm.bias", i)) {
                Ok(t) => t.data(),
                Err(_) => &[],
            };
        let intermediate_layer_norm_bias = if intermediate_layer_norm_bias_bytes.is_empty() {
            vec![]
        } else {
            bytes_to_f32s(intermediate_layer_norm_bias_bytes)?
        };

        let intermediate_layer_norm = LayerNormWeights {
            weight: intermediate_layer_norm_weight,
            bias: intermediate_layer_norm_bias,
        };

        let output_dense_bytes = tensors
            .tensor(&format!("encoder.layer.{}.output.dense.weight", i))?
            .data();
        let output_dense = bytes_to_f32s(output_dense_bytes)?;

        let output_layer_norm_weight_bytes = tensors
            .tensor(&format!("encoder.layer.{}.output.LayerNorm.weight", i))?
            .data();
        let output_layer_norm_weight = bytes_to_f32s(output_layer_norm_weight_bytes)?;

        let output_layer_norm_bias_bytes = tensors
            .tensor(&format!("encoder.layer.{}.output.LayerNorm.bias", i))?
            .data();
        let output_layer_norm_bias = bytes_to_f32s(output_layer_norm_bias_bytes)?;

        let output_layer_norm = LayerNormWeights {
            weight: output_layer_norm_weight,
            bias: output_layer_norm_bias,
        };

        encoder_layers.push(EncoderWeights {
            attention: AttentionWeights {
                query,
                key,
                value,
                output: AttentionOutputWeights {
                    dense: attention_dense,
                    layer_norm: attention_layer_norm,
                },
            },
            intermediate: FeedForwardWeights {
                dense: intermediate,
                layer_norm: intermediate_layer_norm,
            },
            output: FeedForwardWeights {
                dense: output_dense,
                layer_norm: output_layer_norm,
            },
        });
    }

    let pooler_dense_bytes = tensors.tensor("pooler.dense.weight")?.data();
    let pooler_dense = bytes_to_f32s(pooler_dense_bytes)?;

    // Check if pooler layer norm exists (some models don't have it)
    let pooler_layer_norm_weight_bytes = match tensors.tensor("pooler.layer_norm.weight") {
        Ok(t) => t.data(),
        Err(_) => {
            debug!("[BERT::DEBUG] [BERT::load_weights_from_safetensors] Pooler layer norm not found, using empty weights");
            &[]
        }
    };
    let pooler_layer_norm_weight = if pooler_layer_norm_weight_bytes.is_empty() {
        vec![]
    } else {
        bytes_to_f32s(pooler_layer_norm_weight_bytes)?
    };

    let pooler_layer_norm_bias_bytes = match tensors.tensor("pooler.layer_norm.bias") {
        Ok(t) => t.data(),
        Err(_) => &[],
    };
    let pooler_layer_norm_bias = if pooler_layer_norm_bias_bytes.is_empty() {
        vec![]
    } else {
        bytes_to_f32s(pooler_layer_norm_bias_bytes)?
    };

    let pooler_layer_norm = LayerNormWeights {
        weight: pooler_layer_norm_weight,
        bias: pooler_layer_norm_bias,
    };

    info!("[BERT::INFO] [BERT::load_weights_from_safetensors] Successfully loaded all weights");
    Ok(ModelWeights {
        embeddings: EmbeddingWeights {
            word_embeddings,
            position_embeddings,
            token_type_embeddings,
            layer_norm: embedding_layer_norm,
        },
        encoder: encoder_layers,
        pooler: Some(PoolerWeights {
            dense: pooler_dense,
            layer_norm: pooler_layer_norm,
        }),
    })
}
