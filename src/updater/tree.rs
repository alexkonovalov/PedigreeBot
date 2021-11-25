use petgraph::Directed;
use petgraph::dot::Dot;
use std::fmt::{self, Display};
use std::str::FromStr;
use std::string::{ToString};
use petgraph::graph::{IndexType, NodeIndex};
use petgraph::prelude::Graph;

use crate::updater::commands::{ButtonCommand, OutputCommand, Relation, UpdaterCommand, NextAction, YesNo};

pub struct Person {
    name: String,
    is_chilren_sealed: bool,
}

impl Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}


impl Person {
    pub fn new(name: String) -> Self { Self { name, is_chilren_sealed: false } }
}

pub struct Updater {
    expected_command: Option<UpdaterCommand>,
    graph: Graph<Person, Relation, Directed, u32>,
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
                let root_index = self.graph.add_node(Person::new(added_name.to_string()));
                //create root item
                (
                    Some(UpdaterCommand::CheckIfChildren(root_index, String::from(added_name))),
                    OutputCommand::PromptButtons(
                            vec![
                                (ButtonCommand::IsChild(YesNo::Yes), "Yes".to_string()),
                                (ButtonCommand::IsChild(YesNo::No), "No".to_string())
                            ],
                        format!("Congrats! {} added. Please tell me about he/she has any children", added_name)
                    )
                )
                
            },
            Some(UpdaterCommand::CreateRelative( ix, name  )) => {
                let relative_ix = self.graph.add_node(Person::new(added_name.to_string()));
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
            Ok(ButtonCommand::IsChild(yesNo)) => {
                if let Some(UpdaterCommand::CreateLink(ix,ix2, name1, name2)) = &self.expected_command {
                    match yesNo { 
                        YesNo::Yes => {

                        }, // self.graph.add_edge(*ix2, *ix, Relation::Parent ),
                        YesNo::No => {

                        }//Relation::Child => self.graph.add_edge(*ix, *ix2, Relation::Parent ),
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
                            let described_person = &self.graph[described_index];

                            (
                                Some(UpdaterCommand::CreateRelative(described_index, String::from(&described_person.name))),
                                OutputCommand::Prompt(
                                    format!("Please write the name of {} child or parent.", described_person)
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