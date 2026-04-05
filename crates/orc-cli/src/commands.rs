pub enum SlashCommand {
    Help,
    Clear,
    Compact,
    Exit,
    Unknown(String),
}

pub fn parse(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let cmd = trimmed.split_whitespace().next().unwrap_or("");
    match cmd {
        "/help" => Some(SlashCommand::Help),
        "/clear" => Some(SlashCommand::Clear),
        "/compact" => Some(SlashCommand::Compact),
        "/exit" | "/quit" => Some(SlashCommand::Exit),
        other => Some(SlashCommand::Unknown(other.to_string())),
    }
}

pub fn print_help() {
    println!("\n  Available commands:");
    println!("    /help     Show this help message");
    println!("    /clear    Clear conversation history");
    println!("    /compact  Summarize and compact conversation");
    println!("    /exit     Exit orc");
    println!();
}
