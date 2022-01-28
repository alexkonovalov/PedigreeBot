use super::model::{ Person };
use petgraph::{graph::{NodeIndex}, Direction};
use petgraph::{Graph, Directed};

pub fn get_node_description(graph: &Graph<Person, &str, Directed, u32>, ix: &NodeIndex<u32>) -> Option<String> {
    let mut parent_names: Vec::<&str> = vec!();
    let mut child_names: Vec::<&str> = vec!();
    let mut parents = graph.neighbors_directed(*ix, Direction::Incoming).detach();
    let mut children = graph.neighbors_directed(*ix, Direction::Outgoing).detach();
    while let Some(i) = parents.next_node(graph) {
        let parent = &graph[i];
        parent_names.push(&parent.name);
    }
    while let Some(i) = children.next_node(graph) {
        let child = &graph[i];
        child_names.push(&child.name);
    }
    match (!parent_names.is_empty(), !child_names.is_empty()) {
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