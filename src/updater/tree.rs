use petgraph::Directed;
use petgraph::dot::Dot;
use std::fmt::{self, Display};
use std::string::{ToString};
use petgraph::{graph::{NodeIndex}, Direction};
use petgraph::prelude::Graph;

use crate::updater::commands::{ButtonCommand, OutputCommand};

pub struct Person {
    name: String,
    completeness: NodeCompleteness,
}

impl Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Person {
    pub fn new(name: String, completeness: NodeCompleteness) -> Self { Self { name, completeness } }
}

struct DescribedNodeInfo {
    ix: Option<NodeIndex<u32>>,
}

impl DescribedNodeInfo {
    fn new(ix: Option::<NodeIndex<u32>>) -> Self { Self { ix } }
}

pub struct Updater {
    graph: Graph<Person, &'static str, Directed, u32>,
    described_ix: DescribedNodeInfo,
}


#[derive(PartialEq, Debug)]
pub enum NodeCompleteness {
    Plain,
    OneParent,
    ParentsComplete,
    SiblingsComplete,
    ChildrenComplete
}

const NEW_NODE_STATUS: NodeCompleteness = NodeCompleteness::Plain;

enum Action {
    AskFirstParent,
    AskSecondParent,
    AskIfSiblings,
    AskIfMoreSiblings,
    AskIfChildren,
    AskIfMoreChildren
}

fn prompt_next_action(action: &Action, name: &str) -> OutputCommand {
    return match action {
        Action::AskFirstParent => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "Don't know".to_string())
                ],
                format!("Write then name of the 1st parent of {}. If you don't know the name, press the button.", name)
            ),
        Action::AskSecondParent => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "Don't know".to_string())
                ],
                format!("Write then name of the 2nd parent of {}. If you don't know the name, press the button.", name)
            ),
        Action::AskIfSiblings => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "No siblings".to_string())
                ],
                format!("Maybe {} has some siblings? Write the name of the first one that you know or press the button.", name)
            ),
        Action::AskIfMoreSiblings => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "No more siblings".to_string())
                ],
                format!("Tell me the name of one more sibling of {} or press the button.", name)
            ),
        Action::AskIfChildren => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "No children".to_string())
                ],
                format!("Tell me if {} has any children. If so, tell me the name. If none or you don't know, press the button.", name)
            ),
        Action::AskIfMoreChildren => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "No".to_string())
                ],
                format!("Maybe {} has any other kids? If there's none, press the button. If you know someone, write the name.", name)
            )
    }
}

#[derive(Debug)]
pub enum InputCommand<'a> {
    Text(&'a str),
    No
}

impl Updater {
    pub fn new() -> Self { Self { described_ix : DescribedNodeInfo::new(None), graph: Graph::new() } }

    pub fn print_dot(&self) -> String {
        Dot::new(&self.graph).to_string()
    }

    fn add_parent(&mut self, ix: &NodeIndex<u32>, name: &str) {
        let parent_ix = self.graph.add_node(Person::new(name.to_string(), NodeCompleteness::Plain));
        self.graph.add_edge(parent_ix, *ix, "");
    }

  
    fn add_sibling(&mut self, ix: &NodeIndex<u32>, name: &str) {
        let sibling_ix = self.graph.add_node(Person::new(name.to_string(), NodeCompleteness::SiblingsComplete));
        let mut parents = self.graph.neighbors_directed(*ix, Direction::Incoming).detach();
        while let Some(parent) = parents.next_node(&self.graph) {
            self.graph.add_edge(parent, sibling_ix, "");
        }
    }

    fn add_child(&mut self, ix: &NodeIndex<u32>, name: &str) -> NodeIndex<u32> {
        let child_ix = self.graph.add_node(Person::new(name.to_string(), NodeCompleteness::OneParent));
        self.graph.add_edge(*ix, child_ix, "");
        child_ix
    }

    fn has_children(&self, ix: &NodeIndex<u32>) -> bool {
         self.graph.neighbors_directed(*ix, Direction::Outgoing).count() > 0
    }

    fn get_description(&self, ix: &NodeIndex<u32>) -> Option<String> {
        let mut parent_names: Vec::<&str> = vec!();
        let mut child_names: Vec::<&str> = vec!();
        let mut parents = self.graph.neighbors_directed(*ix, Direction::Incoming).detach();
        let mut children = self.graph.neighbors_directed(*ix, Direction::Outgoing).detach();
        while let Some(i) = parents.next_node(&self.graph) {
            let parent = &self.graph[i];
            parent_names.push(&parent.name);
        }
        while let Some(i) = children.next_node(&self.graph) {
            let child = &self.graph[i];
            child_names.push(&child.name);
        }
        match (parent_names.len() > 0, child_names.len() > 0) {
            (true, true) => {
                Some(format!("who is parent of {} and also child of {}",child_names.join(", "), parent_names.join(", ")))
            },
            (false, true) => {
                Some(format!("who is parent of {}", child_names.join(", ")))
            },
            (true, false) => {
                Some(format!("who is child of {}", parent_names.join(", ")))
            }
            (false, false) => None
        }

   }

    fn get_next_node(&self) -> Option<NodeIndex<u32>> {
        let described_ix = self.graph.node_indices().find(|i| {
            let satisfied = (self.graph[*i].completeness == NodeCompleteness::Plain) |
            (self.graph[*i].completeness == NodeCompleteness::OneParent) |
            (self.graph[*i].completeness == NodeCompleteness::ParentsComplete);
            return satisfied;
        });
        if let Some(ix) = described_ix {
            Some(ix)
        } else {
            self.graph.node_indices().find(|i| {
                let satisfied = (self.graph[*i].completeness == NodeCompleteness::Plain) |
                (self.graph[*i].completeness == NodeCompleteness::SiblingsComplete);
                return satisfied;
            })
        }
    }

    pub fn switch_next_relative(&mut self) -> OutputCommand {
        match self.get_next_node() {
            Some(node_ix) => {
                self.described_ix = DescribedNodeInfo::new(Some(node_ix));
                let name = &self.graph[self.described_ix.ix.unwrap()].name;
                let completeness = &self.graph[self.described_ix.ix.unwrap()].completeness;
                
                let info: String;
                if let Some(description) = self.get_description(&node_ix) { 
                    info = format!("{}, {}", name, description);
                }
                else {
                    info = name.to_string();
                }
                
                match completeness {
                    NodeCompleteness::Plain => {
                        return prompt_next_action(&Action::AskFirstParent, &info);
                    },
                    NodeCompleteness::OneParent => {
                        return prompt_next_action(&Action::AskSecondParent, &info);
                    },
                    NodeCompleteness::ParentsComplete => {
                        return prompt_next_action(&Action::AskIfSiblings, &info);
                    },
                    NodeCompleteness::SiblingsComplete => {
                        if self.has_children(&node_ix) {
                            return prompt_next_action(&Action::AskIfMoreChildren, &info);
                        }
                        else {
                            return prompt_next_action(&Action::AskIfChildren, &info);
                        }
                    },
                    _ => ()
                }
                return OutputCommand::Prompt("Oops. Next node didn't match".to_string());
            },
            None => {
                return OutputCommand::Prompt("We asked enough! you can get your pedigree chart by performing /finish command".to_string());
            }
        }
    }

    pub fn handle_command (&mut self, input_command: InputCommand) -> OutputCommand {
        let described_ix = &self.described_ix; //todo rename
        match (described_ix.ix, input_command) {
            (None, InputCommand::Text(name)) => {
                let root_index = self.graph.add_node(Person::new(name.to_string(), NEW_NODE_STATUS));
                self.described_ix = DescribedNodeInfo::new(Some(root_index));
                return prompt_next_action(&Action::AskFirstParent, name);
            }
            (Some(ix), command) => {
                let current_status: &NodeCompleteness;
                let described_name: String;
                let described_ix_copy = ix.clone();
                {
                    let node = &self.graph[ix];
                    current_status = &node.completeness;
                    described_name = node.name.clone();
                }

                match (&current_status, command) {
                    (NodeCompleteness::Plain, InputCommand::No) => {
                        self.graph[described_ix_copy].completeness = NodeCompleteness::SiblingsComplete;
                        return self.switch_next_relative();
                    },
                    (NodeCompleteness::Plain, InputCommand::Text(text)) => {
                        self.add_parent(&described_ix_copy, text);
                        self.graph[described_ix_copy].completeness = NodeCompleteness::OneParent;
                        return prompt_next_action(&Action::AskSecondParent, &described_name)
                    },
                    (NodeCompleteness::OneParent, InputCommand::No) => {
                        self.graph[described_ix_copy].completeness = NodeCompleteness::ParentsComplete;
                        return self.switch_next_relative();
                    },
                    (NodeCompleteness::OneParent, InputCommand::Text(text)) => {
                        self.add_parent(&described_ix_copy, text);
                        self.graph[described_ix_copy].completeness = NodeCompleteness::ParentsComplete;
                        return prompt_next_action(&Action::AskIfSiblings, &described_name)
                    },
                    (NodeCompleteness::ParentsComplete, InputCommand::No) => { //end siblings. switch to next
                        self.graph[described_ix_copy].completeness = NodeCompleteness::SiblingsComplete;
                        return self.switch_next_relative();
                    },
                    (NodeCompleteness::ParentsComplete, InputCommand::Text(text),) => { //add sibling 
                        self.add_sibling(&described_ix_copy, text);
                        return prompt_next_action(&Action::AskIfMoreSiblings, &described_name)
                    },
                    (NodeCompleteness::SiblingsComplete, InputCommand::No) => { //end children. switch to next
                        self.graph[described_ix_copy].completeness = NodeCompleteness::ChildrenComplete;
                        return self.switch_next_relative();
                    },
                    (NodeCompleteness::SiblingsComplete, InputCommand::Text(text)) => { //add child 
                        let child_id = self.add_child(&described_ix_copy, text);
                        self.described_ix = DescribedNodeInfo::new(Some(child_id));
                        return prompt_next_action(&Action::AskSecondParent, &text)
                    },
                    (_,_)=> {
                         return OutputCommand::Prompt("Oops".to_string());
                     }
                }
            }
            _ => {
                return OutputCommand::Prompt("Oops".to_string());
            }
        }
    }
}
