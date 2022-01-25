use super::model::{Action, OutputCommand, ButtonCommand, Person};
use petgraph::{graph::{NodeIndex}, Direction};
use petgraph::{Graph, Directed};

pub fn map_next_action_output(action: &Action, name: &str) -> OutputCommand {
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