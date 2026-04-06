pub enum SlashCommand {
    Exit,
    Status,
    Help,
    Passthrough(String),
}

pub fn parse(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let cmd = trimmed.split_whitespace().next().unwrap_or("");
    match cmd {
        "/exit" | "/quit" => Some(SlashCommand::Exit),
        "/status" => Some(SlashCommand::Status),
        "/help" => Some(SlashCommand::Help),
        _ => Some(SlashCommand::Passthrough(trimmed.to_string())),
    }
}

pub fn print_help(available: &[String]) {
    println!("\n  orc commands:");
    println!("    /exit     Exit orc");
    println!("    /status   Show session stats");
    println!("    /help     Show this help");

    if !available.is_empty() {
        println!("\n  Claude Code commands:");
        for cmd in available {
            println!("    /{cmd}");
        }
    }

    println!();
}
