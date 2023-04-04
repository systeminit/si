use color_eyre::Result;
use convert_case::{Case, Casing};
use dal::{
    prop_tree::{PropTree, PropTreeNode},
    schema::variant::definition::{
        PropDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
        SocketDefinition,
    },
    AccessBuilder, ChangeSetPk, DalContext, ExternalProvider, HistoryActor, InternalProvider,
    NatsProcessor, Prop, PropId, PropKind, Schema, ServicesContext, StandardModel, Tenancy,
    Visibility, WorkspacePk,
};
use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::prelude::*,
    path::Path,
    sync::Arc,
};

pub(crate) mod args;
pub(crate) mod config;

fn make_prop_definition(node: &PropTreeNode, props: &HashMap<PropId, Prop>) -> PropDefinition {
    let mut children = node
        .children
        .iter()
        .map(|child| make_prop_definition(child, props))
        .collect();

    if node.kind != PropKind::Array {
        PropDefinition {
            name: node.name.clone(),
            kind: node.kind,
            doc_link_ref: None,
            doc_link: props
                .get(&node.prop_id)
                .expect("prop not in map")
                .doc_link()
                .map(|l| l.to_string()),
            children,
            entry: None,
            widget: None,
        }
    } else {
        PropDefinition {
            name: node.name.clone(),
            kind: node.kind,
            doc_link_ref: None,
            doc_link: props
                .get(&node.prop_id)
                .expect("prop not in map")
                .doc_link()
                .map(|l| l.to_string()),
            children: vec![],
            entry: children.pop().map(Box::new),
            widget: None,
        }
    }
}

async fn get_definitions_for_schema_name(
    ctx: &DalContext,
    schema_name: &str,
) -> Result<(
    SchemaVariantDefinitionMetadataJson,
    SchemaVariantDefinitionJson,
)> {
    let schema = Schema::schema_for_name(ctx, schema_name).await?;
    let default_variant = schema.default_variant(ctx).await?;

    let metadata_json = SchemaVariantDefinitionMetadataJson::from_schema_and_variant(
        ctx,
        &schema,
        &default_variant,
    )
    .await?;

    let root_prop = PropTree::new(ctx, false, Some(*default_variant.id()))
        .await?
        .root_props
        .pop()
        .expect("there should be a root prop");

    let domain = root_prop
        .children
        .into_iter()
        .find(|node| node.name == "domain")
        .expect("domain prop should exist");

    // Async recursion is too messy so we have to do a little work to get things in the right state
    // to generate the prop definitions
    let mut work_queue = VecDeque::from([&domain]);
    let mut props = HashMap::new();
    while let Some(cur) = work_queue.pop_front() {
        let prop = Prop::get_by_id(ctx, &cur.prop_id)
            .await?
            .expect("prop not found");
        props.insert(cur.prop_id, prop);
        for child in &cur.children {
            work_queue.push_front(child);
        }
    }

    let root_def = make_prop_definition(&domain, &props);

    let input_sockets = InternalProvider::list_for_input_sockets(ctx)
        .await?
        .iter()
        .filter_map(|ip| {
            if ip.schema_variant_id() == default_variant.id() && ip.name() != "Frame" {
                Some(SocketDefinition {
                    name: ip.name().to_string(),
                    arity: None, // XXX: grab arity
                })
            } else {
                None
            }
        })
        .collect();

    let output_sockets = ExternalProvider::list_for_schema_variant(ctx, *default_variant.id())
        .await?
        .iter()
        .filter_map(|ep| {
            if ep.name() != "Frame" {
                Some(SocketDefinition {
                    name: ep.name().to_string(),
                    arity: None,
                })
            } else {
                None
            }
        })
        .collect();

    let var_json = SchemaVariantDefinitionJson {
        props: root_def.children,
        input_sockets,
        output_sockets,
        doc_links: None,
    };

    Ok((metadata_json, var_json))
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dal::init()?;

    let args = args::parse();
    let schema_name = args.schema_name.clone();
    let output_dir = args.output_dir.clone();
    let config = config::Config::try_from(args)?;
    let encryption_key =
        veritech_client::EncryptionKey::load(config.cyclone_encryption_key_path()).await?;
    let nats = NatsClient::new(config.nats()).await?;
    let pg_pool = PgPool::new(config.pg_pool()).await?;
    let veritech = veritech_client::Client::new(nats.clone());
    let job_processor = NatsProcessor::new(nats.clone(), todo!("need to provide an mpsc sender"));
    let services_context = ServicesContext::new(
        pg_pool.clone(),
        nats.clone(),
        Box::new(job_processor),
        veritech.clone(),
        Arc::new(encryption_key),
        "council".to_owned(),
        None,
    );

    let access_builder =
        AccessBuilder::new(Tenancy::new(WorkspacePk::NONE), HistoryActor::SystemInit);
    let ctx_builder = DalContext::builder(services_context);
    let visibility = Visibility {
        change_set_pk: ChangeSetPk::NONE,
        deleted_at: None,
    };
    let ctx = ctx_builder.build(access_builder.build(visibility)).await?;

    let schema_names = if schema_name == "all" {
        Schema::list(&ctx)
            .await?
            .iter()
            .map(|schema| schema.name().to_string())
            .collect()
    } else {
        vec![schema_name]
    };

    for schema_name in schema_names {
        let (metadata_json, var_json) = get_definitions_for_schema_name(&ctx, &schema_name).await?;
        let snake_name = schema_name.to_case(Case::Snake);

        let category = if !metadata_json.category.is_empty() {
            format!("{}_", metadata_json.category.to_case(Case::Snake))
        } else {
            "".to_string()
        };

        let metadata_fn = Path::join(
            Path::new(&output_dir),
            format!("{}{}.metadata.json", &category, &snake_name),
        );
        let vardef_fn = Path::join(
            Path::new(&output_dir),
            format!("{}{}.json", &category, &snake_name),
        );

        println!(
            "Writing {} and {} for Schema {}",
            &metadata_fn.display(),
            &vardef_fn.display(),
            &schema_name
        );

        let mut metadata_file = File::create(metadata_fn)?;
        metadata_file.write_all(serde_json::to_string_pretty(&metadata_json)?.as_bytes())?;

        let mut vardef_file = File::create(vardef_fn)?;
        vardef_file.write_all(serde_json::to_string_pretty(&var_json)?.as_bytes())?;
    }

    Ok(())
}
