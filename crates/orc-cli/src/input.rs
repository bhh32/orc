use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

pub struct InputReader {
    editor: Reedline,
    prompt: DefaultPrompt,
}

impl InputReader {
    pub fn new() -> Self {
        let editor = Reedline::create();
        let prompt = DefaultPrompt::new(
            DefaultPromptSegment::Basic("orc".to_string()),
            DefaultPromptSegment::Empty,
        );

        Self { editor, prompt }
    }

    pub fn read_line(&mut self) -> ReadResult {
        match self.editor.read_line(&self.prompt) {
            Ok(Signal::Success(line)) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    ReadResult::Empty
                } else {
                    ReadResult::Input(trimmed)
                }
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => ReadResult::Exit,
            Err(e) => ReadResult::Error(e.to_string()),
        }
    }
}

pub enum ReadResult {
    Input(String),
    Empty,
    Exit,
    Error(String),
}
