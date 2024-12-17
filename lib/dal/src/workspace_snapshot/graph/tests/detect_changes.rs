#[cfg(test)]
mod test {
    use crate::WorkspaceSnapshotGraphVCurrent;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn identical_graphs() -> Result<()> {
        let base_graph = WorkspaceSnapshotGraphVCurrent::new_for_unit_tests()?;
        let updated_graph = base_graph.clone();
        assert!(base_graph.is_acyclic_directed());
        assert!(updated_graph.is_acyclic_directed());

        let changes = base_graph.detect_changes(&updated_graph);
        assert!(changes.is_empty());

        Ok(())
    }
}
