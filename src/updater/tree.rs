use petgraph::Directed;
use petgraph::dot::Dot;
use strum_macros::EnumString;
use std::str::FromStr;
use std::string::{ToString};
use strum_macros::Display;
use petgraph::graph::{IndexType, NodeIndex};
use petgraph::prelude::Graph;

enum UpdaterCommand {
    CreateRelative(NodeIndex<u32>, String),
    CreateLink(NodeIndex<u32>, NodeIndex<u32>, String, String),
    ContinueOrSwitch(NodeIndex<u32>, String),
}

#[derive(EnumString, Display)]
pub enum Relation {
    Parent,
    Child
}

pub struct Updater {
    expected_command: Option<UpdaterCommand>,
    graph: Graph<String, Relation, Directed, u32>,
}


#[derive(EnumString, Display)]
pub enum NextAction {
    Continue,
    Switch
}

impl Default for Relation {
    fn default() -> Self { Relation::Parent }
}
impl Default for NextAction {
    fn default() -> Self { NextAction::Continue }
}

pub enum ButtonCommand {
    Relation(Relation),
    NextAction(NextAction)
}

impl FromStr for ButtonCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let matches: Vec<&str> = s.split(":").collect();
        if matches.len() != 2 {
            return Err(())
        }
        let command = match matches[0] {
            "Command_Relation" => {
               let relation = match Relation::from_str(matches[1]) {
                    Ok(r) => r,
                    Err(_) => return Err(())
                };
                Ok(ButtonCommand::Relation(relation))
            },
            "Command_NextAction"  => {
                let next_action = match NextAction::from_str(matches[1]) {
                     Ok(r) => r,
                     Err(_) => return Err(())
                 };
                 Ok(ButtonCommand::NextAction(next_action))
             },
            &_ => Err(())
        };
        command
    }
}

impl ToString for ButtonCommand {
    fn to_string(&self) -> String {
        match &self {
            ButtonCommand::Relation(rel) => format!("Command_Relation:{}", rel),
            ButtonCommand::NextAction(action) => format!("Command_NextAction:{}", action),
        }
    }
}


pub enum OutputCommand {
     Prompt(String),
     PromptButtons(Vec<(ButtonCommand, String)>, String)
}

impl Updater {
    pub fn new() -> Self { Self { expected_command : None, graph: Graph::new() } }

    pub fn print_dot(&self) -> String {
        Dot::new(&self.graph).to_string()
    }

    pub fn handle_text (&mut self, name: &str) -> OutputCommand {
        let added_name = name;
        let oup = match &self.expected_command {
            None => {
                let root_index = self.graph.add_node(String::from(added_name));
                //create root item
                (
                    Some(UpdaterCommand::CreateRelative(root_index, String::from(added_name))),
                    OutputCommand::Prompt(
                        format!("Congrats! {} added. Please write the name of his/her child or parent.", added_name)
                    )
                )
                
            },
            Some(UpdaterCommand::CreateRelative( ix, name  )) => {
                let relative_ix = self.graph.add_node(String::from(added_name));
                //create relative  
                (
                    Some(UpdaterCommand::CreateLink(*ix, relative_ix, String::from(added_name), String::from(name))),
                    OutputCommand::PromptButtons(
                        vec![
                            (ButtonCommand::Relation(Relation::Parent), "Parent".to_string()),
                            (ButtonCommand::Relation(Relation::Child), "Child".to_string())
                        ],
                    format!("Congrats {} added! Please tell me if {} is a child or parent of {}", added_name, added_name, name),
                    )
                )
            },
            _ => (None, OutputCommand::Prompt(
                    format!("Sorry! I don't think I can handle this kind of input right now. Maybe you should press the button or choose another command?")
            ))
        };
        self.expected_command = oup.0;
        oup.1
    }
    
    pub fn handle_buttons (&mut self, button_key: &str) -> OutputCommand {
        let oup = match ButtonCommand::from_str(button_key) {
            Ok(ButtonCommand::Relation(relation)) => {
                if let Some(UpdaterCommand::CreateLink(ix,ix2, name1, name2)) = &self.expected_command {
                    match relation  { 
                        Relation::Parent => self.graph.add_edge(*ix2, *ix, Relation::Parent ),
                        Relation::Child => self.graph.add_edge(*ix, *ix2, Relation::Parent ),
                    };

                    (
                        Some(UpdaterCommand::ContinueOrSwitch(*ix, String::from(name2))),
                        OutputCommand::PromptButtons(
                            vec![
                                (ButtonCommand::NextAction(NextAction::Continue), "Continue".to_string()),
                                (ButtonCommand::NextAction(NextAction::Switch), "Switch".to_string())
                            ],
                            format!("Congrats! Relationship between {} and {} added. Please choose do you want to continue describing {} or switch to next relative.", name1, name2, name2),
                        )
                    )
                }
                else {
                    (None, OutputCommand::Prompt(
                        format!("Sorry! I don't think I can handle this kind of input right now.")
                    ))
                }
            },
            Ok(ButtonCommand::NextAction(action)) => {
                if let Some(UpdaterCommand::ContinueOrSwitch(ix, name)) = &self.expected_command {
                    match action {
                        NextAction::Continue => {
                            (
                                Some(UpdaterCommand::CreateRelative(*ix, String::from(name))),
                                OutputCommand::Prompt(
                                    format!("Please write the name of next {}'s child or parent.", name)
                                )
                            )
                        },
                        NextAction::Switch => {
                            let mut next_ix: Option<NodeIndex<u32>> = None;

                            //todo. smelly
                            for i in self.graph.node_indices() {
                                if i.index() > ix.index() {
                                    next_ix = Some(i);
                                    break;
                                }
                            }
                            let described_index = next_ix.unwrap();
                            let described_name = &self.graph[described_index];

                            (
                                Some(UpdaterCommand::CreateRelative(described_index, String::from(described_name))),
                                OutputCommand::Prompt(
                                    format!("Please write the name of {} child or parent.", described_name)
                                )
                            )
                        }
                    }
                    
                }
                else {
                    (None,
                        OutputCommand::Prompt(
                        format!("Sorry! I don't think I can handle this kind of input right now.")
                    ))
                }
            },
            _ => {
                (None,
                OutputCommand::Prompt(
                    format!("Sorry! I don't think I can handle this kind of input right now.")
                ))
            }
        };

        self.expected_command = oup.0;
        return oup.1;
    }

} 