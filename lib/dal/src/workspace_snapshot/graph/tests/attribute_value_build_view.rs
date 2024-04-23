#[allow(clippy::panic)]
#[cfg(test)]
mod test {
    #[tokio::test]
    #[cfg(ignore)]
    async fn attribute_value_build_view() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = content_store::LocalStore::default();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_content_hash = content_store
            .add(&serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_content_hash = content_store
            .add(&serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_content_hash = content_store
            .add(&serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let si_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let si_prop_content_hash = content_store
            .add(&serde_json::json!("SI Prop Content"))
            .expect("Unable to add to content store");
        let si_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    si_prop_id,
                    PropKind::Object,
                    "si",
                    si_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                si_prop_node_index,
            )
            .expect("Unable to add root prop -> si prop edge");

        let name_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let name_prop_content_hash = content_store
            .add(&serde_json::json!("Name Prop Content"))
            .expect("Unable to add to content store");
        let name_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    name_prop_id,
                    PropKind::Object,
                    "name",
                    name_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                name_prop_node_index,
            )
            .expect("Unable to add si prop -> name prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_content_hash = content_store
            .add(&serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let si_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let si_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let si_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    si_av_id,
                    ContentAddress::AttributeValue(si_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                si_av_node_index,
            )
            .expect("Unable to add root av -> si av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add si av -> si prop edge");

        let name_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let name_av_content_hash = content_store
            .add(&serde_json::json!("component name"))
            .expect("Unable to add to content store");
        let name_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    name_av_id,
                    ContentAddress::AttributeValue(name_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeWeight"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                name_av_node_index,
            )
            .expect("Unable to add si av -> name av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(name_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(name_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create name av -> name prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{"si": {"name": "component name"}}],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );
    }

    #[tokio::test]
    #[cfg(ignore)]
    async fn attribute_value_build_view_unordered_object() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = content_store::LocalStore::default();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_content_hash = content_store
            .add(&serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_content_hash = content_store
            .add(&serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_content_hash = content_store
            .add(&serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let si_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let si_prop_content_hash = content_store
            .add(&serde_json::json!("SI Prop Content"))
            .expect("Unable to add to content store");
        let si_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    si_prop_id,
                    PropKind::Object,
                    "si",
                    si_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                si_prop_node_index,
            )
            .expect("Unable to add root prop -> si prop edge");

        let name_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let name_prop_content_hash = content_store
            .add(&serde_json::json!("Name Prop Content"))
            .expect("Unable to add to content store");
        let name_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    name_prop_id,
                    PropKind::Object,
                    "name",
                    name_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                name_prop_node_index,
            )
            .expect("Unable to add si prop -> name prop edge");

        let description_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let description_prop_content_hash = content_store
            .add(&serde_json::json!("Description Prop Content"))
            .expect("Unable to add to content store");
        let description_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    description_prop_id,
                    PropKind::String,
                    "description",
                    description_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add description prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                description_prop_node_index,
            )
            .expect("Unable to add si prop -> description prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_content_hash = content_store
            .add(&serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let si_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let si_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let si_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    si_av_id,
                    ContentAddress::AttributeValue(si_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add si av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                si_av_node_index,
            )
            .expect("Unable to add root av -> si av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(si_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add si av -> si prop edge");

        let name_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let name_av_content_hash = content_store
            .add(&serde_json::json!("component name"))
            .expect("Unable to add to content store");
        let name_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    name_av_id,
                    ContentAddress::AttributeValue(name_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add name av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                name_av_node_index,
            )
            .expect("Unable to add si av -> name av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(name_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(name_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create name av -> name prop edge");

        let description_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let description_av_content_hash = content_store
            .add(&serde_json::json!("Component description"))
            .expect("Unable to add to content store");
        let description_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    description_av_id,
                    ContentAddress::AttributeValue(description_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add description av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(si_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                description_av_node_index,
            )
            .expect("Unable to add si av -> description av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(description_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(description_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add description av -> description prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{
                "si": {
                    "description": "Component description",
                    "name": "component name",
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );
    }

    #[tokio::test]
    #[cfg(ignore)]
    async fn attribute_value_build_view_ordered_array() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = content_store::LocalStore::default();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_content_hash = content_store
            .add(&serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_content_hash = content_store
            .add(&serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_content_hash = content_store
            .add(&serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let domain_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let domain_prop_content_hash = content_store
            .add(&serde_json::json!("domain Prop Content"))
            .expect("Unable to add to content store");
        let domain_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    domain_prop_id,
                    PropKind::Object,
                    "domain",
                    domain_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                domain_prop_node_index,
            )
            .expect("Unable to add root prop -> domain prop edge");

        let ports_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ports_prop_content_hash = content_store
            .add(&serde_json::json!("ports Prop Content"))
            .expect("Unable to add to content store");
        let ports_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    ports_prop_id,
                    PropKind::Array,
                    "ports",
                    ports_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ports prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                ports_prop_node_index,
            )
            .expect("Unable to add domain prop -> ports prop edge");

        let port_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port_prop_content_hash = content_store
            .add(&serde_json::json!("port Prop Content"))
            .expect("Unable to add to content store");
        let port_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    port_prop_id,
                    PropKind::String,
                    "port",
                    port_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(ports_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                port_prop_node_index,
            )
            .expect("Unable to add ports prop -> port prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_content_hash = content_store
            .add(&serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    root_av_id,
                    ContentAddress::AttributeValue(root_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let domain_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let domain_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let domain_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    domain_av_id,
                    ContentAddress::AttributeValue(domain_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                domain_av_node_index,
            )
            .expect("Unable to add root av -> domain av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add domain av -> domain prop edge");

        let ports_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ports_av_content_hash = content_store
            .add(&serde_json::json!([]))
            .expect("Unable to add to content store");
        let ports_av_node_index = graph
            .add_ordered_node(
                change_set,
                NodeWeight::new_content(
                    change_set,
                    ports_av_id,
                    ContentAddress::AttributeValue(ports_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add ports av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                ports_av_node_index,
            )
            .expect("Unable to add domain av -> ports av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(ports_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create ports av -> ports prop edge");

        let port1_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port1_av_content_hash = content_store
            .add(&serde_json::json!("Port 1"))
            .expect("Unable to add to content store");
        let port1_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port1_av_id,
                    ContentAddress::AttributeValue(port1_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 1 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port1_av_node_index,
            )
            .expect("Unable to add ports av -> port 1 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port1_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 1 av -> port prop edge");

        let port2_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port2_av_content_hash = content_store
            .add(&serde_json::json!("Port 2"))
            .expect("Unable to add to content store");
        let port2_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port2_av_id,
                    ContentAddress::AttributeValue(port2_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 2 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port2_av_node_index,
            )
            .expect("Unable to add ports av -> port 2 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port2_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 2 av -> port prop edge");

        let port3_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port3_av_content_hash = content_store
            .add(&serde_json::json!("Port 3"))
            .expect("Unable to add to content store");
        let port3_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port3_av_id,
                    ContentAddress::AttributeValue(port3_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 3 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port3_av_node_index,
            )
            .expect("Unable to add ports av -> port 3 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port3_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 3 av -> port prop edge");

        let port4_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port4_av_content_hash = content_store
            .add(&serde_json::json!("Port 4"))
            .expect("Unable to add to content store");
        let port4_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port4_av_id,
                    ContentAddress::AttributeValue(port4_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 4 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port4_av_node_index,
            )
            .expect("Unable to add ports av -> port 4 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port4_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 4 av -> port prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "ports": [
                        "Port 1",
                        "Port 2",
                        "Port 3",
                        "Port 4",
                    ],
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );

        let new_order = vec![port3_av_id, port1_av_id, port4_av_id, port2_av_id];
        graph
            .update_order(change_set, ports_av_id, new_order)
            .expect("Unable to update order of ports attribute value's children");
        assert_eq!(
            serde_json::json![{
                "domain": {
                    "ports": [
                        "Port 3",
                        "Port 1",
                        "Port 4",
                        "Port 2",
                    ]
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );

        let port5_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let port5_av_content_hash = content_store
            .add(&serde_json::json!("Port 5"))
            .expect("Unable to add to content store");
        let port5_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    port5_av_id,
                    ContentAddress::AttributeValue(port5_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add port 5 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(ports_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                port5_av_node_index,
            )
            .expect("Unable to add ports av -> port 5 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(port5_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(port_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add port 5 av -> port prop edge");

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "ports": [
                        "Port 3",
                        "Port 1",
                        "Port 4",
                        "Port 2",
                        "Port 5",
                    ]
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );
    }

    #[tokio::test]
    #[cfg(ignore)]
    async fn attribute_value_build_view_ordered_map() {
        let change_set = ChangeSet::new_local().expect("Unable to create ChangeSet");
        let change_set = &change_set;
        let mut graph = WorkspaceSnapshotGraph::new(change_set)
            .expect("Unable to create WorkspaceSnapshotGraph");
        let mut content_store = content_store::LocalStore::default();

        let schema_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_content_hash = content_store
            .add(&serde_json::json!("Schema A"))
            .expect("Unable to add to content store");
        let schema_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_id,
                    ContentAddress::Schema(schema_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_node_index,
            )
            .expect("Unable to add root -> schema edge");

        let schema_variant_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let schema_variant_content_hash = content_store
            .add(&serde_json::json!("Schema Variant A"))
            .expect("Unable to add to content store");
        let schema_variant_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    schema_variant_id,
                    ContentAddress::SchemaVariant(schema_variant_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add schema variant");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                schema_variant_node_index,
            )
            .expect("Unable to add schema -> schema variant edge");

        let root_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_prop_content_hash = content_store
            .add(&serde_json::json!("Root prop"))
            .expect("Unable to add to content store");
        let root_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    root_prop_id,
                    PropKind::Object,
                    "root",
                    root_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_prop_node_index,
            )
            .expect("Unable to add schema variant -> root prop edge");

        let domain_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let domain_prop_content_hash = content_store
            .add(&serde_json::json!("domain Prop Content"))
            .expect("Unable to add to content store");
        let domain_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    domain_prop_id,
                    PropKind::Object,
                    "domain",
                    domain_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                domain_prop_node_index,
            )
            .expect("Unable to add root prop -> domain prop edge");

        let environment_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let environment_prop_content_hash = content_store
            .add(&serde_json::json!("environment Prop Content"))
            .expect("Unable to add to content store");
        let environment_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    environment_prop_id,
                    PropKind::Array,
                    "environment",
                    environment_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add environment prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                environment_prop_node_index,
            )
            .expect("Unable to add domain prop -> environment prop edge");

        let env_var_prop_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var_prop_content_hash = content_store
            .add(&serde_json::json!("port Prop Content"))
            .expect("Unable to add to content store");
        let env_var_prop_node_index = graph
            .add_node(
                NodeWeight::new_prop(
                    change_set,
                    env_var_prop_id,
                    PropKind::String,
                    "port",
                    env_var_prop_content_hash,
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var prop");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(environment_prop_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                env_var_prop_node_index,
            )
            .expect("Unable to add environment prop -> env var prop edge");

        let component_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let component_content_hash = content_store
            .add(&serde_json::json!("Component Content"))
            .expect("Unable to add to content store");
        let component_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    component_id,
                    ContentAddress::Component(component_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add component");
        graph
            .add_edge(
                graph.root_index,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                component_node_index,
            )
            .expect("Unable to add root -> component edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(schema_variant_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add component -> schema variant edge");

        let root_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let root_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let root_av_node_index = graph
            .add_node(
                NodeWeight::new_attribute_value(change_set, root_av_id, None, None, None)
                    .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add root av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(component_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())
                    .expect("Unable to create EdgeWeight"),
                root_av_node_index,
            )
            .expect("Unable to add component -> root av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(root_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add root av -> root prop edge");

        let domain_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let domain_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let domain_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    domain_av_id,
                    ContentAddress::AttributeValue(domain_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add domain av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(root_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                domain_av_node_index,
            )
            .expect("Unable to add root av -> domain av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(domain_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add domain av -> domain prop edge");

        let envrionment_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let ports_av_content_hash = content_store
            .add(&serde_json::json!({}))
            .expect("Unable to add to content store");
        let environment_av_node_index = graph
            .add_ordered_node(
                change_set,
                NodeWeight::new_content(
                    change_set,
                    envrionment_av_id,
                    ContentAddress::AttributeValue(ports_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add environment av");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(domain_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(None))
                    .expect("Unable to create EdgeWeight"),
                environment_av_node_index,
            )
            .expect("Unable to add domain av -> environment av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(environment_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to create environment av -> environment prop edge");

        let env_var1_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var1_av_content_hash = content_store
            .add(&serde_json::json!("1111"))
            .expect("Unable to add to content store");
        let port1_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var1_av_id,
                    ContentAddress::AttributeValue(env_var1_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env_var 1 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_1".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                port1_av_node_index,
            )
            .expect("Unable to add environment av -> env var 1 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var1_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 1 av -> env var prop edge");

        let env_var2_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var2_av_content_hash = content_store
            .add(&serde_json::json!("2222"))
            .expect("Unable to add to content store");
        let env_var2_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var2_av_id,
                    ContentAddress::AttributeValue(env_var2_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 2 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_2".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                env_var2_av_node_index,
            )
            .expect("Unable to add environment av -> env var 2 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var2_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 2 av -> env var prop edge");

        let env_var3_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var3_av_content_hash = content_store
            .add(&serde_json::json!("3333"))
            .expect("Unable to add to content store");
        let port3_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var3_av_id,
                    ContentAddress::AttributeValue(env_var3_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 3 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_3".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                port3_av_node_index,
            )
            .expect("Unable to add environment av -> env var 3 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var3_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 3 av -> env var prop edge");

        let env_var4_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var4_av_content_hash = content_store
            .add(&serde_json::json!("4444"))
            .expect("Unable to add to content store");
        let env_var4_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var4_av_id,
                    ContentAddress::AttributeValue(env_var4_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 4 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_4".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                env_var4_av_node_index,
            )
            .expect("Unable to add environment av -> env var 4 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var4_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 4 av -> env var prop edge");

        graph.cleanup();
        graph.dot();

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "environment": {
                        "PORT_1": "1111",
                        "PORT_2": "2222",
                        "PORT_3": "3333",
                        "PORT_4": "4444",
                    },
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );

        let new_order = vec![
            env_var3_av_id,
            env_var1_av_id,
            env_var4_av_id,
            env_var2_av_id,
        ];
        graph
            .update_order(change_set, envrionment_av_id, new_order)
            .expect("Unable to update order of environment attribute value's children");
        assert_eq!(
            serde_json::json![{
                "domain": {
                    "environment": {
                        "PORT_3": "3333",
                        "PORT_1": "1111",
                        "PORT_4": "4444",
                        "PORT_2": "2222",
                    },
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );

        let env_var5_av_id = change_set.generate_ulid().expect("Unable to generate Ulid");
        let env_var5_av_content_hash = content_store
            .add(&serde_json::json!("5555"))
            .expect("Unable to add to content store");
        let env_var5_av_node_index = graph
            .add_node(
                NodeWeight::new_content(
                    change_set,
                    env_var5_av_id,
                    ContentAddress::AttributeValue(env_var5_av_content_hash),
                )
                .expect("Unable to create NodeWeight"),
            )
            .expect("Unable to add env var 5 av");
        graph
            .add_ordered_edge(
                change_set,
                graph
                    .get_node_index_by_id(envrionment_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(
                    change_set,
                    EdgeWeightKind::Contain(Some("PORT_5".to_string())),
                )
                .expect("Unable to create EdgeWeight"),
                env_var5_av_node_index,
            )
            .expect("Unable to add environment av -> env var 5 av edge");
        graph
            .add_edge(
                graph
                    .get_node_index_by_id(env_var5_av_id)
                    .expect("Unable to get NodeIndex"),
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)
                    .expect("Unable to create EdgeWeight"),
                graph
                    .get_node_index_by_id(env_var_prop_id)
                    .expect("Unable to get NodeIndex"),
            )
            .expect("Unable to add env var 5 av -> env var prop edge");

        assert_eq!(
            serde_json::json![{
                "domain": {
                    "environment": {
                        "PORT_3": "3333",
                        "PORT_1": "1111",
                        "PORT_4": "4444",
                        "PORT_2": "2222",
                        "PORT_5": "5555",
                    },
                }
            }],
            graph
                .attribute_value_view(
                    &mut content_store,
                    graph
                        .get_node_index_by_id(root_av_id)
                        .expect("Unable to get NodeIndex"),
                )
                .await
                .expect("Unable to generate attribute value view"),
        );
    }
}
