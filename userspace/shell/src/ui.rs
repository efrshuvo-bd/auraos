//! Text UI for AuraOS home + agent overlay.

pub fn print_boot_banner() {
    println!(
        r#"
╔══════════════════════════════════════╗
║              AuraOS                  ║
║     Agentic AI from the core         ║
╚══════════════════════════════════════╝
"#
    );
}

pub fn print_home() {
    println!(
        r#"
┌──────────── Home ────────────────────┐
│  AuraOS                              │
│  Tap Agent or type below             │
│                                      │
│  [ Agent ]   [ Status ]   [ Apps ]   │
└──────────────────────────────────────┘
"#
    );
}

pub fn print_agent_overlay_hint() {
    println!("Agent overlay: always available — type naturally or use help/status/services/echo.");
}

pub fn print_agent_reply(text: &str, tools: &[String]) {
    println!("┌─ Agent ─────────────────────────────");
    for line in text.lines() {
        println!("│ {line}");
    }
    if !tools.is_empty() {
        println!("│ tools: {}", tools.join(", "));
    }
    println!("└─────────────────────────────────────");
}
