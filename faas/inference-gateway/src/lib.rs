#[cfg(target_arch = "wasm32")]
mod component {
    wit_bindgen::generate!({
        path: "wit",
        world: "inference-gateway-world",
    });

    use exports::tachyon::ai::inference::{Guest, InferenceRequest, InferenceResponse};
    use tachyon::ai::nn_runtime;

    struct InferenceGateway;

    impl Guest for InferenceGateway {
        fn generate(req: InferenceRequest) -> Result<InferenceResponse, String> {
            let request = super::GenerationRequest {
                model_id: req.model_id,
                prompt: req.prompt,
                max_tokens: req.max_tokens,
                temperature: req.temperature,
                lora_adapter: req.lora_adapter,
            };

            let metadata =
                super::build_runtime_metadata(&request).map_err(|err| err.to_string())?;
            let input = super::encode_prompt_tensor(&request)?;
            let context = nn_runtime::load_graph(&request.model_id, &metadata)?;
            context.set_input(0, &input.dimensions, &input.data)?;
            context.compute()?;
            let output = context.get_output(0)?;
            let text = super::decode_output_tensor(&output)?;

            Ok(InferenceResponse {
                prompt_tokens: super::estimate_tokens(&request.prompt),
                completion_tokens: super::estimate_tokens(&text),
                text,
            })
        }
    }

    export!(InferenceGateway);
}

pub struct GenerationRequest {
    pub model_id: String,
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub lora_adapter: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PromptTensor {
    pub dimensions: Vec<u32>,
    pub data: Vec<u8>,
}

pub fn build_runtime_metadata(req: &GenerationRequest) -> anyhow::Result<String> {
    let metadata = serde_json::json!({
        "max_tokens": req.max_tokens,
        "temperature": req.temperature,
        "lora_adapter": req.lora_adapter,
    });

    Ok(serde_json::to_string(&metadata)?)
}

pub fn encode_prompt_tensor(req: &GenerationRequest) -> Result<PromptTensor, String> {
    if req.model_id.trim().is_empty() {
        return Err("model-id must not be empty".to_string());
    }
    if req.prompt.is_empty() {
        return Err("prompt must not be empty".to_string());
    }

    let len = u32::try_from(req.prompt.len())
        .map_err(|_| "prompt is too large to encode as a wasi-nn tensor".to_string())?;

    Ok(PromptTensor {
        dimensions: vec![1, len],
        data: req.prompt.as_bytes().to_vec(),
    })
}

pub fn decode_output_tensor(bytes: &[u8]) -> Result<String, String> {
    String::from_utf8(bytes.to_vec()).map_err(|_| "model output was not valid UTF-8".to_string())
}

pub fn estimate_tokens(text: &str) -> u32 {
    u32::try_from(text.len() / 4).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> GenerationRequest {
        GenerationRequest {
            model_id: "qwen-coder-27b".to_string(),
            prompt: "Write a short answer".to_string(),
            max_tokens: 128,
            temperature: 0.2,
            lora_adapter: Some("uncertainty-v1".to_string()),
        }
    }

    #[test]
    fn metadata_contains_generation_options_and_adapter() {
        let metadata = build_runtime_metadata(&request()).unwrap();

        assert!(metadata.contains("\"max_tokens\":128"));
        assert!(metadata.contains("\"temperature\":0.2"));
        assert!(metadata.contains("\"lora_adapter\":\"uncertainty-v1\""));
    }

    #[test]
    fn prompt_tensor_uses_utf8_bytes() {
        let tensor = encode_prompt_tensor(&request()).unwrap();

        assert_eq!(tensor.dimensions, vec![1, 20]);
        assert_eq!(tensor.data, b"Write a short answer");
    }

    #[test]
    fn prompt_tensor_rejects_missing_model() {
        let mut req = request();
        req.model_id.clear();

        assert_eq!(
            encode_prompt_tensor(&req).unwrap_err(),
            "model-id must not be empty"
        );
    }

    #[test]
    fn output_tensor_decodes_utf8() {
        assert_eq!(decode_output_tensor(b"done").unwrap(), "done");
    }
}
