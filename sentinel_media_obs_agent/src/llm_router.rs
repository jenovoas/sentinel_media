use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::{Result, bail};

#[derive(Serialize, Deserialize, Debug)]
struct VertexRequest {
    contents: Vec<VertexContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<VertexContent>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VertexContent {
    parts: Vec<VertexPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VertexPart {
    text: String,
}

#[derive(Deserialize, Debug)]
struct VertexResponse {
    candidates: Vec<VertexCandidate>,
}

#[derive(Deserialize, Debug)]
struct VertexCandidate {
    content: VertexContentResponse,
}

#[derive(Deserialize, Debug)]
struct VertexContentResponse {
    parts: Vec<VertexPart>,
}

#[derive(Debug, PartialEq)]
pub enum Intent {
    Research(String),
    Memorize(String),
    Produce(String),
    Unknown,
}

pub async fn classify_intent(prompt: &str, api_key: &str) -> Result<Intent> {
    let client = Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let system_msg = r#"
    You are the Semantic Router for the Sentinel Swarm Control Room.
    Your job is to map natural language user requests to specific agent commands.

    AVAILABLE COMMANDS:
    1. Research: For requests to investigate, learn, search, study, or analyze a topic.
    2. Memorize: For requests to remember, store, save facts, or learn rules.
    3. Produce: For requests to create videos, media, reports, or artifacts.

    OUTPUT FORMAT:
    Return a JSON object with "action" and "parameter".
    - action: "RESEARCH", "MEMORIZE", "PRODUCE", or "UNKNOWN"
    - parameter: The extracted subject/topic without verbs.

    Examples:
    - User: "Investiga sobre la fusión nuclear" -> {"action": "RESEARCH", "parameter": "fusión nuclear"}
    - User: "Recuerda que el servidor es azul" -> {"action": "MEMORIZE", "parameter": "el servidor es azul"}
    - User: "Analiza el mercado de cripto" -> {"action": "RESEARCH", "parameter": "mercado de cripto"}
    - User: "Hola que tal" -> {"action": "UNKNOWN", "parameter": ""}
    
    RETURN ONLY JSON. NO MARKDOWN.
    "#;

    let req = VertexRequest {
        contents: vec![VertexContent {
            role: Some("user".to_string()),
            parts: vec![VertexPart { text: prompt.to_string() }],
        }],
        system_instruction: Some(VertexContent {
            role: None,
            parts: vec![VertexPart { text: system_msg.to_string() }],
        }),
    };

    let res = client.post(url).json(&req).send().await?;
    
    if !res.status().is_success() {
        let err = res.text().await?;
        bail!("Gemini API Error: {}", err);
    }

    let res_json = res.json::<VertexResponse>().await?;
    let raw_text = res_json.candidates.first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.trim())
        .unwrap_or("{}");

    // Clean JSON
    let cleaned = raw_text.trim_matches('`').replace("json", "").trim().to_string();

    #[derive(Deserialize)]
    struct RouterResponse {
        action: String,
        parameter: String,
    }

    let parsed: RouterResponse = serde_json::from_str(&cleaned).unwrap_or(RouterResponse {
        action: "UNKNOWN".to_string(),
        parameter: "".to_string(),
    });

    match parsed.action.to_uppercase().as_str() {
        "RESEARCH" => Ok(Intent::Research(parsed.parameter)),
        "MEMORIZE" => Ok(Intent::Memorize(parsed.parameter)),
        "PRODUCE" => Ok(Intent::Produce(parsed.parameter)),
        _ => Ok(Intent::Unknown),
    }
}
