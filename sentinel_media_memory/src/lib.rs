use anyhow::{Error as E, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use hf_hub::api::sync::Api;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokenizers::{PaddingParams, Tokenizer};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Document {
    pub path: String,
    pub content: String,
    pub vector: Vec<f32>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct VectorStore {
    pub documents: Vec<Document>,
}

impl VectorStore {
    pub fn load(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let data = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn add(&mut self, doc: Document) {
        self.documents.retain(|d| d.path != doc.path);
        self.documents.push(doc);
    }

    pub fn search(&self, query_vec: &[f32], limit: usize) -> Vec<(&Document, f32)> {
        let mut scored: Vec<_> = self
            .documents
            .iter()
            .map(|doc| {
                let similarity = cosine_similarity(&doc.vector, query_vec);
                (doc, similarity)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.into_iter().take(limit).collect()
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

pub struct CandleEmbedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl CandleEmbedder {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;
        let api = Api::new()?;
        let model_id = "sentence-transformers/all-MiniLM-L6-v2";
        let repo = api.model(model_id.to_string());

        let config_filename = repo.get("config.json")?;
        let weights_filename = repo.get("model.safetensors")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;

        let config: Config = serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, &device)?
        };
        let model = BertModel::load(vb, &config)?;
        let mut tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

        if let Some(pp) = tokenizer.get_padding_mut() {
            pp.strategy = tokenizers::PaddingStrategy::BatchLongest
        } else {
            let pp = PaddingParams {
                strategy: tokenizers::PaddingStrategy::BatchLongest,
                ..Default::default()
            };
            tokenizer.with_padding(Some(pp));
        }

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        let tokens = self
            .tokenizer
            .encode_batch(vec![text], true)
            .map_err(E::msg)?;
        let token_ids = tokens
            .iter()
            .map(|t| Tensor::new(t.get_ids(), &self.device).unwrap())
            .collect::<Vec<_>>();
        let token_ids = Tensor::stack(&token_ids, 0)?;
        let token_type_ids = token_ids.zeros_like()?;

        let embeddings = self.model.forward(&token_ids, &token_type_ids, None)?;

        let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()?;
        let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;
        let embeddings = normalize_l2(&embeddings)?;

        let vec = embeddings.get(0)?.to_vec1::<f32>()?;
        Ok(vec)
    }
}

fn normalize_l2(v: &Tensor) -> Result<Tensor> {
    let norm = v.sqr()?.sum_keepdim(1)?.sqrt()?;
    Ok((v.broadcast_div(&norm))?)
}
