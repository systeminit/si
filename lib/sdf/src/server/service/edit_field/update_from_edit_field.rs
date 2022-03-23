use axum::Json;
use dal::{
    edit_field::{EditFieldAble, EditFieldBaggage, EditFieldObjectKind},
    schema::{self, SchemaVariant},
    socket::Socket,
    Component, HistoryActor, Prop, QualificationCheck, Schema, Visibility, WorkspaceId,
    WriteTenancy,
};
use serde::{Deserialize, Serialize};

use super::{EditFieldError, EditFieldResult};
use crate::server::extract::{Authorization, EncryptionKey, NatsTxn, PgRwTxn, Veritech};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFromEditFieldRequest {
    pub object_kind: EditFieldObjectKind,
    pub object_id: i64,
    pub edit_field_id: String,
    pub value: Option<serde_json::Value>,
    pub baggage: Option<EditFieldBaggage>,
    pub workspace_id: Option<WorkspaceId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFromEditFieldResponse {
    pub success: bool,
}

pub async fn update_from_edit_field(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Veritech(veritech): Veritech,
    EncryptionKey(encryption_key): EncryptionKey,
    Authorization(claim): Authorization,
    Json(request): Json<UpdateFromEditFieldRequest>,
) -> EditFieldResult<Json<UpdateFromEditFieldResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;
    let write_tenancy = match request.workspace_id {
        Some(workspace_id) => WriteTenancy::new_workspace(workspace_id),
        None => WriteTenancy::new_billing_account(claim.billing_account_id),
    };

    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);

    match request.object_kind {
        EditFieldObjectKind::Component => {
            Component::update_from_edit_field(
                &txn,
                &nats,
                veritech,
                &encryption_key,
                &write_tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
        EditFieldObjectKind::ComponentProp => {
            // Eventually, this won't be infallible. -- Adam
            #[allow(clippy::infallible_destructuring_match)]
            let _baggage = request.baggage.ok_or(EditFieldError::MissingBaggage)?;
            todo!()
            // FIXME(nick): need to figure out prop id;
            //
            // Component::update_prop_from_edit_field(
            //     &txn,
            //     &nats,
            //     veritech,
            //     &encryption_key,
            //     &write_tenancy,
            //     &request.visibility,
            //     &history_actor,
            //     request.object_id.into(),
            //     baggage.attribute_context.prop_id(),
            //     request.edit_field_id,
            //     request.value,
            //     None, // TODO: Eventually, pass the key! -- Adam
            // )
            // .await?
        }
        EditFieldObjectKind::Prop => {
            Prop::update_from_edit_field(
                &txn,
                &nats,
                veritech,
                &encryption_key,
                &write_tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
        EditFieldObjectKind::QualificationCheck => {
            QualificationCheck::update_from_edit_field(
                &txn,
                &nats,
                veritech,
                &encryption_key,
                &write_tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
        EditFieldObjectKind::Schema => {
            Schema::update_from_edit_field(
                &txn,
                &nats,
                veritech,
                &encryption_key,
                &write_tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
        EditFieldObjectKind::SchemaUiMenu => {
            schema::UiMenu::update_from_edit_field(
                &txn,
                &nats,
                veritech,
                &encryption_key,
                &write_tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
        EditFieldObjectKind::SchemaVariant => {
            SchemaVariant::update_from_edit_field(
                &txn,
                &nats,
                veritech,
                &encryption_key,
                &write_tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
        EditFieldObjectKind::Socket => {
            Socket::update_from_edit_field(
                &txn,
                &nats,
                veritech,
                &encryption_key,
                &write_tenancy,
                &request.visibility,
                &history_actor,
                request.object_id.into(),
                request.edit_field_id,
                request.value,
            )
            .await?
        }
    };

    txn.commit().await?;
    nats.commit().await?;

    Ok(Json(UpdateFromEditFieldResponse { success: true }))
}
