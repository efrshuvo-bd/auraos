//! AuraOS shell — home composition + always-available agent overlay (serial/UI).

mod framebuffer;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use shared::ipc::{self, Message};
use shared::{SHELL_SERVICE, AURA_VERSION};
use std::io::{self, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "aura-shell", about = "AuraOS agentic shell")]
struct Args {
    /// Agent Core address
    #[arg(long, default_value = "127.0.0.1:7420")]
    agent: String,

    /// Also spawn Agent Core if not running (host convenience)
    #[arg(long, default_value_t = true)]
    auto_agent: bool,

    /// Write a PPM framebuffer snapshot of the home+agent UI
    #[arg(long, default_value = ".aura/shell-demo.ppm")]
    snapshot: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aura_shell=info".into()),
        )
        .init();

    let args = Args::parse();
    ui::print_boot_banner();
    framebuffer::render_home_with_agent(&args.snapshot)?;
    info!(path = %args.snapshot, "wrote framebuffer demo snapshot");

    let mut agent_child = None;
    let stream = match TcpStream::connect(&args.agent).await {
        Ok(s) => s,
        Err(_) if args.auto_agent => {
            info!("Agent Core not up — launching aura-agent");
            agent_child = Some(spawn_local_agent(&args.agent)?);
            wait_connect(&args.agent).await?
        }
        Err(e) => return Err(e).context("connect Agent Core"),
    };

    let mut stream = stream;
    handshake(&mut stream).await?;
    ui::print_home();
    println!("Agent overlay ready. Type a prompt (or /quit).\n");

    let mut req_id = 1u64;
    let stdin = io::stdin();
    loop {
        print!("aura› ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let prompt = line.trim();
        if prompt.is_empty() {
            continue;
        }
        if prompt == "/quit" || prompt == "/exit" {
            break;
        }
        if prompt == "/home" {
            ui::print_home();
            continue;
        }
        if prompt == "/agent" {
            ui::print_agent_overlay_hint();
            continue;
        }

        write_msg(
            &mut stream,
            &Message::AgentRequest {
                id: req_id,
                prompt: prompt.to_string(),
            },
        )
        .await?;

        let resp = read_until_response(&mut stream, req_id).await?;
        match resp {
            Message::AgentResponse {
                text, tools_used, ..
            } => {
                ui::print_agent_reply(&text, &tools_used);
            }
            Message::Error { message, .. } => {
                println!("! agent error: {message}");
            }
            other => println!("! unexpected: {other:?}"),
        }
        req_id += 1;
    }

    if let Some(mut child) = agent_child {
        let _ = child.kill();
    }
    Ok(())
}

fn spawn_local_agent(addr: &str) -> Result<std::process::Child> {
    let bin = find_bin("aura-agent")?;
    std::process::Command::new(bin)
        .arg("--listen")
        .arg(addr)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .context("spawn aura-agent")
}

fn find_bin(name: &str) -> Result<std::path::PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join(format!("{name}.exe"));
            if p.exists() {
                return Ok(p);
            }
            let p = dir.join(name);
            if p.exists() {
                return Ok(p);
            }
        }
    }
    for rel in [
        format!("target/debug/{name}.exe"),
        format!("target/debug/{name}"),
        format!("target/release/{name}.exe"),
        format!("target/release/{name}"),
    ] {
        let p = std::path::PathBuf::from(rel);
        if p.exists() {
            return Ok(p);
        }
    }
    anyhow::bail!("build workspace first so {name} exists next to the shell")
}

async fn wait_connect(addr: &str) -> Result<TcpStream> {
    for _ in 0..50 {
        if let Ok(s) = TcpStream::connect(addr).await {
            return Ok(s);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    anyhow::bail!("Agent Core did not start at {addr}")
}

async fn handshake(stream: &mut TcpStream) -> Result<()> {
    write_msg(
        stream,
        &Message::Hello {
            from: SHELL_SERVICE.into(),
            version: AURA_VERSION.into(),
        },
    )
    .await?;
    // Drain hello from agent / ack
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    for _ in 0..10 {
        let n = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            stream.read(&mut tmp),
        )
        .await;
        match n {
            Ok(Ok(0)) => break,
            Ok(Ok(n)) => {
                buf.extend_from_slice(&tmp[..n]);
                while let Some((msg, c)) = ipc::try_decode(&buf)? {
                    buf.drain(..c);
                    match msg {
                        Message::Hello { .. } | Message::HelloAck { .. } => {
                            info!("IPC handshake complete");
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
            _ => break,
        }
    }
    info!("continuing without strict handshake");
    Ok(())
}

async fn write_msg(stream: &mut TcpStream, msg: &Message) -> Result<()> {
    let bytes = ipc::encode(msg)?;
    stream.write_all(&bytes).await?;
    Ok(())
}

async fn read_until_response(stream: &mut TcpStream, id: u64) -> Result<Message> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            anyhow::bail!("agent closed connection");
        }
        buf.extend_from_slice(&tmp[..n]);
        while let Some((msg, c)) = ipc::try_decode(&buf)? {
            buf.drain(..c);
            match &msg {
                Message::AgentResponse { id: rid, .. } if *rid == id => return Ok(msg),
                Message::Error { id: Some(rid), .. } if *rid == id => return Ok(msg),
                Message::Hello { .. } | Message::HelloAck { .. } => {}
                Message::Error { id: None, .. } => return Ok(msg),
                _ => {}
            }
        }
    }
}
