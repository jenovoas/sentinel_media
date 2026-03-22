use reqwest::Client;
use serde_json::json;
use anyhow::Result;

pub struct HttpAuthClient {
    client: Client,
}

impl HttpAuthClient {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn synthesize_openai(&self, system_msg: &str, user_msg: &str, token: &str) -> Result<String> {
        // Endpoint reverse-engineered for ChatGPT (Subject to change)
        let url = "https://chatgpt.com/backend-api/conversation"; 
        
        // OpenAI requires extensive headers to mimic a browser. 
        // This implementation assumes the user provides a valid Session Token.
        // Note: CLOUDFLARE might block this without a proper PoW token or headless browser.
        
        let prompt = format!("{}\n\n{}", system_msg, user_msg);

        let body = json!({
            "action": "next",
            "messages": [
                {
                    "id": uuid::Uuid::new_v4().to_string(),
                    "author": { "role": "user" },
                    "content": { "content_type": "text", "parts": [prompt] },
                    "metadata": {}
                }
            ],
            "model": "text-davinci-002-render-sha", // Default model, might need update
            "parent_message_id": uuid::Uuid::new_v4().to_string()
        });

        let res = self.client.post(url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0")
            .header("Accept", "*/*")
            .header("Authorization", format!("Bearer {}", token)) // Sometimes Bearer, sometimes Cookie
            .header("Cookie", format!("__Secure-next-auth.session-token={}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
             let status = res.status();
             let err = res.text().await.unwrap_or_default();
             return Err(anyhow::anyhow!("OpenAI HTTP Error ({}): {}", status, err));
        }

        // Response is usually a stream of data: events.
        // For simplicity, we capture the raw text here, but parsing SSE is complex.
        let text = res.text().await?;
        Ok(text) 
    }

    pub async fn synthesize_antigravity(&self, system_msg: &str, user_msg: &str, token: &str) -> Result<String> {
        // Antigravity Internal Endpoint
        // Assuming a standard HTTP API secured by token
        let url = "http://localhost:3000/api/generate"; // Placeholder

        let body = json!({
            "system": system_msg,
            "prompt": user_msg,
        });

        let res = self.client.post(url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
             let status = res.status();
             let err = res.text().await.unwrap_or_default();
             return Err(anyhow::anyhow!("Antigravity HTTP Error ({}): {}", status, err));
        }

        let json: serde_json::Value = res.json().await?;
        if let Some(content) = json["response"].as_str() {
            Ok(content.to_string())
        } else {
             Ok(json.to_string())
        }
    }
}
