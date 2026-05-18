use reqwest::blocking::Client;
use serde_json::json;
use anyhow::Result;
use std::time::Duration;

pub fn explain_code(snippet: &str, model: &str, ollama_url: &str) -> Result<String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;
    let prompt = format!(
        "You are a helpful coding assistant. Explain the following code simply, as if to a junior developer:\n\n```\n{}\n```\n\nExplanation:",
        snippet
    );
    let request = json!({
        "model": model,
        "prompt": prompt,
        "stream": false
    });
    let response = client
        .post(format!("{}/api/generate", ollama_url))
        .json(&request)
        .send()?;
    let json_response = response.json::<serde_json::Value>()?;
    let explanation = json_response["response"]
        .as_str()
        .unwrap_or("No explanation generated")
        .to_string();
    Ok(explanation)
}