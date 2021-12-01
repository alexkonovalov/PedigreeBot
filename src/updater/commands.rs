use strum_macros::EnumString;
use strum_macros::Display;

#[derive(EnumString, Display)]
pub enum ButtonCommand {
    No,
}

pub enum OutputCommand {
     Prompt(String),
     PromptButtons(Vec<(ButtonCommand, String)>, String)
}
