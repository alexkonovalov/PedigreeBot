#[cfg(test)]
mod tests {
    use crate::updater::{graph_updater::GraphUpdater, model::{InputCommand, OutputAction}};

    #[test]
    fn empty_updater_prints_empty() {
        let updater = GraphUpdater::new();
        assert_eq!(updater.print_dot(), 
"digraph {
}
");
    }

    #[test]
    fn updater_with_one_entry_prints_graph_with_one_node() {
        let mut updater = GraphUpdater::new();
        let output_action = updater.handle_command(InputCommand::Text("Robert"));
        assert_eq!(output_action, OutputAction::AskFirstParent("Robert".to_string()));
        assert_eq!(updater.print_dot(),
"digraph {
    0 [ label = \"Robert\" ]
}
");
    }

}