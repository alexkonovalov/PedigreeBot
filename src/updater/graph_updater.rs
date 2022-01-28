use petgraph::{Directed};
use petgraph::dot::Dot;
use std::string::ToString;
use petgraph::{graph::{NodeIndex}, Direction};
use petgraph::prelude::Graph;
use super::{model::{Person, DescribedNodeInfo, NodeCompleteness, OutputAction, InputAction, NEW_NODE_STATUS}, utility::get_node_description};

pub struct GraphUpdater {
    graph: Graph<Person, &'static str, Directed, u32>,
    described_ix: DescribedNodeInfo,
}

impl GraphUpdater {
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
        get_node_description(&self.graph, ix)
    }

    fn get_next_node(&self) -> Option<NodeIndex<u32>> {
        let described_ix = self.graph.node_indices().find(|i| {
            [NodeCompleteness::Plain, NodeCompleteness::OneParent, NodeCompleteness::ParentsComplete].contains(&self.graph[*i].completeness)
        });
        if let Some(ix) = described_ix {
            Some(ix)
        } else {
            self.graph.node_indices().find(|i| {
                self.graph[*i].completeness == NodeCompleteness::SiblingsComplete
            })
        }
    }

    fn switch_next_relative(&mut self) -> OutputAction {
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
                        OutputAction::AskFirstParent(info)
                    },
                    NodeCompleteness::OneParent => {
                        OutputAction::AskSecondParent(info)
                    },
                    NodeCompleteness::ParentsComplete => {
                        OutputAction::AskIfSiblings(info)
                    },
                    NodeCompleteness::SiblingsComplete => {
                        if self.has_children(&node_ix) {
                            OutputAction::AskIfMoreChildren(info)
                        }
                        else {
                            OutputAction::AskIfChildren(info)
                        }
                    },
                    NodeCompleteness::ChildrenComplete => {
                        OutputAction::NotifyError
                    }
                }
            },
            None => {
                OutputAction::NotifyComplete
            }
        }
    }

    pub fn handle_command (&mut self, input_command: InputAction) -> OutputAction {
        let described_ix = &self.described_ix; //todo rename
        match (described_ix.ix, input_command) {
            (None, InputAction::Text(name)) => {
                let root_index = self.graph.add_node(Person::new(name.to_string(), NEW_NODE_STATUS));
                self.described_ix = DescribedNodeInfo::new(Some(root_index));
                OutputAction::AskFirstParent(name.to_string())
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
                    (NodeCompleteness::Plain, InputAction::No) => {
                        self.graph[described_ix_copy].completeness = NodeCompleteness::SiblingsComplete;
                        self.switch_next_relative()
                    },
                    (NodeCompleteness::Plain, InputAction::Text(text)) => {
                        self.add_parent(&described_ix_copy, text);
                        self.graph[described_ix_copy].completeness = NodeCompleteness::OneParent;
                        OutputAction::AskSecondParent(described_name)
                    },
                    (NodeCompleteness::OneParent, InputAction::No) => {
                        self.graph[described_ix_copy].completeness = NodeCompleteness::ParentsComplete;
                        self.switch_next_relative()
                    },
                    (NodeCompleteness::OneParent, InputAction::Text(text)) => {
                        self.add_parent(&described_ix_copy, text);
                        self.graph[described_ix_copy].completeness = NodeCompleteness::ParentsComplete;
                        OutputAction::AskIfSiblings(described_name)
                    },
                    (NodeCompleteness::ParentsComplete, InputAction::No) => { //end siblings. switch to next
                        self.graph[described_ix_copy].completeness = NodeCompleteness::SiblingsComplete;
                        self.switch_next_relative()
                    },
                    (NodeCompleteness::ParentsComplete, InputAction::Text(text),) => { //add sibling 
                        self.add_sibling(&described_ix_copy, text);
                        OutputAction::AskIfMoreSiblings(described_name)
                    },
                    (NodeCompleteness::SiblingsComplete, InputAction::No) => { //end children. switch to next
                        self.graph[described_ix_copy].completeness = NodeCompleteness::ChildrenComplete;
                        self.switch_next_relative()
                    },
                    (NodeCompleteness::SiblingsComplete, InputAction::Text(text)) => { //add child 
                        let child_id = self.add_child(&described_ix_copy, text);
                        self.described_ix = DescribedNodeInfo::new(Some(child_id)); //switch describe child
                        OutputAction::AskSecondParent(text.to_string())
                    },
                    (NodeCompleteness::ChildrenComplete, _) => {
                        OutputAction::NotifyError
                    }
                }
            }
            _ => {
                OutputAction::NotifyError
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const ROOT_NODE : &str = "Robert";
    const MOM_NODE : &str = "Alexandra";
    const DAD_NODE : &str = "Bernard";
    const BRO_NODE : &str = "Bruce";
    const CHILD_NODE : &str = "Anna";
    const SPOUSE_NODE : &str = "Marie";

    #[test]
    fn empty() {
        let updater = GraphUpdater::new();
        assert_eq!(updater.print_dot(), 
"digraph {
}
");
    }

    #[test]
    fn one_node_added_complete() {
        let mut updater = GraphUpdater::new();
        let output_action = updater.handle_command(InputAction::Text(ROOT_NODE));
        let output_action_1 = updater.handle_command(InputAction::No);
        let output_action_2 = updater.handle_command(InputAction::No);
        let output_action_3 = updater.handle_command(InputAction::Text(""));
        assert_eq!(output_action, OutputAction::AskFirstParent(ROOT_NODE.to_string()), "Should ask for 1st parent");
        assert_eq!(output_action_1, OutputAction::AskIfChildren(ROOT_NODE.to_string()), "Should ask for kids");
        assert_eq!(output_action_2, OutputAction::NotifyComplete, "Should finilize graph");
        assert_eq!(output_action_3, OutputAction::NotifyError, "Should notify that graph is already finished");
        assert_eq!(updater.print_dot(),
format!("digraph {{
    0 [ label = \"{}\" ]
}}
", ROOT_NODE), "Should print graph with root node");
    }

    #[test]
    fn family_with_two_children() {
        let mut updater = GraphUpdater::new();
        let output_action_1 = updater.handle_command(InputAction::Text(ROOT_NODE));
        let output_action_2 = updater.handle_command(InputAction::Text(MOM_NODE));
        let output_action_3 = updater.handle_command(InputAction::Text(DAD_NODE));
        let output_action_4 = updater.handle_command(InputAction::Text(BRO_NODE));

        assert_eq!(output_action_1, OutputAction::AskFirstParent(ROOT_NODE.to_string()), "Should ask for 1st parent");
        assert_eq!(output_action_2, OutputAction::AskSecondParent(ROOT_NODE.to_string()), "Should ask for 2nd parent");
        assert_eq!(output_action_3, OutputAction::AskIfSiblings(ROOT_NODE.to_string()), "Should ask for sibling");
        assert_eq!(output_action_4, OutputAction::AskIfMoreSiblings(ROOT_NODE.to_string()), "Should ask for more siblings");
        assert_eq!(updater.print_dot(),
format!("digraph {{
    0 [ label = \"{}\" ]
    1 [ label = \"{}\" ]
    2 [ label = \"{}\" ]
    3 [ label = \"{}\" ]
    1 -> 0 [ label = \"\" ]
    2 -> 0 [ label = \"\" ]
    2 -> 3 [ label = \"\" ]
    1 -> 3 [ label = \"\" ]
}}
", ROOT_NODE, MOM_NODE, DAD_NODE, BRO_NODE), "Should print graph with 2 kids and 2 parents");
    }

    #[test]
    fn orphan_root_with_child_and_spouse() {
        let mut updater = GraphUpdater::new();
        let output_action_1 = updater.handle_command(InputAction::Text(ROOT_NODE));
        let output_action_2 = updater.handle_command(InputAction::No);
        let output_action_3 = updater.handle_command(InputAction::Text(CHILD_NODE));
        let output_action_4 = updater.handle_command(InputAction::Text(SPOUSE_NODE));
        let output_action_5 = updater.handle_command(InputAction::No);

        assert_eq!(output_action_1, OutputAction::AskFirstParent(ROOT_NODE.to_string()), "Should ask for parent");
        assert_eq!(output_action_2, OutputAction::AskIfChildren(ROOT_NODE.to_string()), "Should jump straight to children");
        assert_eq!(output_action_3, OutputAction::AskSecondParent(CHILD_NODE.to_string()), "Should switch to kid's second parent");
        assert_eq!(output_action_4, OutputAction::AskIfSiblings(CHILD_NODE.to_string()), "Should check if kids has siblings");
        assert_eq!(output_action_5, OutputAction::AskFirstParent(format!("{}, who is parent of {}", SPOUSE_NODE, CHILD_NODE)), "Should start asking about spouse");
        assert_eq!(updater.print_dot(),
format!("digraph {{
    0 [ label = \"{}\" ]
    1 [ label = \"{}\" ]
    2 [ label = \"{}\" ]
    0 -> 1 [ label = \"\" ]
    2 -> 1 [ label = \"\" ]
}}
", ROOT_NODE, CHILD_NODE, SPOUSE_NODE), "Should print graph with root node and 1 child");
    }

}