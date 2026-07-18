//! AuraOS init — Agent Core is required; shell follows.

use anyhow::{bail, Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "aura-init", about = "AuraOS PID 1")]
struct Args {
    /// Path to aura-agent binary
    #[arg(long, env = "AURA_AGENT_BIN")]
    agent_bin: Option<PathBuf>,

    /// Path to aura-shell binary
    #[arg(long, env = "AURA_SHELL_BIN")]
    shell_bin: Option<PathBuf>,

    /// Agent listen address
    #[arg(long, default_value = "127.0.0.1:7420")]
    agent_addr: String,

    /// If set, do not start shell (agent only)
    #[arg(long)]
    agent_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aura_init=info".into()),
        )
        .init();

    let args = Args::parse();
    info!("AuraOS init online (PID 1 host demo)");

    let agent_bin = resolve_bin(args.agent_bin, "aura-agent")?;
    let mut agent = spawn_agent(&agent_bin, &args.agent_addr)?;
    info!(path = %agent_bin.display(), "started Agent Core");

    sleep(Duration::from_millis(400)).await;
    if let Some(status) = agent.try_wait()? {
        error!(%status, "Agent Core exited — init failing closed");
        bail!("Agent Core is required and failed to start");
    }

    wait_for_agent(&args.agent_addr).await?;

    if args.agent_only {
        info!("agent_only: waiting on Agent Core");
        let status = agent.wait().await?;
        bail!("Agent Core exited: {status}");
    }

    let shell_bin = resolve_bin(args.shell_bin, "aura-shell")?;
    info!(path = %shell_bin.display(), "starting shell");
    let mut shell = Command::new(&shell_bin)
        .arg("--agent")
        .arg(&args.agent_addr)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("spawn shell {}", shell_bin.display()))?;

    let shell_status = shell.wait().await?;
    info!(%shell_status, "shell exited");

    let _ = agent.kill().await;
    Ok(())
}

fn resolve_bin(explicit: Option<PathBuf>, name: &str) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(name));
            candidates.push(dir.join(format!("{name}.exe")));
        }
    }
    candidates.push(PathBuf::from("target/debug").join(name));
    candidates.push(PathBuf::from("target/debug").join(format!("{name}.exe")));
    candidates.push(PathBuf::from("target/release").join(name));
    candidates.push(PathBuf::from("target/release").join(format!("{name}.exe")));

    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }
    bail!("could not find {name}; pass --agent-bin/--shell-bin or build the workspace");
}

fn spawn_agent(bin: &PathBuf, addr: &str) -> Result<Child> {
    Command::new(bin)
        .arg("--listen")
        .arg(addr)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("spawn agent {}", bin.display()))
}

async fn wait_for_agent(addr: &str) -> Result<()> {
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            info!(%addr, "Agent Core ready");
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }
    bail!("Agent Core did not become ready at {addr}");
}
