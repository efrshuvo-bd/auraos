//! Persistent session memory for Agent Core.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AgentMemory {
    pub turns: Vec<Turn>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Turn {
    pub user: String,
    pub agent: String,
}

impl AgentMemory {
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if path.exists() {
            let data = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn remember_turn(&mut self, user: &str, agent: &str) {
        self.turns.push(Turn {
            user: user.to_string(),
            agent: agent.to_string(),
        });
        if self.turns.len() > 64 {
            let drain = self.turns.len() - 64;
            self.turns.drain(0..drain);
        }
    }

    pub fn recent_context(&self, n: usize) -> String {
        self.turns
            .iter()
            .rev()
            .take(n)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|t| format!("U: {}\nA: {}", t.user, t.agent))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }
}
