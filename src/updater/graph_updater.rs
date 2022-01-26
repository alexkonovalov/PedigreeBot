use petgraph::Directed;
use petgraph::dot::Dot;
use std::string::ToString;
use petgraph::{graph::{NodeIndex}, Direction};
use petgraph::prelude::Graph;

use crate::updater::model::{ OutputCommand};

use super::{model::{Person, DescribedNodeInfo, NodeCompleteness, Action, InputCommand, NEW_NODE_STATUS}, utility::get_node_description};
use super::utility::map_next_action_output;

pub struct GraphUpdater {
    graph: Graph<Person, &'static str, Directed, u32>,
    described_ix: DescribedNodeInfo,
}

impl GraphUpdater {
    pub fn new() -> Self { Self { described_ix : DescribedNodeInfo::new(None), graph: Graph::new() } }

    //todo move out. dot should be outside updater
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
        get_node_description(&self.graph, ix)
   }

    fn get_next_node(&self) -> Option<NodeIndex<u32>> {
        let described_ix = self.graph.node_indices().find(|i| {
            
            (self.graph[*i].completeness == NodeCompleteness::Plain) |
            (self.graph[*i].completeness == NodeCompleteness::OneParent) |
            (self.graph[*i].completeness == NodeCompleteness::ParentsComplete)
        });
        if let Some(ix) = described_ix {
            Some(ix)
        } else {
            self.graph.node_indices().find(|i| {
                
                (self.graph[*i].completeness == NodeCompleteness::Plain) |
                (self.graph[*i].completeness == NodeCompleteness::SiblingsComplete)
            })
        }
    }

    fn switch_next_relative(&mut self) -> OutputCommand {
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
                        return map_next_action_output(&Action::AskFirstParent, &info);
                    },
                    NodeCompleteness::OneParent => {
                        return map_next_action_output(&Action::AskSecondParent, &info);
                    },
                    NodeCompleteness::ParentsComplete => {
                        return map_next_action_output(&Action::AskIfSiblings, &info);
                    },
                    NodeCompleteness::SiblingsComplete => {
                        if self.has_children(&node_ix) {
                            return map_next_action_output(&Action::AskIfMoreChildren, &info);
                        }
                        else {
                            return map_next_action_output(&Action::AskIfChildren, &info);
                        }
                    },
                    _ => ()
                }
                OutputCommand::Prompt("Oops. Next node didn't match".to_string())
            },
            None => {
                OutputCommand::Prompt("We asked enough! you can get your pedigree chart by performing /finish command".to_string())
            }
        }
    }

    pub fn handle_command (&mut self, input_command: InputCommand) -> OutputCommand {
        let described_ix = &self.described_ix; //todo rename
        match (described_ix.ix, input_command) {
            (None, InputCommand::Text(name)) => {
                let root_index = self.graph.add_node(Person::new(name.to_string(), NEW_NODE_STATUS));
                self.described_ix = DescribedNodeInfo::new(Some(root_index));
                map_next_action_output(&Action::AskFirstParent, name)
            }
            (Some(ix), command) => {
                let current_status: &NodeCompleteness;
                let described_name: String;
                let described_ix_copy = ix;
                {
                    let node = &self.graph[ix];
                    current_status = &node.completeness;
                    described_name = node.name.clone();
                }

                match (&current_status, command) {
                    (NodeCompleteness::Plain, InputCommand::No) => {
                        self.graph[described_ix_copy].completeness = NodeCompleteness::SiblingsComplete;
                        self.switch_next_relative()
                    },
                    (NodeCompleteness::Plain, InputCommand::Text(text)) => {
                        self.add_parent(&described_ix_copy, text);
                        self.graph[described_ix_copy].completeness = NodeCompleteness::OneParent;
                        map_next_action_output(&Action::AskSecondParent, &described_name)
                    },
                    (NodeCompleteness::OneParent, InputCommand::No) => {
                        self.graph[described_ix_copy].completeness = NodeCompleteness::ParentsComplete;
                        self.switch_next_relative()
                    },
                    (NodeCompleteness::OneParent, InputCommand::Text(text)) => {
                        self.add_parent(&described_ix_copy, text);
                        self.graph[described_ix_copy].completeness = NodeCompleteness::ParentsComplete;
                        map_next_action_output(&Action::AskIfSiblings, &described_name)
                    },
                    (NodeCompleteness::ParentsComplete, InputCommand::No) => { //end siblings. switch to next
                        self.graph[described_ix_copy].completeness = NodeCompleteness::SiblingsComplete;
                        self.switch_next_relative()
                    },
                    (NodeCompleteness::ParentsComplete, InputCommand::Text(text),) => { //add sibling 
                        self.add_sibling(&described_ix_copy, text);
                        map_next_action_output(&Action::AskIfMoreSiblings, &described_name)
                    },
                    (NodeCompleteness::SiblingsComplete, InputCommand::No) => { //end children. switch to next
                        self.graph[described_ix_copy].completeness = NodeCompleteness::ChildrenComplete;
                        self.switch_next_relative()
                    },
                    (NodeCompleteness::SiblingsComplete, InputCommand::Text(text)) => { //add child 
                        let child_id = self.add_child(&described_ix_copy, text);
                        self.described_ix = DescribedNodeInfo::new(Some(child_id));
                        map_next_action_output(&Action::AskSecondParent, text)
                    },
                    (_,_)=> {
                         OutputCommand::Prompt("Oops".to_string())
                     }
                }
            }
            _ => {
                OutputCommand::Prompt("Oops".to_string())
            }
        }
    }
}
