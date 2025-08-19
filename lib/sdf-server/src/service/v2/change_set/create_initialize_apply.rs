use axum::{
    Json,
    response::IntoResponse,
};
use dal::{
    ChangeSet,
    Component,
    DalContext,
    Prop,
    Schema,
    SchemaVariant,
    Secret,
    SecretAlgorithm,
    SecretVersion,
    diagram::view::View,
    prop::PropPath,
    property_editor::values::PropertyEditorValues,
};
use sdf_extract::{
    PosthogEventTracker,
    workspace::WorkspaceDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use si_id::KeyPairPk;

use super::Result;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AwsCredentialData {
    pub name: String,
    pub description: Option<String>,
    pub crypted: Vec<u8>,
    pub key_pair_pk: KeyPairPk,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub aws_region: String,
    pub credential: AwsCredentialData,
}

const INITIALIZATION_CHANGE_SET_NAME: &str = "Initialization";

pub async fn create_initialize_apply(
    WorkspaceDalContext(ref mut ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
    Json(request): Json<Request>,
) -> Result<impl IntoResponse> {
    let mut initialization_change_set =
        ChangeSet::fork_head(ctx, INITIALIZATION_CHANGE_SET_NAME).await?;
    ctx.write_audit_log(
        AuditLogKind::CreateChangeSet,
        INITIALIZATION_CHANGE_SET_NAME.to_string(),
    )
    .await?;

    let mut initialization_changeset_ctx =
        ctx.clone_with_new_visibility(initialization_change_set.id.into());
    let initialization_result =
        initialize_and_apply(&mut initialization_changeset_ctx, request).await;

    // Ensure that if something went wrong on initialization, we abandon the change set
    if initialization_result.is_err() {
        let old_status = initialization_change_set.status;
        initialization_change_set.abandon(ctx).await?;
        ctx.write_audit_log(
            AuditLogKind::AbandonChangeSet {
                from_status: old_status.into(),
            },
            initialization_change_set.name,
        )
        .await?;

        return initialization_result;
    }

    // TODO set fields on this new event
    tracker.track(
        ctx,
        "onboarded",
        serde_json::json!({
                    "change_set_name": INITIALIZATION_CHANGE_SET_NAME,
        }),
    );

    Ok(())
}

async fn initialize_and_apply(ctx: &mut DalContext, request: Request) -> Result<()> {
    // TODO: there's still some audit logs missing on this function
    let view_id = View::get_id_for_default(ctx).await?;
    let credential_name = request.credential.name;

    // Create the credential component
    let credential_component = {
        let schema = Schema::get_or_install_by_name(ctx, "AWS Credential").await?;
        let sv_id = SchemaVariant::default_id_for_schema(ctx, schema.id()).await?;
        let component = Component::new(ctx, &credential_name, sv_id, view_id).await?;

        // Create the secret and set it to the credential
        let secret = Secret::new(
            ctx,
            &credential_name,
            "AWS Credential".to_owned(),
            request.credential.description,
            &request.credential.crypted,
            request.credential.key_pair_pk,
            request.credential.version,
            request.credential.algorithm,
        )
        .await?;

        ctx.write_audit_log(
            AuditLogKind::CreateSecret {
                name: secret.name().to_string(),
                secret_id: secret.id(),
            },
            secret.name().to_string(),
        )
        .await?;

        // Find the AWS Credential prop and attach the secret
        {
            let aws_credential_prop = Prop::find_prop_by_path(
                ctx,
                sv_id,
                &PropPath::new(["root", "secrets", "AWS Credential"]),
            )
            .await?;

            let property_values = PropertyEditorValues::assemble(ctx, component.id()).await?;
            let aws_credential_attribute_value_id =
                property_values.find_by_prop_id_or_err(aws_credential_prop.id())?;

            Secret::attach_for_attribute_value(
                ctx,
                aws_credential_attribute_value_id,
                Some(secret.id()),
            )
            .await?;
        }

        let variant = SchemaVariant::get_by_id(ctx, sv_id).await?;

        ctx.write_audit_log(
            AuditLogKind::CreateComponent {
                name: credential_name.clone(),
                component_id: component.id(),
                schema_variant_id: sv_id,
                schema_variant_name: variant.display_name().to_owned(),
            },
            credential_name.clone(),
        )
        .await?;

        component
    };

    // Create the region component
    let region = request.aws_region;
    let region_component = {
        let schema = Schema::get_or_install_by_name(ctx, "Region").await?;
        let sv_id = SchemaVariant::default_id_for_schema(ctx, schema.id()).await?;
        let component = Component::new(ctx, &region, sv_id, view_id).await?;

        let variant = SchemaVariant::get_by_id(ctx, sv_id).await?;

        ctx.write_audit_log(
            AuditLogKind::CreateComponent {
                name: region.clone(),
                component_id: component.id(),
                schema_variant_id: sv_id,
                schema_variant_name: variant.display_name().to_owned(),
            },
            region.clone(),
        )
        .await?;

        component
    };

    // Set the region value and subscribe the region's credential to the cred component
    {
        let sources = serde_json::from_value(serde_json::json!({
            "/domain/region" : region,
            "/secrets/credential": {
              "$source": { "component": credential_component.id().to_string(), "path": "/secrets/AWS Credential" }
            },
        }))?;

        dal::update_attributes(ctx, region_component.id(), sources).await?;
    }

    // Commit so the components are created and the DVU is triggered
    ctx.commit().await?;

    // Await the DVU before applying the changeset
    ChangeSet::wait_for_dvu(ctx, true).await?;

    let change_set = ctx.change_set()?.clone();
    let old_status = change_set.status;

    ChangeSet::prepare_for_force_apply(ctx).await?;
    ctx.write_audit_log(
        AuditLogKind::ApproveChangeSetApply {
            from_status: old_status.into(),
        },
        change_set.name.clone(),
    )
    .await?;

    ChangeSet::apply_to_base_change_set(ctx).await?;

    ctx.write_audit_log(AuditLogKind::ApplyChangeSet, change_set.name.clone())
        .await?;

    Ok(())
}
