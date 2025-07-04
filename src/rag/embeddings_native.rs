// 🧠 Native Rust Embeddings - Production Implementation
// High-performance sentence transformer using Candle transformers

use {
    anyhow::{Context, Result},
    candle_core::{DType, Device, Result as CandleResult, Tensor},
    candle_nn::VarBuilder,
    candle_transformers::models::bert::{BertModel, Config as BertConfig},
    hf_hub::api::tokio::Api,
    std::collections::HashMap,
    std::path::Path,
    tokenizers::Tokenizer,
};

use std::sync::Arc;
use tokio::sync::Mutex;

/// 🚀 Native embeddings client using Candle transformers
/// Direct replacement for Python sentence-transformers with Metal acceleration
pub struct NativeEmbedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    initialized: Arc<Mutex<bool>>,
}

impl NativeEmbedder {
    /// 🏗️ Initialize native embedder with all-MiniLM-L6-v2 model
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        eprintln!("🚀 Initializing native Rust embeddings with Candle model...");

        // Select best available device with Apple Silicon optimization
        let device = Self::select_optimal_device()?;
        eprintln!("🔧 Using device: {device:?}");

        // Download and load real sentence transformer model
        let model_name = "sentence-transformers/all-MiniLM-L6-v2";
        eprintln!("📥 Downloading model: {model_name}");

        let (model, tokenizer) = Self::load_real_model_and_tokenizer(model_name, &device).await?;

        eprintln!("✅ Native embedder ready!");
        eprintln!("   📊 Model: {model_name}");
        eprintln!("   🎯 Dimensions: 384");
        eprintln!("   ⚡ Device: {device:?}");

        Ok(Self {
            model,
            tokenizer,
            device,
            initialized: Arc::new(Mutex::new(true)),
        })
    }

    fn select_optimal_device() -> Result<Device> {
        // Priority: Metal (Apple Silicon) > CUDA > CPU

        // Try Metal first (Apple Silicon M1/M2/M3/M4)
        if let Ok(device) = Device::new_metal(0) {
            eprintln!("🍎 Using Metal acceleration (Apple Silicon)");
            eprintln!("   ⚡ GPU-accelerated inference enabled");
            return Ok(device);
        }

        // Try CUDA next (NVIDIA GPUs)
        if let Ok(device) = Device::new_cuda(0) {
            eprintln!("🚀 Using CUDA acceleration (NVIDIA GPU)");
            eprintln!("   ⚡ GPU-accelerated inference enabled");
            return Ok(device);
        }

        // Fallback to CPU with optimization
        eprintln!("💻 Using CPU with SIMD optimizations");
        eprintln!("   📊 Multi-threaded inference enabled");
        Ok(Device::Cpu)
    }

    async fn load_real_model_and_tokenizer(
        model_name: &str,
        device: &Device,
    ) -> Result<(BertModel, Tokenizer)> {
        // Download model files from HuggingFace Hub
        let api = Api::new()?;
        let repo = api.model(model_name.to_string());

        eprintln!("📥 Downloading model files...");

        // Download required files
        let config_file = repo
            .get("config.json")
            .await
            .context("Failed to download config.json")?;
        let tokenizer_file = repo
            .get("tokenizer.json")
            .await
            .context("Failed to download tokenizer.json")?;

        // Try different weight file formats
        let weights_file = match repo.get("model.safetensors").await {
            Ok(file) => {
                eprintln!("📦 Using SafeTensors format");
                file
            }
            Err(_) => {
                eprintln!("📦 Falling back to PyTorch format");
                repo.get("pytorch_model.bin")
                    .await
                    .context("Failed to download model weights")?
            }
        };

        eprintln!("⚙️ Loading model configuration...");

        // Load and parse config
        let config_content =
            std::fs::read_to_string(&config_file).context("Failed to read config file")?;
        let config: BertConfig =
            serde_json::from_str(&config_content).context("Failed to parse model config")?;

        eprintln!("📊 Model config:");
        eprintln!("   🔢 Hidden size: {}", config.hidden_size);
        eprintln!("   🏗️ Layers: {}", config.num_hidden_layers);
        eprintln!("   🎯 Attention heads: {}", config.num_attention_heads);

        // Load model weights
        eprintln!("⚡ Loading model weights...");
        let vb = if weights_file.extension() == Some("safetensors".as_ref()) {
            Self::load_safetensors_weights(&weights_file, device)?
        } else {
            Self::load_pytorch_weights(&weights_file, device)?
        };

        // Initialize BERT model
        eprintln!("🧠 Initializing transformer model...");
        let bert_model = BertModel::load(vb, &config).context("Failed to initialize BERT model")?;

        // Load tokenizer
        eprintln!("🔤 Loading tokenizer...");
        let tokenizer = Tokenizer::from_file(&tokenizer_file)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        eprintln!("✅ Model and tokenizer loaded successfully!");

        Ok((bert_model, tokenizer))
    }

    fn load_safetensors_weights(
        weights_file: &Path,
        device: &Device,
    ) -> Result<VarBuilder<'static>> {
        eprintln!("📦 Loading SafeTensors weights...");
        unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_file], DType::F32, device)
                .map_err(|e| anyhow::anyhow!("Failed to load safetensors: {}", e))
        }
    }

    fn load_pytorch_weights(weights_file: &Path, device: &Device) -> Result<VarBuilder<'static>> {
        eprintln!("📦 Loading PyTorch weights...");
        let weights_map = candle_core::pickle::read_all(weights_file)?;

        // Convert to device tensors
        let mut tensors: HashMap<String, Tensor> = HashMap::new();
        for (key, tensor) in weights_map {
            let device_tensor = tensor
                .to_device(device)
                .with_context(|| format!("Failed to move tensor {key} to device"))?;
            tensors.insert(key, device_tensor);
        }

        Ok(VarBuilder::from_tensors(tensors, DType::F32, device))
    }

    /// 🔤 Generate embedding for single text
    pub async fn embed_text(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        self.embed_text_production(text).await
    }

    async fn embed_text_production(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self.initialized.lock().await;

        // Tokenize with proper padding and truncation
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| format!("Tokenization failed: {e}"))?;

        // Prepare input tensors
        let input_ids = Tensor::new(encoding.get_ids(), &self.device)?.unsqueeze(0)?; // Add batch dimension

        let attention_mask =
            Tensor::new(encoding.get_attention_mask(), &self.device)?.unsqueeze(0)?; // Add batch dimension

        // Optional: token type IDs (for sentence pair tasks)
        let token_type_ids = Tensor::zeros((1, input_ids.dim(1)?), DType::U32, &self.device)?;

        // Forward pass through BERT
        let sequence_output = self
            .model
            .forward(&input_ids, &attention_mask, Some(&token_type_ids))
            .map_err(|e| format!("Model forward pass failed: {e}"))?;

        // Apply mean pooling to get sentence embedding
        let pooled_embedding = self.mean_pooling_production(&sequence_output, &attention_mask)?;

        // Normalize for cosine similarity (standard for sentence transformers)
        let normalized_embedding = self.normalize_embedding_production(&pooled_embedding)?;

        // Convert to CPU and extract vector
        let cpu_tensor = normalized_embedding.to_device(&Device::Cpu)?;
        let embedding_vec = cpu_tensor
            .squeeze(0)?
            .to_vec1::<f32>()
            .map_err(|e| format!("Failed to convert tensor to vector: {e}"))?;

        Ok(embedding_vec)
    }

    fn mean_pooling_production(
        &self,
        sequence_output: &Tensor,
        attention_mask: &Tensor,
    ) -> CandleResult<Tensor> {
        // Expand attention mask to match hidden dimension
        let mask_expanded = attention_mask
            .unsqueeze(2)?
            .expand((
                sequence_output.dim(0)?,
                sequence_output.dim(1)?,
                sequence_output.dim(2)?,
            ))?
            .to_dtype(DType::F32)?;

        // Apply mask to sequence output
        let masked_embeddings = sequence_output.broadcast_mul(&mask_expanded)?;

        // Sum over sequence length
        let sum_embeddings = masked_embeddings.sum(1)?;

        // Sum attention mask to get sequence lengths
        let sum_mask = attention_mask.sum(1)?.to_dtype(DType::F32)?.unsqueeze(1)?;

        // Avoid division by zero
        let sum_mask_clamped = sum_mask.clamp(1e-9f32, f32::INFINITY)?;

        // Compute mean by dividing sum by sequence length
        sum_embeddings.broadcast_div(&sum_mask_clamped)
    }

    fn normalize_embedding_production(&self, embedding: &Tensor) -> CandleResult<Tensor> {
        // Compute L2 norm
        let squared = embedding.sqr()?;
        let sum_squared = squared.sum_keepdim(1)?;
        let norm = sum_squared.sqrt()?;

        // Clamp to avoid division by zero
        let norm_clamped = norm.clamp(1e-12f32, f32::INFINITY)?;

        // Normalize
        embedding.broadcast_div(&norm_clamped)
    }

    /// 📦 Generate production embeddings for multiple texts with optimized batching
    pub async fn embed_batch(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }


        // 🔧 FIX: Use proper async processing instead of blocking operations
        let mut embeddings = Vec::with_capacity(texts.len());
        
        // Process each text individually to avoid deadlocks
        for (i, text) in texts.iter().enumerate() {
            
            if i > 0 && i % 5 == 0 {
                eprintln!("📊 Progress: {}/{} embeddings completed", i, texts.len());
            }
            
            // 🔧 FIXED: Use direct async call instead of blocking
            let embedding = self.embed_text(text).await?;
            embeddings.push(embedding);
            
            // Small delay to prevent overwhelming the model
            if texts.len() > 10 && i % 4 == 3 {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        eprintln!("✅ Batch complete: {} embeddings generated", embeddings.len());
        Ok(embeddings)
    }
}

/// 🔄 Native Rust embeddings backend (Python removed)
pub struct EmbeddingsBackend {
    native: NativeEmbedder,
}

impl EmbeddingsBackend {
    /// 🚀 Create native Rust embeddings client
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        eprintln!("🦀 Initializing native Rust embeddings...");
        let native = NativeEmbedder::new().await?;
        eprintln!("✅ Native Rust embeddings ready!");
        Ok(Self { native })
    }

    /// 🔤 Generate single embedding
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, String> {
        self.native
            .embed_text(text)
            .await
            .map_err(|e| format!("Native embedding error: {e}"))
    }

    /// 📦 Generate batch embeddings
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        self.native
            .embed_batch(texts)
            .await
            .map_err(|e| format!("Native batch embedding error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires model download
    async fn test_production_embedder_initialization() {
        let embedder = NativeEmbedder::new().await.unwrap();
        // Test that we can generate an embedding
        let embedding = embedder.embed_text("test").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    #[ignore] // Requires model download
    async fn test_production_embedding_dimensions() {
        let embedder = NativeEmbedder::new().await.unwrap();
        let embedding = embedder.embed_text("test sentence").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    #[ignore] // Requires model download
    async fn test_production_batch_embeddings() {
        let embedder = NativeEmbedder::new().await.unwrap();
        let texts = vec![
            "Hello world".to_string(),
            "Machine learning with Rust".to_string(),
            "Vector embeddings for semantic search".to_string(),
        ];
        let embeddings = embedder.embed_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        for embedding in &embeddings {
            assert_eq!(embedding.len(), 384);

            // Check normalization
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!(
                (norm - 1.0).abs() < 0.01,
                "Embedding not normalized: norm = {norm}"
            );
        }
    }

    #[tokio::test]
    #[ignore] // Requires model download
    async fn test_production_embedding_backend() {
        let backend = EmbeddingsBackend::new().await.unwrap();
        let embedding = backend.embed_text("backend test").await.unwrap();
        assert_eq!(embedding.len(), 384);
        eprintln!("Backend test completed successfully");
    }
}

#[cfg(test)]
mod compatibility_tests {
    use super::*;

    /// Test compatibility with Python sentence-transformers output
    #[tokio::test]
    #[ignore] // Requires both Python service and model download
    async fn test_compatibility_with_python() {
        // This test requires the Python service to be running
        let python_client = super::super::rag_client::EmbeddingsClient::new();
        let rust_embedder = NativeEmbedder::new().await.unwrap();

        let test_texts = vec![
            "Hello world",
            "Machine learning is fascinating",
            "Natural language processing with transformers",
            "Vector embeddings enable semantic search",
            "Rust is a systems programming language",
        ];

        for text in test_texts {
            eprintln!("Testing compatibility for: '{text}'");

            // Get embeddings from both implementations
            let python_embedding = python_client.embed_text(text).await.unwrap();
            let rust_embedding = rust_embedder.embed_text(text).await.unwrap();

            // Check dimensions match
            assert_eq!(python_embedding.len(), rust_embedding.len());
            assert_eq!(rust_embedding.len(), 384);

            // Compute cosine similarity between Python and Rust embeddings
            let cosine_sim = cosine_similarity(&python_embedding, &rust_embedding);

            eprintln!("  Cosine similarity: {cosine_sim:.6}");

            // Embeddings should be very similar (> 0.99 similarity)
            assert!(
                cosine_sim > 0.99,
                "Embeddings not compatible: similarity = {cosine_sim:.6} (expected > 0.99)"
            );
        }

        eprintln!("✅ All compatibility tests passed!");
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a > 0.0 && norm_b > 0.0 {
            dot_product / (norm_a * norm_b)
        } else {
            0.0
        }
    }
}
