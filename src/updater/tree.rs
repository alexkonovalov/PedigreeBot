use petgraph::Directed;
use petgraph::dot::Dot;
use petgraph::graph::Node;
use petgraph::visit::IntoNeighborsDirected;
use std::fmt::{self, Display};
use std::str::FromStr;
use std::string::{ToString};
use petgraph::{graph::{IndexType, NodeIndex}, Direction};
use petgraph::prelude::Graph;

use crate::updater::commands::{ButtonCommand, OutputCommand };

pub struct Person {
    name: String,
    completeness: NodeCompleteness,
    is_chilren_sealed: bool,
    is_siblings_sealed: bool,
}

impl Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Person {
    pub fn new(name: String, completeness: NodeCompleteness) -> Self { Self { name, completeness, is_chilren_sealed: false, is_siblings_sealed: false } }
}

enum ExpectedRelative {
    Parent,
    Child,
    Sibling
}

struct DescribedNodeInfo {
    ix: Option<NodeIndex<u32>>,
    is_transient: bool,
}

impl DescribedNodeInfo {
    fn new(ix: Option::<NodeIndex<u32>>) -> Self { Self { ix, is_transient: false } }
}

pub struct Updater {
   // expected_command: Option<UpdaterCommand>,
    graph: Graph<Person, &'static str, Directed, u32>,
    described_ix: DescribedNodeInfo,
// isTransient: bool,
    //expected_relative: Option<ExpectedRelative>
}


#[derive(PartialEq, Debug)]
pub enum NodeCompleteness {
    Plain,
    OneParent,
    ParentsComplete,
// SiblingsTransient,
    SiblingsComplete,
// ChildrenTrainsient,
    ChildrenComplete
}

const NEW_NODE_STATUS: NodeCompleteness = NodeCompleteness::Plain;

fn get_next_state(state: &NodeCompleteness) -> Option<NodeCompleteness> {
    match state {
        NodeCompleteness::Plain => {
            Some(NodeCompleteness::OneParent)
        }
        NodeCompleteness::OneParent => {
            Some(NodeCompleteness::ParentsComplete)
        }
        NodeCompleteness::ParentsComplete => {
            Some(NodeCompleteness::SiblingsComplete)
        }
        NodeCompleteness::SiblingsComplete => {
            Some(NodeCompleteness::ChildrenComplete)
        }
        NodeCompleteness::ChildrenComplete => {
            None //todo shouldnt be reachable
        }
    }
}

enum Action {
    AskFirstParent,
    AskSecondParent,
    AskIfSiblings,
    AskIfMoreSiblings,
    AskIfChildren,
    AskIfMoreChildren,
    Nothing
}

//todo fix optional params
fn prompt_next_action(action: &Action, name: &str) -> OutputCommand {
    return match action {
        Action::AskFirstParent => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::SealSiblings, "Don't know".to_string())
                ],
                format!("Write then name of the 1st parent of {}. If you don't know the name, press the button:", name)
            ),
        Action::AskSecondParent => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::SealSiblings, "Don't know".to_string())
                ],
                format!("Write then name of the 2nd parent of {}. If you don't know the name, press the button:", name)
            ),
        Action::AskIfSiblings => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::SealSiblings, "No / Don't know".to_string())
                ],
                format!("Tell me if there are any siblings of {}. If there are no siblings, or you have no idea then press the button.", name)
            ),
        Action::AskIfMoreSiblings => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::SealSiblings, "No. this is it".to_string())
                ],
                format!("Tell me the name of one more sibling of {} or press the button.", name)
            ),
        Action::AskIfChildren => 
            OutputCommand::Prompt(
                format!("Tell me if {} has any children", name)
            ),
        Action::AskIfMoreChildren => 
            OutputCommand::Prompt(
                format!("Prompt if more children of {}", name)
            ),
        _ => OutputCommand::Prompt("oops".to_string())
    }
}

#[derive(Debug)]
pub enum InputCommand<'a> {
    Text(&'a str),
    Yes,
    No
}

impl Updater {
    pub fn new() -> Self { Self { described_ix : DescribedNodeInfo::new(None), graph: Graph::new() } }

    pub fn print_dot(&self) -> String {
        Dot::new(&self.graph).to_string()
    }

    //graph update functions
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
    
                println!("switched to node {}, {:?}, {}", self.graph[*i].name, self.graph[*i].completeness, satisfied);
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
                match completeness {
                    NodeCompleteness::Plain => {
                        return prompt_next_action(&Action::AskFirstParent, name);
                    },
                    NodeCompleteness::OneParent => {
                        return prompt_next_action(&Action::AskSecondParent, name);
                    },
                    NodeCompleteness::ParentsComplete => {
                        return prompt_next_action(&Action::AskIfSiblings, name);
                    },
                    _ => ()
                }
                return OutputCommand::Prompt("oops. next node didnt match".to_string());
            },
            None => {
                return OutputCommand::Prompt("oops. no next node".to_string());
            }
        }
    }

    pub fn handle (&mut self, input_command: InputCommand) -> OutputCommand {
        let described_ix = &self.described_ix; //todo
        match (described_ix.ix, described_ix.is_transient, input_command) {
            (None, false, InputCommand::Text(name)) => {
                let root_index = self.graph.add_node(Person::new(name.to_string(), NEW_NODE_STATUS));
                self.described_ix = DescribedNodeInfo::new(Some(root_index));
                return prompt_next_action(&Action::AskFirstParent, name);
            }
            (Some(ix), is_transient, command) => {
                let current_status: &NodeCompleteness;
                let described_name: String;
                let described_ix_copy = ix.clone();
                {
                    let node = &self.graph[ix];
                    current_status = &node.completeness;
                    described_name = node.name.clone();
                }

                match (&current_status, command, is_transient) {
                    (NodeCompleteness::Plain, InputCommand::No, _) => {
                        self.graph[described_ix_copy].completeness = get_next_state(&NodeCompleteness::SiblingsComplete).unwrap();
                        return self.switch_next_relative();
                    },
                    (NodeCompleteness::OneParent, InputCommand::No, _) => {
                        self.graph[described_ix_copy].completeness = get_next_state(&NodeCompleteness::SiblingsComplete).unwrap();
                        return self.switch_next_relative();
                    },
                    (NodeCompleteness::Plain, InputCommand::Text(text), _) => {
                        self.add_parent(&described_ix_copy, text);
                        self.graph[described_ix_copy].completeness = get_next_state(&NodeCompleteness::Plain).unwrap();
                        return prompt_next_action(&Action::AskSecondParent, &described_name)
                    },
                    (NodeCompleteness::OneParent, InputCommand::Text(text), _) => {
                        self.add_parent(&described_ix_copy, text);
                        self.graph[described_ix_copy].completeness = get_next_state(&NodeCompleteness::OneParent).unwrap();
                        return prompt_next_action(&Action::AskIfSiblings, &described_name)
                    },
                    (NodeCompleteness::ParentsComplete, InputCommand::Text(text), _) => { //add first sibling 
                        self.described_ix.is_transient = true;
                        self.add_sibling(&described_ix_copy, text);
                        return prompt_next_action(&Action::AskIfMoreSiblings, &described_name)
                    },
                    (NodeCompleteness::ParentsComplete, InputCommand::No, _) => { //end sibnings. switch to next
                        self.described_ix.is_transient = false;
                        self.graph[described_ix_copy].completeness = get_next_state(&NodeCompleteness::ParentsComplete).unwrap();
                        return self.switch_next_relative();
                    },
                    (a,b, c)=> {
                         println!("finita!!!@! {:?}", a);
                         println!("finita!!!@! {:?}", b);
                         println!("finita!!!@! {:?}", c);
                       //  new_state = None;
                         return OutputCommand::Prompt("oops".to_string());
                     }
                }
            }
            _ => {
                return OutputCommand::Prompt("oops".to_string());
            }
        }
    }
}
