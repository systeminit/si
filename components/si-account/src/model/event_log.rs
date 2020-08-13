use chrono::Utc;

pub use crate::protobuf::{
    EventLog, EventLogGetReply, EventLogGetRequest, EventLogLevel, EventLogListReply,
    EventLogListRequest, EventLogPayload, EventLogSiProperties,
};
use si_data::{DataError, Db, ListResult, Result};

use tracing::warn;

impl EventLog {
    pub async fn entity_create(
        db: &Db,
        created_by_user_id: &str,
        entity: &impl serde::ser::Serialize,
    ) -> Result<()> {
        let entity_json: serde_json::Value = serde_json::to_value(entity)?;
        let billing_account_id = entity_json["siProperties"]["billingAccountId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.billingAccountId".into(),
            ))?;
        let workspace_id = entity_json["siProperties"]["workspaceId"]
            .as_str()
            .ok_or(DataError::RequiredField("siProperties.workspaceId".into()))?;
        let organization_id = entity_json["siProperties"]["organizationId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.organizationId".into(),
            ))?;
        let change_set_id = entity_json["siStorable"]["changeSetId"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.changeSetId".into()))?;

        let type_name = entity_json["siStorable"]["typeName"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.typeName".into()))?;
        let name = entity_json["name"]
            .as_str()
            .ok_or(DataError::RequiredField("name".into()))?;
        let id = entity_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let message = format!("created {} named {}", type_name, name);

        let payload_data = serde_json::to_string(entity)?;

        let timestamp = Utc::now();

        EventLog::create(
            db,
            Some("Entity Created".into()),
            Some("Entity Created".into()),
            Some(EventLogLevel::Info),
            Some(EventLogSiProperties {
                billing_account_id: Some(billing_account_id.into()),
                workspace_id: Some(workspace_id.into()),
                organization_id: Some(organization_id.into()),
            }),
            Some(message),
            Some(EventLogPayload {
                kind: Some("entity".into()),
                data: Some(payload_data),
            }),
            vec![
                billing_account_id.into(),
                workspace_id.into(),
                organization_id.into(),
                change_set_id.into(),
                id.into(),
            ],
            Some(timestamp.to_string()),
            Some(created_by_user_id.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn entity_update(
        db: &Db,
        created_by_user_id: &str,
        entity: &impl serde::ser::Serialize,
    ) -> Result<()> {
        let entity_json: serde_json::Value = serde_json::to_value(entity)?;
        let billing_account_id = entity_json["siProperties"]["billingAccountId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.billingAccountId".into(),
            ))?;
        let workspace_id = entity_json["siProperties"]["workspaceId"]
            .as_str()
            .ok_or(DataError::RequiredField("siProperties.workspaceId".into()))?;
        let organization_id = entity_json["siProperties"]["organizationId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.organizationId".into(),
            ))?;
        let change_set_id = entity_json["siStorable"]["changeSetId"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.changeSetId".into()))?;

        let type_name = entity_json["siStorable"]["typeName"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.typeName".into()))?;
        let name = entity_json["name"]
            .as_str()
            .ok_or(DataError::RequiredField("name".into()))?;
        let id = entity_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let message = format!("updated {} named {}", type_name, name);

        let payload_data = serde_json::to_string(entity)?;

        let timestamp = Utc::now();

        EventLog::create(
            db,
            Some("Entity Updated".into()),
            Some("Entity Updated".into()),
            Some(EventLogLevel::Info),
            Some(EventLogSiProperties {
                billing_account_id: Some(billing_account_id.into()),
                workspace_id: Some(workspace_id.into()),
                organization_id: Some(organization_id.into()),
            }),
            Some(message),
            Some(EventLogPayload {
                kind: Some("entity".into()),
                data: Some(payload_data),
            }),
            vec![
                billing_account_id.into(),
                workspace_id.into(),
                organization_id.into(),
                change_set_id.into(),
                id.into(),
            ],
            Some(timestamp.to_string()),
            Some(created_by_user_id.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn entity_delete(
        db: &Db,
        created_by_user_id: &str,
        entity: &impl serde::ser::Serialize,
    ) -> Result<()> {
        let entity_json: serde_json::Value = serde_json::to_value(entity)?;
        let billing_account_id = entity_json["siProperties"]["billingAccountId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.billingAccountId".into(),
            ))?;
        let workspace_id = entity_json["siProperties"]["workspaceId"]
            .as_str()
            .ok_or(DataError::RequiredField("siProperties.workspaceId".into()))?;
        let organization_id = entity_json["siProperties"]["organizationId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.organizationId".into(),
            ))?;
        let change_set_id = entity_json["siStorable"]["changeSetId"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.changeSetId".into()))?;

        let type_name = entity_json["siStorable"]["typeName"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.typeName".into()))?;
        let name = entity_json["name"]
            .as_str()
            .ok_or(DataError::RequiredField("name".into()))?;
        let id = entity_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let message = format!("deleted {} named {}", type_name, name);

        let payload_data = serde_json::to_string(entity)?;

        let timestamp = Utc::now();

        EventLog::create(
            db,
            Some("Entity Deleted".into()),
            Some("Entity Deleted".into()),
            Some(EventLogLevel::Info),
            Some(EventLogSiProperties {
                billing_account_id: Some(billing_account_id.into()),
                workspace_id: Some(workspace_id.into()),
                organization_id: Some(organization_id.into()),
            }),
            Some(message),
            Some(EventLogPayload {
                kind: Some("entity".into()),
                data: Some(payload_data),
            }),
            vec![
                billing_account_id.into(),
                workspace_id.into(),
                organization_id.into(),
                change_set_id.into(),
                id.into(),
            ],
            Some(timestamp.to_string()),
            Some(created_by_user_id.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn change_set_entry_execute_end(
        db: &Db,
        created_by_user_id: &str,
        change_set_entry_json: &serde_json::Value,
    ) -> Result<()> {
        let billing_account_id = change_set_entry_json["siProperties"]["billingAccountId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.billingAccountId".into(),
            ))?;
        let workspace_id = change_set_entry_json["siProperties"]["workspaceId"]
            .as_str()
            .ok_or(DataError::RequiredField("siProperties.workspaceId".into()))?;
        let organization_id = change_set_entry_json["siProperties"]["organizationId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.organizationId".into(),
            ))?;
        let type_name = change_set_entry_json["siStorable"]["typeName"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.typeName".into()))?;

        let message = if type_name.ends_with("entity_event") {
            let name = change_set_entry_json["inputEntity"]["name"]
                .as_str()
                .ok_or(DataError::RequiredField("inputEntity.name".into()))?;
            let action = change_set_entry_json["actionName"]
                .as_str()
                .ok_or(DataError::RequiredField("actionName".into()))?;
            let type_name = change_set_entry_json["inputEntity"]["siStorable"]["typeName"]
                .as_str()
                .ok_or(DataError::RequiredField(
                    "inputEntity.siStorable.typeName".into(),
                ))?;
            format!(
                "finished executing change set entry action {} for {} named {}",
                action, type_name, name,
            )
        } else {
            let name = change_set_entry_json["name"]
                .as_str()
                .ok_or(DataError::RequiredField("name".into()))?;
            format!(
                "finished executing change set entry for {} named {}",
                type_name, name
            )
        };
        let id = change_set_entry_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let payload_data = change_set_entry_json.to_string();

        let timestamp = Utc::now();

        EventLog::create(
            db,
            Some("Change Set Entry Executed".into()),
            Some("Change Set Entry Executed".into()),
            Some(EventLogLevel::Info),
            Some(EventLogSiProperties {
                billing_account_id: Some(billing_account_id.into()),
                workspace_id: Some(workspace_id.into()),
                organization_id: Some(organization_id.into()),
            }),
            Some(message),
            Some(EventLogPayload {
                kind: Some("change_set_entry".into()),
                data: Some(payload_data),
            }),
            vec![
                billing_account_id.into(),
                workspace_id.into(),
                organization_id.into(),
                id.into(),
            ],
            Some(timestamp.to_string()),
            Some(created_by_user_id.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn change_set_entry_execute_start(
        db: &Db,
        created_by_user_id: &str,
        change_set_entry_json: &serde_json::Value,
    ) -> Result<()> {
        let billing_account_id = change_set_entry_json["siProperties"]["billingAccountId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.billingAccountId".into(),
            ))?;
        let workspace_id = change_set_entry_json["siProperties"]["workspaceId"]
            .as_str()
            .ok_or(DataError::RequiredField("siProperties.workspaceId".into()))?;
        let organization_id = change_set_entry_json["siProperties"]["organizationId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.organizationId".into(),
            ))?;
        let type_name = change_set_entry_json["siStorable"]["typeName"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.typeName".into()))?;

        let message = if type_name.ends_with("entity_event") {
            let name = change_set_entry_json["inputEntity"]["name"]
                .as_str()
                .ok_or(DataError::RequiredField("inputEntity.name".into()))?;
            let action = change_set_entry_json["actionName"]
                .as_str()
                .ok_or(DataError::RequiredField("actionName".into()))?;
            let type_name = change_set_entry_json["inputEntity"]["siStorable"]["typeName"]
                .as_str()
                .ok_or(DataError::RequiredField(
                    "inputEntity.siStorable.typeName".into(),
                ))?;
            format!(
                "starting to execute change set entry action {} for {} named {}",
                action, type_name, name,
            )
        } else {
            let name = change_set_entry_json["name"]
                .as_str()
                .ok_or(DataError::RequiredField("name".into()))?;
            format!(
                "starting to execute change set entry for {} named {}",
                type_name, name
            )
        };
        let id = change_set_entry_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let payload_data = change_set_entry_json.to_string();

        let timestamp = Utc::now();

        EventLog::create(
            db,
            Some("Change Set Entry Executed".into()),
            Some("Change Set Entry Executed".into()),
            Some(EventLogLevel::Info),
            Some(EventLogSiProperties {
                billing_account_id: Some(billing_account_id.into()),
                workspace_id: Some(workspace_id.into()),
                organization_id: Some(organization_id.into()),
            }),
            Some(message),
            Some(EventLogPayload {
                kind: Some("change_set_entry".into()),
                data: Some(payload_data),
            }),
            vec![
                billing_account_id.into(),
                workspace_id.into(),
                organization_id.into(),
                id.into(),
            ],
            Some(timestamp.to_string()),
            Some(created_by_user_id.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn change_set_entry_execute(
        db: &Db,
        created_by_user_id: &str,
        change_set_json: &serde_json::Value,
    ) -> Result<()> {
        let billing_account_id = change_set_json["siProperties"]["billingAccountId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.billingAccountId".into(),
            ))?;
        let workspace_id = change_set_json["siProperties"]["workspaceId"]
            .as_str()
            .ok_or(DataError::RequiredField("siProperties.workspaceId".into()))?;
        let organization_id = change_set_json["siProperties"]["organizationId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.organizationId".into(),
            ))?;

        let type_name = change_set_json["siStorable"]["typeName"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.typeName".into()))?;
        let name = change_set_json["name"]
            .as_str()
            .ok_or(DataError::RequiredField("name".into()))?;
        let id = change_set_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let message = format!("executed change set entry for {} named {}", type_name, name);

        let payload_data = change_set_json.to_string();

        let timestamp = Utc::now();

        EventLog::create(
            db,
            Some("Change Set Entry Executed".into()),
            Some("Change Set Entry Executed".into()),
            Some(EventLogLevel::Info),
            Some(EventLogSiProperties {
                billing_account_id: Some(billing_account_id.into()),
                workspace_id: Some(workspace_id.into()),
                organization_id: Some(organization_id.into()),
            }),
            Some(message),
            Some(EventLogPayload {
                kind: Some("change_set_entry".into()),
                data: Some(payload_data),
            }),
            vec![
                billing_account_id.into(),
                workspace_id.into(),
                organization_id.into(),
                id.into(),
            ],
            Some(timestamp.to_string()),
            Some(created_by_user_id.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn change_set_execute(
        db: &Db,
        created_by_user_id: &str,
        change_set: &impl serde::ser::Serialize,
    ) -> Result<()> {
        let change_set_json: serde_json::Value = serde_json::to_value(change_set)?;
        let billing_account_id = change_set_json["siProperties"]["billingAccountId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.billingAccountId".into(),
            ))?;
        let workspace_id = change_set_json["siProperties"]["workspaceId"]
            .as_str()
            .ok_or(DataError::RequiredField("siProperties.workspaceId".into()))?;
        let organization_id = change_set_json["siProperties"]["organizationId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.organizationId".into(),
            ))?;
        let change_set_id = change_set_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let type_name = change_set_json["siStorable"]["typeName"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.typeName".into()))?;
        let name = change_set_json["name"]
            .as_str()
            .ok_or(DataError::RequiredField("name".into()))?;
        let id = change_set_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let message = format!("executed change set named {}", name);

        let payload_data = serde_json::to_string(change_set)?;

        let timestamp = Utc::now();

        EventLog::create(
            db,
            Some("Change Set Executed".into()),
            Some("Change Set Executed".into()),
            Some(EventLogLevel::Info),
            Some(EventLogSiProperties {
                billing_account_id: Some(billing_account_id.into()),
                workspace_id: Some(workspace_id.into()),
                organization_id: Some(organization_id.into()),
            }),
            Some(message),
            Some(EventLogPayload {
                kind: Some("change_set".into()),
                data: Some(payload_data),
            }),
            vec![
                billing_account_id.into(),
                workspace_id.into(),
                organization_id.into(),
                change_set_id.into(),
                id.into(),
            ],
            Some(timestamp.to_string()),
            Some(created_by_user_id.to_string()),
        )
        .await?;
        Ok(())
    }

    pub async fn change_set_closed(
        db: &Db,
        created_by_user_id: &str,
        change_set: &impl serde::ser::Serialize,
    ) -> Result<()> {
        let change_set_json: serde_json::Value = serde_json::to_value(change_set)?;
        let billing_account_id = change_set_json["siProperties"]["billingAccountId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.billingAccountId".into(),
            ))?;
        let workspace_id = change_set_json["siProperties"]["workspaceId"]
            .as_str()
            .ok_or(DataError::RequiredField("siProperties.workspaceId".into()))?;
        let organization_id = change_set_json["siProperties"]["organizationId"]
            .as_str()
            .ok_or(DataError::RequiredField(
                "siProperties.organizationId".into(),
            ))?;
        let change_set_id = change_set_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let type_name = change_set_json["siStorable"]["typeName"]
            .as_str()
            .ok_or(DataError::RequiredField("siStorable.typeName".into()))?;
        let name = change_set_json["name"]
            .as_str()
            .ok_or(DataError::RequiredField("name".into()))?;
        let id = change_set_json["id"]
            .as_str()
            .ok_or(DataError::RequiredField("id".into()))?;

        let message = format!("closed change set named {}", name);

        let payload_data = serde_json::to_string(change_set)?;

        let timestamp = Utc::now();

        EventLog::create(
            db,
            Some("Change Set Closed".into()),
            Some("Change Set Closed".into()),
            Some(EventLogLevel::Info),
            Some(EventLogSiProperties {
                billing_account_id: Some(billing_account_id.into()),
                workspace_id: Some(workspace_id.into()),
                organization_id: Some(organization_id.into()),
            }),
            Some(message),
            Some(EventLogPayload {
                kind: Some("change_set".into()),
                data: Some(payload_data),
            }),
            vec![
                billing_account_id.into(),
                workspace_id.into(),
                organization_id.into(),
                change_set_id.into(),
                id.into(),
            ],
            Some(timestamp.to_string()),
            Some(created_by_user_id.to_string()),
        )
        .await?;
        Ok(())
    }
}
