#[derive(Debug)]
pub enum OutputCommand {
    Prompt(String),
    PromptButtons(Vec<(ButtonCommand, String)>, String)
}