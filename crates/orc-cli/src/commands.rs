pub enum SlashCommand {
    Exit,
    Status,
    Cost,
    Help,
    Model(String),
    Effort(String),
    Passthrough(String),
}

pub fn parse(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();

    match cmd {
        "/exit" | "/quit" => Some(SlashCommand::Exit),
        "/status" => Some(SlashCommand::Status),
        "/cost" => Some(SlashCommand::Cost),
        "/help" => Some(SlashCommand::Help),
        "/model" if !arg.is_empty() => Some(SlashCommand::Model(arg.to_string())),
        "/model" => Some(SlashCommand::Model(String::new())),
        "/effort" if !arg.is_empty() => Some(SlashCommand::Effort(arg.to_string())),
        "/effort" => Some(SlashCommand::Effort(String::new())),
        _ => Some(SlashCommand::Passthrough(trimmed.to_string())),
    }
}

pub fn print_help(available: &[String]) {
    println!("\n  orc commands:");
    println!("    /model <name>   Switch model (opus, sonnet, haiku)");
    println!("    /effort <level> Set effort (low, medium, high, max)");
    println!("    /cost           Show session cost summary");
    println!("    /status         Show session info");
    println!("    /exit           Exit orc");
    println!("    /help           Show this help");

    if !available.is_empty() {
        println!("\n  Claude Code commands:");
        for cmd in available {
            println!("    /{cmd}");
        }
    }

    println!();
}
