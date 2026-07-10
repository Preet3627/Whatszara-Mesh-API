use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
}

#[async_trait::async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, messages: &[LLMMessage], system_prompt: Option<&str>) -> Result<LLMResponse, String>;
    async fn list_models(&self) -> Result<Vec<String>, String>;
    fn default_model(&self) -> &str;
    fn set_model(&mut self, model: &str);
    fn current_model(&self) -> &str;
    fn set_endpoint(&mut self, _endpoint: &str) {}
    fn set_api_key(&mut self, _key: &str) {}
}

// ── Mesh API ────────────────────────────────────
// Mesh API (https://developers.meshapi.ai) is an AI router that provides
// access to 300+ models through a single OpenAI-compatible endpoint.
// It supports BYOK (Bring Your Own Key) for upstream providers.
//
// Docs: https://developers.meshapi.ai/docs/guides/quickstart
// Auth: Bearer rsk_... (Router Service Key)
// BYOK: Provider keys stored securely, Mesh routes to best provider
// Endpoint: https://api.meshapi.ai/v1
pub struct MeshApiProvider {
    pub api_key: String,
    pub model: String,
    pub endpoint: String,
    // BYOK - Bring Your Own upstream provider keys
    pub openai_key: String,
    pub anthropic_key: String,
    pub groq_key: String,
}

#[async_trait::async_trait]
impl LLMProvider for MeshApiProvider {
    fn name(&self) -> &str { "mesh-api" }
    fn current_model(&self) -> &str { &self.model }
    fn set_model(&mut self, model: &str) { self.model = model.to_string(); }
    fn set_endpoint(&mut self, endpoint: &str) { self.endpoint = endpoint.to_string(); }
    fn set_api_key(&mut self, key: &str) { self.api_key = key.to_string(); }

    async fn chat(&self, messages: &[LLMMessage], system_prompt: Option<&str>) -> Result<LLMResponse, String> {
        let client = reqwest::Client::new();
        let mut payload_messages: Vec<serde_json::Value> = messages.iter().map(|m| {
            serde_json::json!({"role": m.role, "content": m.content})
        }).collect();

        if let Some(sp) = system_prompt {
            payload_messages.insert(0, serde_json::json!({"role": "system", "content": sp}));
        }

        let mut payload = serde_json::json!({
            "model": self.model,
            "messages": payload_messages,
        });

        let mut headers = reqwest::header::HeaderMap::new();
        let auth = format!("Bearer {}", self.api_key);
        headers.insert("Authorization", reqwest::header::HeaderValue::from_str(&auth).unwrap());

        // BYOK headers - pass upstream provider keys if configured
        if !self.openai_key.is_empty() {
            headers.insert("x-mesh-openai-key", reqwest::header::HeaderValue::from_str(&self.openai_key).unwrap());
        }
        if !self.anthropic_key.is_empty() {
            headers.insert("x-mesh-anthropic-key", reqwest::header::HeaderValue::from_str(&self.anthropic_key).unwrap());
        }
        if !self.groq_key.is_empty() {
            headers.insert("x-mesh-groq-key", reqwest::header::HeaderValue::from_str(&self.groq_key).unwrap());
        }

        let resp = client.post(format!("{}/chat/completions", self.endpoint))
            .headers(headers)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| format!("Mesh API request failed: {}", e))?;

        let data: serde_json::Value = resp.json().await.map_err(|e| format!("Mesh API parse failed: {}", e))?;

        let content = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
        let model_used = data["model"].as_str().unwrap_or(&self.model).to_string();
        Ok(LLMResponse { content, model: model_used, provider: "mesh-api".into() })
    }

    async fn list_models(&self) -> Result<Vec<String>, String> {
        let client = reqwest::Client::new();
        let resp = client.get(format!("{}/models", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|_| "Mesh API not available".to_string())?;

        let data: serde_json::Value = resp.json().await.map_err(|_| "Bad response".to_string())?;
        let models = data["data"].as_array().unwrap_or(&vec![]).iter()
            .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
            .collect();
        Ok(models)
    }

    fn default_model(&self) -> &str {
        if self.model.is_empty() { "openai/gpt-4o" } else { &self.model }
    }
}

// ── Provider Registry ─────────────────────────
pub struct ProviderRegistry {
    pub providers: Vec<Box<dyn LLMProvider>>,
    pub active: usize,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: vec![], active: 0 }
    }

    pub fn register(&mut self, provider: Box<dyn LLMProvider>) {
        self.providers.push(provider);
    }

    pub fn active_provider(&self) -> &dyn LLMProvider {
        self.providers[self.active].as_ref()
    }

    pub fn set_active(&mut self, name: &str) -> Result<(), String> {
        for (i, p) in self.providers.iter().enumerate() {
            if p.name() == name {
                self.active = i;
                return Ok(());
            }
        }
        Err(format!("Provider '{}' not found", name))
    }

    pub fn list_names(&self) -> Vec<String> {
        self.providers.iter().map(|p| p.name().to_string()).collect()
    }

    pub fn set_model(&mut self, provider: &str, model: &str) -> Result<(), String> {
        for p in &mut self.providers {
            if p.name() == provider {
                p.set_model(model);
                return Ok(());
            }
        }
        Err(format!("Provider '{}' not found", provider))
    }

    pub async fn list_all_models(&self) -> Vec<(String, Vec<String>, String)> {
        let mut results = vec![];
        for p in &self.providers {
            let models = p.list_models().await.unwrap_or_else(|_| vec![p.default_model().to_string()]);
            let current = p.current_model().to_string();
            let effective = if current.is_empty() { models.first().cloned().unwrap_or_else(|| current) } else { current };
            results.push((p.name().to_string(), models, effective));
        }
        results
    }

    pub async fn chat(&self, messages: &[LLMMessage], system_prompt: Option<&str>) -> Result<LLMResponse, String> {
        if self.providers.is_empty() {
            return Err("No LLM providers configured".into());
        }
        self.active_provider().chat(messages, system_prompt).await
    }
}
