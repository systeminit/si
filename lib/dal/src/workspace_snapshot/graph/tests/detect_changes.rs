#[cfg(test)]
mod test {
    use crate::WorkspaceSnapshotGraphVCurrent;

    #[test]
    fn identical_graphs() {
        let base_graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()
            .expect("could not create a new graph for unit tests");
        let updated_graph = base_graph.clone();
        assert!(base_graph.is_acyclic_directed());
        assert!(updated_graph.is_acyclic_directed());

        let changes = base_graph
            .detect_changes(&updated_graph)
            .expect("could not detect changes");
        assert!(changes.is_empty());
    }
}
