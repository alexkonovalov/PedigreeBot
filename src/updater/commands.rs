use strum_macros::EnumString;
use std::str::FromStr;
use std::string::{ToString};
use strum_macros::Display;
use petgraph::graph::{NodeIndex};

pub enum UpdaterCommand {
    ReplyChild(NodeIndex<u32>),
    CreateChild(NodeIndex<u32>)
    // CreateRelative(NodeIndex<u32>, String),
    // CreateLink(NodeIndex<u32>, NodeIndex<u32>, String, String),
    // ContinueOrSwitch(NodeIndex<u32>, String),
    // CheckIfChildren(NodeIndex<u32>, String),
}

// #[derive(EnumString, Display)]
// pub enum Relation {
//     Parent,
//     Child
// }

// #[derive(EnumString, Display)]
// pub enum YesNo {
//     Yes,
//     No
// }

// #[derive(EnumString, Display)]
// pub enum NextAction {
//     Continue,
//     Switch
// }

#[derive(EnumString, Display)]
pub enum ButtonCommand {
    AddChild,
    SealChildren,
}

// impl Default for Relation {
//     fn default() -> Self { Relation::Parent }
// }
// impl Default for NextAction {
//     fn default() -> Self { NextAction::Continue }
// }

// pub enum ButtonCommand {
//     IsChild(YesNo),
//     Relation(Relation),
//     NextAction(NextAction)
// }

// impl FromStr for ButtonCommand {
//     type Err = ();

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let matches: Vec<&str> = s.split(":").collect();
//         if matches.len() != 2 {
//             return Err(())
//         }
//         let command = match matches[0] {
//             "Command_Relation" => {
//                let relation = match Relation::from_str(matches[1]) {
//                     Ok(r) => r,
//                     Err(_) => return Err(())
//                 };
//                 Ok(ButtonCommand::Relation(relation))
//             },
//             "Command_NextAction"  => {
//                 let next_action = match NextAction::from_str(matches[1]) {
//                      Ok(r) => r,
//                      Err(_) => return Err(())
//                  };
//                  Ok(ButtonCommand::NextAction(next_action))
//              },
//              "Command_IsChild" => {
//                 let yes_no = match YesNo::from_str(matches[1]) {
//                      Ok(r) => r,
//                      Err(_) => return Err(())
//                  };
//                  Ok(ButtonCommand::IsChild(yes_no))
//              },
//             &_ => Err(())
//         };
//         command
//     }
// }

// impl ToString for ButtonCommand {
//     fn to_string(&self) -> String {
//         match &self {
//             ButtonCommand::IsChild(rel) => format!("Command_IsChild:{}", rel),
//             ButtonCommand::Relation(rel) => format!("Command_Relation:{}", rel),
//             ButtonCommand::NextAction(action) => format!("Command_NextAction:{}", action),
//         }
//     }
// }

pub enum OutputCommand {
     Prompt(String),
     PromptButtons(Vec<(ButtonCommand, String)>, String)
}

// pub fn map_to_output(command: UpdaterCommand) -> OutputCommand {
//     match command {
//         _ => OutputCommand::Prompt(format!("Sorry! I don't think I can handle this kind of input right now. Maybe you should press the button or choose another command?"))
//         Some(UpdaterCommand::CreateRelative(root_index, String::from(added_name))) => 
//     }
// }