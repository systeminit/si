use std::{
    collections::{
        HashMap,
        HashSet,
    },
    io::Read,
};

use chrono::{
    DateTime,
    TimeZone,
    Utc,
};
use dal::{
    ComponentType,
    ContentHash,
    DalContext,
    EdgeWeight,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    PropKind,
    SocketArity,
    SocketKind,
    Ulid,
    WorkspaceSnapshotGraph,
    WorkspaceSnapshotGraphVCurrent,
    action::{
        ActionCompletionStatus,
        prototype::ActionKind,
    },
    approval_requirement::ApprovalRequirementApprover,
    attribute::path::AttributePath,
    func::{
        FuncKind,
        argument::FuncArgumentKind,
    },
    layer_db_types::{
        ApprovalRequirementDefinitionContent,
        AttributePrototypeContent,
        AttributePrototypeContentV1,
        ComponentContent,
        ContentTypes,
        ContentTypesDiscriminants,
        DeprecatedActionBatchContent,
        DeprecatedActionContent,
        DeprecatedActionPrototypeContent,
        DeprecatedActionRunnerContent,
        DeprecatedActionRunnerContentV1,
        FuncArgumentContent,
        FuncArgumentContentV1,
        FuncContent,
        FuncContentV3,
        GeometryContent,
        InputSocketContent,
        InputSocketContentV2,
        ManagementPrototypeContent,
        ManagementPrototypeContentV1,
        ModuleContent,
        OutputSocketContent,
        PropContent,
        PropContentV2,
        SchemaContent,
        SchemaVariantContent,
        SecretContent,
        StaticArgumentValueContent,
        ValidationContent,
        ViewContent,
    },
    prop::WidgetOption,
    socket::connection_annotation::ConnectionAnnotation,
    validation::ValidationStatus,
    workspace_snapshot::{
        content_address::ContentAddress,
        node_weight::{
            AttributePrototypeArgumentNodeWeight,
            CategoryNodeWeight,
            NodeWeight,
            OrderingNodeWeight,
            category_node_weight::CategoryNodeKind,
            diagram_object_node_weight::DiagramObjectKind,
            reason_node_weight::Reason,
            traits::SiVersionedNodeWeight,
        },
    },
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use serde_json::json;
use si_events::{
    CasValue,
    EncryptedSecretKey,
    Timestamp,
};
use si_layer_cache::db::serialize;
use strum::IntoEnumIterator;

const CURRENT_SERIALIZED_GRAPH_DIR_PATH: &str = "./lib/dal/tests";
const CURRENT_SERIALIZED_GRAPH_FILENAME: &str = "serialization-test-data-2024-11-21.snapshot";
const CURRENT_SERIALIZED_CONTENT_FILENAME: &str =
    "serialization-test-content-store-data-2025-06-10.bin";

// If you're modifying this, you probably just added a new node or edge weight. Before you replace
// the snapshot with one that includes the new weights, ensure that your current code passes the
// deserialization test against the snapshot with the *old* weights. If it does, update the
// validation snapshot file.
#[allow(unused)]
async fn make_me_one_with_everything(ctx: &DalContext) -> WorkspaceSnapshotGraph {
    let mut graph = WorkspaceSnapshotGraphVCurrent::new(ctx)
        .await
        .expect("make new");

    let mut node_indexes = vec![];

    // For every enum that goes into a node weight, try to pick the last variant
    // in the enum, so on the verification pass we will break if anything is
    // added out of order
    for node_kind in NodeWeightDiscriminants::iter() {
        let weight = match node_kind {
            NodeWeightDiscriminants::Action => {
                NodeWeight::new_action(Ulid::new().into(), Ulid::new(), Ulid::new())
            }
            NodeWeightDiscriminants::ActionPrototype => NodeWeight::new_action_prototype(
                Ulid::new(),
                Ulid::new(),
                ActionKind::Update,
                "foo".into(),
                None,
            ),
            NodeWeightDiscriminants::AttributePrototypeArgument => {
                NodeWeight::AttributePrototypeArgument(
                    AttributePrototypeArgumentNodeWeight::new_for_deserialization_test(),
                )
            }
            NodeWeightDiscriminants::AttributeValue => NodeWeight::new_attribute_value(
                Ulid::new(),
                Ulid::new(),
                Some(ContentAddress::JsonValue(ContentHash::new(
                    "foo".as_bytes(),
                ))),
                Some(ContentAddress::JsonValue(ContentHash::new(
                    "foo".as_bytes(),
                ))),
            ),
            NodeWeightDiscriminants::Category => NodeWeight::Category(CategoryNodeWeight::new(
                Ulid::new(),
                Ulid::new(),
                CategoryNodeKind::DependentValueRoots,
            )),
            NodeWeightDiscriminants::Component => NodeWeight::new_component(
                Ulid::new(),
                Ulid::new(),
                ContentHash::new("bar".as_bytes()),
            ),
            NodeWeightDiscriminants::Content => NodeWeight::new_content(
                Ulid::new(),
                Ulid::new(),
                ContentAddress::ManagementPrototype(ContentHash::new("baz".as_bytes())),
            ),
            NodeWeightDiscriminants::DependentValueRoot => {
                NodeWeight::new_dependent_value_root(Ulid::new(), Ulid::new(), Ulid::new())
            }
            NodeWeightDiscriminants::Func => NodeWeight::new_func(
                Ulid::new(),
                Ulid::new(),
                "foo",
                FuncKind::Management,
                ContentHash::new("quux".as_bytes()),
            ),
            NodeWeightDiscriminants::FuncArgument => NodeWeight::new_func_argument(
                Ulid::new(),
                Ulid::new(),
                "arg",
                ContentHash::new("the attitude era".as_bytes()),
            ),
            NodeWeightDiscriminants::Ordering => {
                NodeWeight::Ordering(OrderingNodeWeight::new(Ulid::new(), Ulid::new()))
            }
            NodeWeightDiscriminants::Prop => NodeWeight::new_prop(
                Ulid::new(),
                Ulid::new(),
                PropKind::String,
                "foo",
                ContentHash::new("bar".as_bytes()),
            ),
            NodeWeightDiscriminants::Secret => NodeWeight::new_secret(
                Ulid::new(),
                Ulid::new(),
                EncryptedSecretKey::new("shhh".as_bytes()),
                ContentHash::new("content".as_bytes()),
            ),
            NodeWeightDiscriminants::FinishedDependentValueRoot => {
                NodeWeight::new_finished_dependent_value_root(Ulid::new(), Ulid::new(), Ulid::new())
            }
            NodeWeightDiscriminants::InputSocket => NodeWeight::new_input_socket(
                Ulid::new(),
                Ulid::new(),
                SocketArity::Many,
                ContentHash::new("bar".as_bytes()),
            ),
            NodeWeightDiscriminants::SchemaVariant => NodeWeight::new_schema_variant(
                Ulid::new(),
                Ulid::new(),
                false,
                ContentHash::new("variant".as_bytes()),
            ),
            NodeWeightDiscriminants::ManagementPrototype => NodeWeight::new_management_prototype(
                Ulid::new(),
                Ulid::new(),
                ContentHash::new("management".as_bytes()),
            ),
            NodeWeightDiscriminants::Geometry => NodeWeight::new_geometry(
                Ulid::new(),
                Ulid::new(),
                ContentHash::new("geometry".as_bytes()),
            ),
            NodeWeightDiscriminants::View => NodeWeight::new_view(
                Ulid::new(),
                Ulid::new(),
                ContentHash::new("view".as_bytes()),
            ),
            NodeWeightDiscriminants::DiagramObject => NodeWeight::new_diagram_object(
                Ulid::new(),
                Ulid::new(),
                DiagramObjectKind::View(Ulid::new().into()),
            ),
            NodeWeightDiscriminants::ApprovalRequirementDefinition => {
                NodeWeight::new_approval_requirement_definition(
                    Ulid::new(),
                    Ulid::new(),
                    ContentHash::new("stellaris hurts my brain".as_bytes()),
                )
            }
            NodeWeightDiscriminants::Reason => {
                NodeWeight::new_reason(Ulid::new(), Ulid::new(), Reason::UserAdded(None))
            }
        };

        let idx = graph.add_or_replace_node(weight).expect("add node");
        // Attach to root
        graph
            .add_edge(
                graph.root(),
                EdgeWeight::new(EdgeWeightKind::new_use()),
                idx,
            )
            .expect("add edge");
        node_indexes.push(idx);
    }

    let mut last_node = 0;

    for edge_kind in EdgeWeightKindDiscriminants::iter() {
        let edge_weight_kind = match edge_kind {
            EdgeWeightKindDiscriminants::Action => EdgeWeightKind::Action,
            EdgeWeightKindDiscriminants::ActionPrototype => EdgeWeightKind::ActionPrototype,
            EdgeWeightKindDiscriminants::AuthenticationPrototype => {
                EdgeWeightKind::AuthenticationPrototype
            }
            EdgeWeightKindDiscriminants::Contain => EdgeWeightKind::Contain(Some("foo".to_owned())),
            EdgeWeightKindDiscriminants::DeprecatedFrameContains => {
                EdgeWeightKind::DeprecatedFrameContains
            }
            EdgeWeightKindDiscriminants::Ordering => EdgeWeightKind::Ordering,
            EdgeWeightKindDiscriminants::Ordinal => EdgeWeightKind::Ordinal,
            EdgeWeightKindDiscriminants::Prop => EdgeWeightKind::Prop,
            EdgeWeightKindDiscriminants::Prototype => {
                EdgeWeightKind::Prototype(Some("bar".to_owned()))
            }
            EdgeWeightKindDiscriminants::PrototypeArgument => EdgeWeightKind::PrototypeArgument,
            EdgeWeightKindDiscriminants::PrototypeArgumentValue => {
                EdgeWeightKind::PrototypeArgumentValue
            }
            EdgeWeightKindDiscriminants::Proxy => EdgeWeightKind::Proxy,
            EdgeWeightKindDiscriminants::Root => EdgeWeightKind::Root,
            EdgeWeightKindDiscriminants::Socket => EdgeWeightKind::Socket,
            EdgeWeightKindDiscriminants::SocketValue => EdgeWeightKind::SocketValue,
            EdgeWeightKindDiscriminants::Use => EdgeWeightKind::new_use(),
            EdgeWeightKindDiscriminants::ValidationOutput => EdgeWeightKind::ValidationOutput,
            EdgeWeightKindDiscriminants::ManagementPrototype => EdgeWeightKind::ManagementPrototype,
            EdgeWeightKindDiscriminants::Represents => EdgeWeightKind::Represents,
            EdgeWeightKindDiscriminants::Manages => EdgeWeightKind::Manages,
            EdgeWeightKindDiscriminants::DiagramObject => EdgeWeightKind::DiagramObject,
            EdgeWeightKindDiscriminants::ApprovalRequirementDefinition => {
                EdgeWeightKind::ApprovalRequirementDefinition
            }
            EdgeWeightKindDiscriminants::ValueSubscription => {
                EdgeWeightKind::ValueSubscription(AttributePath::from_json_pointer("/json_pointer"))
            }
            EdgeWeightKindDiscriminants::DefaultSubscriptionSource => {
                EdgeWeightKind::DefaultSubscriptionSource
            }
            EdgeWeightKindDiscriminants::Reason => EdgeWeightKind::Reason,
        };

        if last_node + 1 == node_indexes.len() {
            last_node = 0;
        };

        let edge_weight = EdgeWeight::new(edge_weight_kind);

        graph
            .add_edge(
                node_indexes[last_node],
                edge_weight,
                node_indexes[last_node + 1],
            )
            .expect("add edge");

        last_node += 1;
    }

    WorkspaceSnapshotGraph::V4(graph)
}

fn make_static_utc() -> DateTime<Utc> {
    // The windows 95 release date
    Utc.with_ymd_and_hms(1995, 8, 24, 0, 0, 0).unwrap()
}

fn make_static_timestamp() -> Timestamp {
    Timestamp {
        created_at: make_static_utc(),
        updated_at: make_static_utc(),
    }
}

fn make_static_ulid<T>() -> T
where
    T: From<Ulid>,
{
    T::from(Ulid::from_string("01GW28EN4ZKTD192J5EFC3P3K0").unwrap())
}

fn make_me_one_with_everything_content_types_edition() -> Vec<ContentTypes> {
    let mut result = vec![];
    let timestamp = make_static_timestamp();
    let utc = make_static_utc();

    let cas_value = CasValue::from(
        serde_json::json!({ "null": null, "bool": true, "number": 1.618033988, "string": "string", "array": ["of", "string"] }),
    );

    for content_type_kind in ContentTypesDiscriminants::iter() {
        result.push(match content_type_kind {
            ContentTypesDiscriminants::Any => ContentTypes::Any(cas_value.clone()),
            ContentTypesDiscriminants::AttributePrototype => {
                AttributePrototypeContent::V1(AttributePrototypeContentV1 { timestamp }).into()
            }
            ContentTypesDiscriminants::Component => {
                ComponentContent::V2(dal::layer_db_types::ComponentContentV2 { timestamp }).into()
            }
            ContentTypesDiscriminants::DeprecatedAction => {
                DeprecatedActionContent::V1(dal::layer_db_types::DeprecatedActionContentV1 {
                    creation_user_pk: Some(make_static_ulid()),
                    timestamp,
                })
                .into()
            }
            ContentTypesDiscriminants::DeprecatedActionBatch => DeprecatedActionBatchContent::V1(
                dal::layer_db_types::DeprecatedActionBatchContentV1 {
                    author: "Schaffa".into(),
                    actors: "Edki Guardian Warrant".into(),
                    started_at: Some(utc),
                    finished_at: Some(utc),
                    completion_status: Some(ActionCompletionStatus::Success),
                    timestamp: make_static_timestamp(),
                },
            )
            .into(),
            ContentTypesDiscriminants::DeprecatedActionPrototype => {
                DeprecatedActionPrototypeContent::V1(
                    dal::layer_db_types::DeprecatedActionPrototypeContentV1 {
                        kind: ActionKind::Create,
                        name: Some("orogeny".into()),
                        timestamp,
                    },
                )
                .into()
            }
            ContentTypesDiscriminants::DeprecatedActionRunner => {
                DeprecatedActionRunnerContent::V1(DeprecatedActionRunnerContentV1 {
                    timestamp,
                    component_id: make_static_ulid(),
                    component_name: "Jija".into(),
                    schema_name: "The Lost Moon".into(),
                    func_name: "Syl Anagist".into(),
                    action_prototype_id: make_static_ulid(),
                    action_kind: ActionKind::Destroy,
                    resource: Some("sessapinae".into()),
                    started_at: Some(utc),
                    finished_at: Some(utc),
                    completion_status: Some(ActionCompletionStatus::Unstarted),
                    completion_message: Some("the onyx".into()),
                })
                .into()
            }
            ContentTypesDiscriminants::Func => FuncContent::V3(FuncContentV3 {
                timestamp,
                display_name: Some("Binof".into()),
                description: Some("Leadership Yumenes".into()),
                link: Some("https://zombo.com".into()),
                hidden: true,
                builtin: true,
                backend_response_type: dal::FuncBackendResponseType::SchemaVariantDefinition,
                backend_kind: dal::FuncBackendKind::JsSchemaVariantDefinition,
                handler: Some("handler".into()),
                code_base64: Some("this isn't base64, is it".into()),
                code_blake3: ContentHash::nil(),
                is_locked: true,
                is_transformation: false,
            })
            .into(),
            ContentTypesDiscriminants::FuncArgument => {
                FuncArgumentContent::V1(FuncArgumentContentV1 {
                    kind: FuncArgumentKind::String,
                    element_kind: Some(FuncArgumentKind::String),
                    timestamp,
                })
                .into()
            }
            ContentTypesDiscriminants::InputSocket => {
                InputSocketContent::V2(InputSocketContentV2 {
                    timestamp,
                    name: "That port they use to jack into the Matrix".into(),
                    inbound_type_definition: Some("the type of all types".into()),
                    outbound_type_definition: Some("the barber who cuts their own hair".into()),
                    kind: SocketKind::Standard,
                    required: true,
                    ui_hidden: true,
                    connection_annotations: vec![ConnectionAnnotation::from_tokens_array(vec![
                        "a".into(),
                        "b".into(),
                    ])],
                })
                .into()
            }
            ContentTypesDiscriminants::Module => {
                ModuleContent::V2(dal::layer_db_types::ModuleContentV2 {
                    timestamp,
                    name: "Mod yule? That's changing christmas".into(),
                    root_hash: "abcdefghijkl".into(),
                    version: "turning and turning in the widening gyre".into(),
                    description: "the falcon cannot hear the falconer".into(),
                    created_by_email: "yeats@sandymount".into(),
                    created_at: utc,
                    schema_id: Some(make_static_ulid()),
                })
                .into()
            }
            ContentTypesDiscriminants::Prop => PropContent::V2(PropContentV2 {
                timestamp,
                name: "a prop?".into(),
                kind: PropKind::Float,
                widget_kind: dal::property_editor::schema::WidgetKind::TextArea,
                widget_options: Some(vec![WidgetOption {
                    label: "prim".into(),
                    value: "and proper".into(),
                }]),
                doc_link: Some("https://en.wikipedia.org/wiki/RTFM".into()),
                documentation: Some("papers please".into()),
                hidden: true,
                refers_to_prop_id: Some(make_static_ulid()),
                diff_func_id: Some(make_static_ulid()),
                validation_format: Some("smack barm pey wet".into()),
                ui_optionals: Some(HashMap::from([
                    ("suggestSources".to_owned(), json!([{ "schema": "AWS::EC2::VPC", "prop": "/resource_value/VpcId" }]).into()),
                    ("suggestAsSourceFor".to_owned(), json!([{ "schema": "AWS::EC2::Subnet", "prop": "/resource_value/SubnetId" }]).into())
                ])),
            })
            .into(),
            ContentTypesDiscriminants::Schema => {
                SchemaContent::V1(dal::layer_db_types::SchemaContentV1 {
                    timestamp,
                    name: "A schema is a plan".into(),
                    ui_hidden: true,
                    is_builtin: true,
                })
                .into()
            }
            ContentTypesDiscriminants::SchemaVariant => {
                SchemaVariantContent::V3(dal::layer_db_types::SchemaVariantContentV3 {
                    timestamp,
                    ui_hidden: true,
                    version: "1.0".into(),
                    display_name: "display name".into(),
                    category: "someday a real rain will come".into(),
                    color: "vantablack".into(),
                    component_type: ComponentType::ConfigurationFrameUp,
                    link: Some("https://novell.com".into()),
                    description: Some("a variant of a plan".into()),
                    asset_func_id: Some(make_static_ulid()),
                    finalized_once: true,
                    is_builtin: true,
                })
                .into()
            }
            ContentTypesDiscriminants::Secret => {
                SecretContent::V1(dal::layer_db_types::SecretContentV1 {
                    timestamp,
                    created_by: Some(make_static_ulid()),
                    updated_by: Some(make_static_ulid()),
                    name: "shhhhhhh! mum's the word".into(),
                    definition: "look it up in the dictionary".into(),
                    description: Some("just don't add a journalist to your signal".into()),
                })
                .into()
            }
            ContentTypesDiscriminants::StaticArgumentValue => {
                StaticArgumentValueContent::V1(dal::layer_db_types::StaticArgumentValueContentV1 {
                    timestamp,
                    value: cas_value.clone(),
                })
                .into()
            }
            ContentTypesDiscriminants::Validation => {
                ValidationContent::V1(dal::layer_db_types::ValidationContentV1 {
                    timestamp,
                    status: ValidationStatus::Success,
                    message: Some(
                        "valid comes from the latin 'validus', meaning strong, healthy and shares an indo-germanic root with 'well'".into(),
                    ),
                })
                .into()
            }
            ContentTypesDiscriminants::OutputSocket => {
                OutputSocketContent::V1(dal::layer_db_types::OutputSocketContentV1 {
                    timestamp,
                    name: "GIGO".into(),
                    type_definition: Some(
                        "a type is a set of x such that for all x, P(x) is true".into(),
                    ),
                    arity: SocketArity::One,
                    kind: SocketKind::Standard,
                    required: true,
                    ui_hidden: true,
                    connection_annotations: vec![ConnectionAnnotation::from_tokens_array(vec![
                        "a".into(),
                        "b".into(),
                    ])],
                })
                .into()
            }
            ContentTypesDiscriminants::ManagementPrototype => {
                ManagementPrototypeContent::V1(ManagementPrototypeContentV1 {
                    name: "demenager".into(),
                    description: Some("debrouiller".into()),
                })
                .into()
            }
            ContentTypesDiscriminants::Geometry => {
                GeometryContent::V1(dal::layer_db_types::GeometryContentV1 {
                    timestamp,
                    x: "pi".into(),
                    y: "e".into(),
                    width: Some("why are these".into()),
                    height: Some("strings".into()),
                })
                .into()
            }
            ContentTypesDiscriminants::View => {
                ViewContent::V1(dal::layer_db_types::ViewContentV1 {
                    timestamp,
                    name: "buena vista".into(),
                })
                .into()
            }
            ContentTypesDiscriminants::ApprovalRequirementDefinition => {
                ApprovalRequirementDefinitionContent::V1(
                    dal::layer_db_types::ApprovalRequirementDefinitionContentV1 {
                        minimum: 1_000_000,
                        approvers: HashSet::from([ApprovalRequirementApprover::User(
                            make_static_ulid(),
                        )]),
                    },
                )
                .into()
            }
        });
    }

    result
}

// Run this test to produce a serialized version of the graph. Do this any time
// a breaking change in serialization occurs.
//
// cd to the root of "si"
// then run:
//
// $ buck2 run //lib/dal:test-integration -- write_deserialization_data --ignored
//
// Then delete the old copy of the graph, and replace the constant
// `CURRENT_SERIALIZED_GRAPH_FILENAME` with the filename of the new graph.
#[test]
#[ignore = "only run this when you want to produce a new serialized graph"]
async fn write_deserialization_data(ctx: &DalContext) {
    let graph = make_me_one_with_everything(ctx).await;

    let (serialized, _) = serialize::to_vec(&graph).expect("serialize");

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let mut file = std::fs::File::create(format!(
        "{CURRENT_SERIALIZED_GRAPH_DIR_PATH}/serialization-test-data-{date}.snapshot"
    ))
    .expect("create file");
    file.write_all(&serialized).expect("write file");
}

// Run this test to produce a serialized version of the content types. Do this any time
// a breaking change in serialization occurs.
//
// cd to the root of "si"
// then run:
//
// $ buck2 run //lib/dal:test-integration -- write_deserialization_content_store_data --ignored
//
// Then delete the old copy of the graph, and replace the constant
// `CURRENT_SERIALIZED_CONTENT_FILENAME` with the filename of the new graph.
#[test]
#[ignore = "only run this when you want to produce a new serialized set of content store objects"]
async fn write_deserialization_content_store_data() {
    let content = make_me_one_with_everything_content_types_edition();

    let (serialized, _) = serialize::to_vec(&content).expect("serialize");

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let mut file = std::fs::File::create(format!(
        "{CURRENT_SERIALIZED_GRAPH_DIR_PATH}/serialization-test-content-store-data-{date}.bin"
    ))
    .expect("create file");
    file.write_all(&serialized).expect("write file");
}

#[test]
async fn content_can_be_deserialized(_ctx: &DalContext) {
    let mut file = std::fs::File::open(dbg!(format!(
        "{CURRENT_SERIALIZED_GRAPH_DIR_PATH}/{CURRENT_SERIALIZED_CONTENT_FILENAME}"
    )))
    .expect("open file");
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).expect("able to read bytes");

    let content: Vec<ContentTypes> = serialize::from_bytes(&bytes).expect("deserialize");
    let expected = make_me_one_with_everything_content_types_edition();

    assert_eq!(expected, content);
}

#[test]
async fn content_can_be_serialized_and_then_deserialized(_ctx: &DalContext) {
    let original_content = make_me_one_with_everything_content_types_edition();
    let (bytes, _) = serialize::to_vec(&original_content).expect("serialize");

    let content: Vec<ContentTypes> = serialize::from_bytes(&bytes).expect("deserialize");
    let expected = make_me_one_with_everything_content_types_edition();

    assert_eq!(expected, content);
}

// This tests that old versions of the graph can be deserialized.
#[test]
async fn graph_can_be_deserialized(_ctx: &DalContext) {
    let mut file = std::fs::File::open(format!(
        "{CURRENT_SERIALIZED_GRAPH_DIR_PATH}/{CURRENT_SERIALIZED_GRAPH_FILENAME}"
    ))
    .expect("open file");
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).expect("able to read bytes");

    let graph: WorkspaceSnapshotGraph = serialize::from_bytes(&bytes).expect("deserialize");
    assert_eq!(32, graph.node_count());
    verify_graph_invariants(&graph);
}

// This tests that the *current* version of the graph can be round-tripped.
#[test]
async fn graph_can_be_serialized_and_then_deserialized(ctx: &DalContext) {
    let original_graph = make_me_one_with_everything(ctx).await;
    let (bytes, _) = serialize::to_vec(&original_graph).expect("serialize");

    let graph: WorkspaceSnapshotGraph = serialize::from_bytes(&bytes).expect("deserialize");
    assert_eq!(35, graph.node_count());
    verify_graph_invariants(&graph);
}

fn verify_graph_invariants(graph: &WorkspaceSnapshotGraph) {
    // Where we can, verify that the enums on the node weights match what we expect
    for (node_weight, _) in graph.nodes() {
        match node_weight {
            NodeWeight::Action(_) => {}
            NodeWeight::ActionPrototype(action_prototype_node_weight) => {
                assert_eq!(ActionKind::Update, action_prototype_node_weight.kind());
            }
            NodeWeight::AttributePrototypeArgument(_) => {}
            NodeWeight::AttributeValue(_) => {}
            NodeWeight::Category(_) => {}
            NodeWeight::Component(_) => {}
            NodeWeight::Content(_) => {}
            NodeWeight::DependentValueRoot(_) => {}
            NodeWeight::Func(func_node_weight) => {
                assert_eq!(FuncKind::Management, func_node_weight.func_kind());
            }
            NodeWeight::FuncArgument(_) => {}
            NodeWeight::Ordering(_) => {}
            NodeWeight::Prop(prop_node_weight) => {
                assert_eq!(PropKind::String, prop_node_weight.kind());
            }
            NodeWeight::Secret(_) => {}
            NodeWeight::FinishedDependentValueRoot(_) => {}
            NodeWeight::InputSocket(input_socket_node_weight) => {
                assert_eq!(SocketArity::Many, input_socket_node_weight.inner().arity());
            }
            NodeWeight::SchemaVariant(_) => {}
            NodeWeight::ManagementPrototype(_) => {}
            NodeWeight::Geometry(_) => {}
            NodeWeight::View(_) => {}
            NodeWeight::DiagramObject(_) => {}
            NodeWeight::ApprovalRequirementDefinition(_) => {}
            NodeWeight::Reason(reason) => {
                assert!(matches!(reason.inner().reason, Reason::UserAdded(_)));
            }
        }
    }
}
