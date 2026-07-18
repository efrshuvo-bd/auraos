//! AuraOS Agent Core — privileged userspace AI service.

mod backend;
mod memory;
mod tools;

use anyhow::Result;
use clap::Parser;
use shared::ipc::{self, Message};
use shared::{AGENT_SERVICE, AURA_VERSION};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "aura-agent", about = "AuraOS Agent Core")]
struct Args {
    /// Listen address for shell/init IPC
    #[arg(long, default_value = "127.0.0.1:7420")]
    listen: String,

    /// Persist agent memory here
    #[arg(long, default_value = ".aura/agent-memory.json")]
    memory: PathBuf,

    /// OpenAI-compatible API base URL
    #[arg(long, env = "AURA_LLM_BASE_URL")]
    llm_base_url: Option<String>,

    /// API key for cloud LLM
    #[arg(long, env = "AURA_LLM_API_KEY")]
    llm_api_key: Option<String>,

    /// Model id
    #[arg(long, env = "AURA_LLM_MODEL", default_value = "gpt-4o-mini")]
    llm_model: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aura_agent=info".into()),
        )
        .init();

    let args = Args::parse();
    let mem = Arc::new(Mutex::new(memory::AgentMemory::load_or_default(&args.memory)?));
    let backend = backend::ModelBackend::from_env(
        args.llm_base_url.clone(),
        args.llm_api_key.clone(),
        args.llm_model.clone(),
    );

    info!(
        service = AGENT_SERVICE,
        version = AURA_VERSION,
        listen = %args.listen,
        "Agent Core starting (required system service)"
    );

    let listener = TcpListener::bind(&args.listen).await?;
    info!("Agent Core listening on {}", args.listen);

    loop {
        let (stream, peer) = listener.accept().await?;
        info!(%peer, "client connected");
        let mem = mem.clone();
        let backend = backend.clone();
        let memory_path = args.memory.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, mem, backend, memory_path).await {
                warn!("client error: {e:#}");
            }
        });
    }
}

async fn handle_client(
    mut stream: TcpStream,
    mem: Arc<Mutex<memory::AgentMemory>>,
    backend: backend::ModelBackend,
    memory_path: PathBuf,
) -> Result<()> {
    let hello = Message::Hello {
        from: AGENT_SERVICE.into(),
        version: AURA_VERSION.into(),
    };
    write_msg(&mut stream, &hello).await?;

    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    let mut next_id = 1u64;

    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        while let Some((msg, consumed)) = ipc::try_decode(&buf)? {
            buf.drain(..consumed);
            match msg {
                Message::Hello { from, .. } => {
                    info!(%from, "hello from client");
                    write_msg(
                        &mut stream,
                        &Message::HelloAck {
                            from: AGENT_SERVICE.into(),
                        },
                    )
                    .await?;
                }
                Message::AgentRequest { id, prompt } => {
                    let (text, tools_used) =
                        answer(&prompt, id, &mem, &backend, &mut next_id).await?;
                    {
                        let mut m = mem.lock().await;
                        m.remember_turn(&prompt, &text);
                        m.save(&memory_path)?;
                    }
                    write_msg(
                        &mut stream,
                        &Message::AgentResponse {
                            id,
                            text,
                            tools_used,
                        },
                    )
                    .await?;
                }
                other => {
                    write_msg(
                        &mut stream,
                        &Message::Error {
                            id: None,
                            message: format!("unsupported: {other:?}"),
                        },
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}

async fn answer(
    prompt: &str,
    _id: u64,
    mem: &Arc<Mutex<memory::AgentMemory>>,
    backend: &backend::ModelBackend,
    _next_id: &mut u64,
) -> Result<(String, Vec<String>)> {
    let trimmed = prompt.trim();
    let lower = trimmed.to_ascii_lowercase();

    // Local tool routing for built-ins (always available offline).
    if lower == "help" || lower.starts_with("help ") {
        let out = tools::run_help();
        return Ok((out, vec!["help".into()]));
    }
    if lower == "status" || lower == "system_status" || lower.starts_with("status ") {
        let out = tools::run_system_status(mem).await;
        return Ok((out, vec!["system_status".into()]));
    }
    if lower == "services" || lower == "list_services" {
        let out = tools::run_list_services();
        return Ok((out, vec!["list_services".into()]));
    }
    if let Some(rest) = lower.strip_prefix("echo ") {
        let out = tools::run_echo(rest);
        return Ok((out, vec!["echo".into()]));
    }
    if lower == "echo" {
        return Ok((tools::run_echo(""), vec!["echo".into()]));
    }

    // Cloud / mock LLM for free-form prompts.
    let history = {
        let m = mem.lock().await;
        m.recent_context(6)
    };
    let text = backend.complete(&history, trimmed).await?;
    Ok((text, vec![]))
}

async fn write_msg(stream: &mut TcpStream, msg: &Message) -> Result<()> {
    let bytes = ipc::encode(msg)?;
    stream.write_all(&bytes).await?;
    Ok(())
}
