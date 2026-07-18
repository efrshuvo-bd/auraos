//! Pluggable model backends: cloud OpenAI-compatible API or local mock.

use anyhow::{Context, Result};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Method, Request, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use serde_json::json;
use tracing::info;

type HttpClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;

fn new_http_client() -> HttpClient {
    Client::builder(TokioExecutor::new()).build(HttpsConnector::new())
}

#[derive(Clone)]
pub enum ModelBackend {
    Mock,
    OpenAiCompat {
        base_url: String,
        api_key: String,
        model: String,
        client: HttpClient,
    },
}

impl ModelBackend {
    pub fn from_env(base_url: Option<String>, api_key: Option<String>, model: String) -> Self {
        match (base_url, api_key) {
            (Some(base), Some(key)) if !base.is_empty() && !key.is_empty() => {
                info!(%model, "using OpenAI-compatible cloud backend");
                Self::OpenAiCompat {
                    base_url: base.trim_end_matches('/').to_string(),
                    api_key: key,
                    model,
                    client: new_http_client(),
                }
            }
            _ => {
                info!("using mock LLM backend (set AURA_LLM_API_KEY for cloud)");
                Self::Mock
            }
        }
    }

    pub async fn complete(&self, history: &str, prompt: &str) -> Result<String> {
        match self {
            Self::Mock => Ok(format!(
                "Aura Agent (mock): I heard \"{prompt}\". Try `help`, `status`, `services`, or `echo ...`. Context:\n{history}"
            )),
            Self::OpenAiCompat {
                base_url,
                api_key,
                model,
                client,
            } => {
                let url = format!("{base_url}/chat/completions");
                let body = json!({
                    "model": model,
                    "messages": [
                        {
                            "role": "system",
                            "content": "You are AuraOS Agent Core, the privileged system AI on an agentic mobile OS. Prefer concise answers. Suggest built-in tools when useful: help, status, services, echo."
                        },
                        {
                            "role": "user",
                            "content": format!("Recent context:\n{history}\n\nUser: {prompt}")
                        }
                    ]
                });
                let uri: Uri = url.parse().context("LLM URL")?;
                let payload = serde_json::to_string(&body)?;
                let req = Request::builder()
                    .method(Method::POST)
                    .uri(uri)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(payload)))?;
                let resp = client.request(req).await.context("LLM HTTP request")?;
                let status = resp.status();
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let val: serde_json::Value =
                    serde_json::from_slice(&body_bytes).context("LLM JSON")?;
                if !status.is_success() {
                    anyhow::bail!("LLM error {status}: {val}");
                }
                let text = val["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("(empty model response)")
                    .to_string();
                Ok(text)
            }
        }
    }
}
