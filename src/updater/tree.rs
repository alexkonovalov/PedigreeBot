use petgraph::Directed;
use petgraph::dot::Dot;
use petgraph::visit::IntoNeighborsDirected;
use std::fmt::{self, Display};
use std::str::FromStr;
use std::string::{ToString};
use petgraph::{graph::{IndexType, NodeIndex}, Direction};
use petgraph::prelude::Graph;

use crate::updater::commands::{ButtonCommand, OutputCommand };

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

enum ExpectedRelative {
    Parent,
    Child
}

pub struct Updater {
   // expected_command: Option<UpdaterCommand>,
    graph: Graph<Person, &'static str, Directed, u32>,
    described_ix: Option<NodeIndex<u32>>,
    expected_relative: Option<ExpectedRelative>
}

impl Updater {
    pub fn new() -> Self { Self { described_ix : None, expected_relative: None, graph: Graph::new() } }

    pub fn print_dot(&self) -> String {
        Dot::new(&self.graph).to_string()
    }

    pub fn handle_text (&mut self, name: &str) -> OutputCommand {
        if let None = self.described_ix {
            //create root 
            let root_index = self.graph.add_node(Person::new(name.to_string()));
            self.described_ix = Some(root_index);
            return OutputCommand::PromptButtons(
                    vec![
                        (ButtonCommand::AddChild, "Yes".to_string()),
                        (ButtonCommand::SealChildren, "No".to_string())
                    ],
                format!("Congrats! {} added. Please tell me if he/she has any children", name)
            )
        }
        if let (Some(described_ix), Some(expected_relative)) = (self.described_ix, &self.expected_relative) {
            match expected_relative {
                ExpectedRelative::Child => {
                    let child_ix = self.graph.add_node(Person::new(name.to_string()));
                    self.graph.add_edge(described_ix, child_ix, "");
                    let described_name  = &self.graph[described_ix].name;
                    self.expected_relative = Some(ExpectedRelative::Child);

                    return OutputCommand::PromptButtons(
                            vec![
                                (ButtonCommand::AddChild, "Yes".to_string()),
                                (ButtonCommand::SealChildren, "No".to_string())
                            ],
                        format!("Congrats! {} added. Please tell me {} if he/she has any more children", name, described_name)
                    )
                }
                ExpectedRelative::Parent => {
                    let parent_ix = self.graph.add_node(Person::new(name.to_string()));
                    self.graph.add_edge(parent_ix, described_ix, "");
                    let described_name  = &self.graph[described_ix].name;
                    let parents_number = self.graph.neighbors_directed(described_ix, Direction::Incoming).count();

                    if parents_number < 2 {
                        self.expected_relative = Some(ExpectedRelative::Parent);
                        return OutputCommand::Prompt(
                            format!("Congrats! {} added. Please tell me the 2nd {}'s parent's name", name, described_name)
                        )
                    }
                    else {
                        self.expected_relative = None;
                        self.described_ix = self.graph.node_indices().find(|i| {
                            let person = &self.graph[*i];
                            //let parents_number = self.graph.neighbors_directed(i, Direction::Incoming).count();
                            !person.is_chilren_sealed
                        });
                        let new_described_name = &self.graph[self.described_ix.unwrap()].name;
                        return OutputCommand::PromptButtons(
                            vec![
                                (ButtonCommand::AddChild, "Yes".to_string()),
                                (ButtonCommand::SealChildren, "No".to_string())
                            ],
                        format!("Congrats! We finished withh {}. Let's begin describing {} Please tell me if he/she has any/any more children", described_name, new_described_name)
                        )
                    }
                }
            }

        }
        OutputCommand::Prompt("oops".to_string())
    }

    pub fn handle_buttons (&mut self, button: &str) -> OutputCommand {
        if let (Some(ix), Ok(button_command)) = (self.described_ix, ButtonCommand::from_str(button)) {
            match button_command {
                ButtonCommand::SealChildren => {
                    self.graph[ix].is_chilren_sealed = true;
                    self.expected_relative = Some(ExpectedRelative::Parent);
                    let name = &self.graph[ix].name;
                    return OutputCommand::Prompt(format!("Okay! Give me the name of {}'s 1st parent", name));
                }
                ButtonCommand::AddChild => {
                    let name = &self.graph[ix].name;
                    self.expected_relative = Some(ExpectedRelative::Child);
                    return OutputCommand::Prompt(format!("Give me some of {}'s kids name", name));
                }
            }

        }
        // if let None = self.described_ix {
        //     //create root 
        //     let root_index = self.graph.add_node(Person::new(name.to_string()));
        // }
        OutputCommand::Prompt("oops".to_string())
    }

    // pub fn handle_text (&mut self, name: &str) -> OutputCommand {
    //     let added_name = name;
    //     let oup = match &self.expected_command {
    //         None => {
    //             let root_index = self.graph.add_node(Person::new(added_name.to_string()));
    //             //create root item
    //             (
    //                 Some(UpdaterCommand::CheckIfChildren(root_index, String::from(added_name))),
    //                 OutputCommand::PromptButtons(
    //                         vec![
    //                             (ButtonCommand::IsChild(YesNo::Yes), "Yes".to_string()),
    //                             (ButtonCommand::IsChild(YesNo::No), "No".to_string())
    //                         ],
    //                     format!("Congrats! {} added. Please tell me about he/she has any children", added_name)
    //                 )
    //             )
                
    //         },
    //         Some(UpdaterCommand::CreateRelative( ix, name  )) => {
    //             let relative_ix = self.graph.add_node(Person::new(added_name.to_string()));
    //             //create relative  
    //             (
    //                 Some(UpdaterCommand::CreateLink(*ix, relative_ix, String::from(added_name), String::from(name))),
    //                 OutputCommand::PromptButtons(
    //                     vec![
    //                         (ButtonCommand::Relation(Relation::Parent), "Parent".to_string()),
    //                         (ButtonCommand::Relation(Relation::Child), "Child".to_string())
    //                     ],
    //                 format!("Congrats {} added! Please tell me if {} is a child or parent of {}", added_name, added_name, name),
    //                 )
    //             )
    //         },
    //         _ => (None, OutputCommand::Prompt(
    //                 format!("Sorry! I don't think I can handle this kind of input right now. Maybe you should press the button or choose another command?")
    //         ))
    //     };
    //     self.expected_command = oup.0;
    //     oup.1
    // }
    
    // pub fn handle_buttons (&mut self, button_key: &str) -> OutputCommand {
    //     let oup = match ButtonCommand::from_str(button_key) {
    //         Ok(ButtonCommand::IsChild(yesNo)) => {
    //             if let Some(UpdaterCommand::CreateLink(ix,ix2, name1, name2)) = &self.expected_command {
    //                 match yesNo { 
    //                     YesNo::Yes => {

    //                     }, // self.graph.add_edge(*ix2, *ix, Relation::Parent ),
    //                     YesNo::No => {

    //                     }//Relation::Child => self.graph.add_edge(*ix, *ix2, Relation::Parent ),
    //                 };

    //                 (
    //                     Some(UpdaterCommand::ContinueOrSwitch(*ix, String::from(name2))),
    //                     OutputCommand::PromptButtons(
    //                         vec![
    //                             (ButtonCommand::NextAction(NextAction::Continue), "Continue".to_string()),
    //                             (ButtonCommand::NextAction(NextAction::Switch), "Switch".to_string())
    //                         ],
    //                         format!("Congrats! Relationship between {} and {} added. Please choose do you want to continue describing {} or switch to next relative.", name1, name2, name2),
    //                     )
    //                 )
    //             }
    //             else {
    //                 (None, OutputCommand::Prompt(
    //                     format!("Sorry! I don't think I can handle this kind of input right now.")
    //                 ))
    //             }
    //         },
    //         Ok(ButtonCommand::Relation(relation)) => {
    //             if let Some(UpdaterCommand::CreateLink(ix,ix2, name1, name2)) = &self.expected_command {
    //                 match relation  { 
    //                     Relation::Parent => self.graph.add_edge(*ix2, *ix, Relation::Parent ),
    //                     Relation::Child => self.graph.add_edge(*ix, *ix2, Relation::Parent ),
    //                 };

    //                 (
    //                     Some(UpdaterCommand::ContinueOrSwitch(*ix, String::from(name2))),
    //                     OutputCommand::PromptButtons(
    //                         vec![
    //                             (ButtonCommand::NextAction(NextAction::Continue), "Continue".to_string()),
    //                             (ButtonCommand::NextAction(NextAction::Switch), "Switch".to_string())
    //                         ],
    //                         format!("Congrats! Relationship between {} and {} added. Please choose do you want to continue describing {} or switch to next relative.", name1, name2, name2),
    //                     )
    //                 )
    //             }
    //             else {
    //                 (None, OutputCommand::Prompt(
    //                     format!("Sorry! I don't think I can handle this kind of input right now.")
    //                 ))
    //             }
    //         },
    //         Ok(ButtonCommand::NextAction(action)) => {
    //             if let Some(UpdaterCommand::ContinueOrSwitch(ix, name)) = &self.expected_command {
    //                 match action {
    //                     NextAction::Continue => {
    //                         (
    //                             Some(UpdaterCommand::CreateRelative(*ix, String::from(name))),
    //                             OutputCommand::Prompt(
    //                                 format!("Please write the name of next {}'s child or parent.", name)
    //                             )
    //                         )
    //                     },
    //                     NextAction::Switch => {
    //                         let mut next_ix: Option<NodeIndex<u32>> = None;

    //                         //todo. smelly
    //                         for i in self.graph.node_indices() {
    //                             if i.index() > ix.index() {
    //                                 next_ix = Some(i);
    //                                 break;
    //                             }
    //                         }
    //                         let described_index = next_ix.unwrap();
    //                         let described_person = &self.graph[described_index];

    //                         (
    //                             Some(UpdaterCommand::CreateRelative(described_index, String::from(&described_person.name))),
    //                             OutputCommand::Prompt(
    //                                 format!("Please write the name of {} child or parent.", described_person)
    //                             )
    //                         )
    //                     }
    //                 }
                    
    //             }
    //             else {
    //                 (None,
    //                     OutputCommand::Prompt(
    //                     format!("Sorry! I don't think I can handle this kind of input right now.")
    //                 ))
    //             }
    //         },
    //         _ => {
    //             (None,
    //             OutputCommand::Prompt(
    //                 format!("Sorry! I don't think I can handle this kind of input right now.")
    //             ))
    //         }
    //     };

    //     self.expected_command = oup.0;
    //     return oup.1;
    // }

} 