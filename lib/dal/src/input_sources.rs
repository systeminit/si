use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

use crate::prop::PropError;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::{
    DalContext, InputSocket, InputSocketId, OutputSocket, OutputSocketId, Prop, PropId, PropKind,
    SchemaVariant, SchemaVariantError, SchemaVariantId, WorkspaceSnapshotError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum InputSourcesError {
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type InputSourcesResult<T> = Result<T, InputSourcesError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSocketView {
    pub schema_variant_id: SchemaVariantId,
    pub input_socket_id: InputSocketId,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputSocketView {
    pub schema_variant_id: SchemaVariantId,
    pub output_socket_id: OutputSocketId,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSourceProp {
    pub schema_variant_id: SchemaVariantId,
    pub prop_id: PropId,
    pub kind: PropKind,
    pub name: String,
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSources {
    pub input_sockets: Vec<InputSocketView>,
    pub output_sockets: Vec<OutputSocketView>,
    pub props: Vec<InputSourceProp>,
}

impl InputSources {
    /// Assemble [`InputSources`] for all [`SchemaVariants`](SchemaVariant) in the workspace.
    pub async fn assemble_for_all_schema_variants(ctx: &DalContext) -> InputSourcesResult<Self> {
        let all_schema_variant_ids = SchemaVariant::list_ids(ctx).await?;

        let mut input_socket_views = Vec::new();
        let mut output_socket_views = Vec::new();

        for schema_variant_id in &all_schema_variant_ids {
            let (input_socket_views_for_schema_variant, output_socket_views_for_schema_variant) =
                Self::assemble_socket_views(ctx, *schema_variant_id).await?;

            input_socket_views.extend(input_socket_views_for_schema_variant);
            output_socket_views.extend(output_socket_views_for_schema_variant);
        }

        let mut input_source_props = Vec::new();
        for schema_variant_id in all_schema_variant_ids {
            let input_source_props_for_schema_variant =
                Self::assemble_input_source_props(ctx, schema_variant_id).await?;
            input_source_props.extend(input_source_props_for_schema_variant);
        }

        Ok(Self {
            input_sockets: input_socket_views,
            output_sockets: output_socket_views,
            props: input_source_props,
        })
    }

    /// Assemble [`InputSources`] for the provided [`SchemaVariantId`](SchemaVariant) in the workspace.
    pub async fn assemble(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSourcesResult<Self> {
        let (input_socket_views, output_socket_views) =
            Self::assemble_socket_views(ctx, schema_variant_id).await?;
        let input_source_props = Self::assemble_input_source_props(ctx, schema_variant_id).await?;

        Ok(Self {
            input_sockets: input_socket_views,
            output_sockets: output_socket_views,
            props: input_source_props,
        })
    }

    async fn assemble_socket_views(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSourcesResult<(Vec<InputSocketView>, Vec<OutputSocketView>)> {
        let input_sockets = InputSocket::list(ctx, schema_variant_id).await?;
        let output_sockets = OutputSocket::list(ctx, schema_variant_id).await?;

        let input_socket_views = input_sockets
            .iter()
            .map(|socket| InputSocketView {
                schema_variant_id,
                input_socket_id: socket.id(),
                name: socket.name().to_owned(),
            })
            .collect();
        let output_socket_views = output_sockets
            .iter()
            .map(|socket| OutputSocketView {
                schema_variant_id,
                output_socket_id: socket.id(),
                name: socket.name().to_owned(),
            })
            .collect();

        Ok((input_socket_views, output_socket_views))
    }

    async fn assemble_input_source_props(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSourcesResult<Vec<InputSourceProp>> {
        let root_prop_id = SchemaVariant::get_root_prop_id(ctx, schema_variant_id).await?;
        let root_prop = Prop::get_by_id_or_error(ctx, root_prop_id).await?;

        let mut work_queue = VecDeque::new();
        work_queue.push_back(root_prop);

        // NOTE(nick): we may or may not need DFS-traversal with a corresponding action. Ye be warned!
        let mut input_socket_props = Vec::new();
        while let Some(prop) = work_queue.pop_front() {
            // TODO(nick): determine if we need to skip hidden props.
            // if prop.hidden {
            //     continue;
            // }

            // Only descend on object props.
            if PropKind::Object == prop.kind {
                work_queue.extend(Prop::direct_child_props_ordered(ctx, prop.id).await?);
            }

            input_socket_props.push(InputSourceProp {
                schema_variant_id,
                prop_id: prop.id,
                kind: prop.kind,
                name: prop.name.to_owned(),
                path: prop.path(ctx).await?.with_replaced_sep("/"),
            })
        }

        Ok(input_socket_props)
    }
}
