use serde::{Deserialize, Serialize};

use crate::cli::server::{CommandContext, ServerResult};
use crate::data::{Connection, Db};
use crate::models::{ops::OpEntityAction, ops::OpEntitySet, ChangeSet, EditSession, Entity, Event};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntityActionCommand {
    pub entity_id: String,
    pub system_id: String,
    pub action: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntitySetCommand {
    pub entity_id: String,
    pub path: Vec<String>,
    pub value: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRun {
    pub action: EntityActionCommand,
    pub set_commands: Vec<EntitySetCommand>,
}

impl ChangeRun {
    pub fn new(
        entity_id: impl Into<String>,
        system_id: impl Into<String>,
        action: impl Into<String>,
    ) -> ChangeRun {
        let entity_id = entity_id.into();
        let system_id = system_id.into();
        let action = action.into();
        ChangeRun {
            action: EntityActionCommand {
                entity_id,
                system_id,
                action,
            },
            set_commands: Vec::new(),
        }
    }

    pub async fn execute(
        &self,
        db: &Db,
        nats: &Connection,
        ctx: &CommandContext,
    ) -> ServerResult<()> {
        let target_entity = Entity::get_any(&db, &self.action.entity_id).await?;

        let root_event = Event::cli_change_run(
            &db,
            &nats,
            &target_entity,
            &self.action.action,
            &self.action.system_id,
            None,
        )
        .await?;
        ctx.set_root_event(root_event.clone()).await;
        root_event.save(db, nats).await?;

        let mut change_set = ChangeSet::new(
            &db,
            &nats,
            None,
            target_entity.si_storable.billing_account_id.clone(),
            target_entity.si_storable.organization_id.clone(),
            target_entity.si_storable.workspace_id.clone(),
            String::from(ctx.user_id.as_ref()),
        )
        .await?;
        ctx.add_tracking_id(change_set.id.clone()).await;

        let edit_session = EditSession::new(
            &db,
            &nats,
            None,
            change_set.id.clone(),
            change_set.si_storable.billing_account_id.clone(),
            change_set.si_storable.organization_id.clone(),
            change_set.si_storable.workspace_id.clone(),
            String::from(ctx.user_id.as_ref()),
        )
        .await?;
        ctx.add_tracking_id(edit_session.id.clone()).await;

        for set_command in self.set_commands.iter() {
            let op = OpEntitySet::new(
                &db,
                &nats,
                set_command.entity_id.clone(),
                set_command.path.clone(),
                set_command.value.clone(),
                None,
                target_entity.si_storable.billing_account_id.clone(),
                target_entity.si_storable.organization_id.clone(),
                target_entity.si_storable.workspace_id.clone(),
                change_set.id.clone(),
                edit_session.id.clone(),
                String::from(ctx.user_id.as_ref()),
            )
            .await?;
            ctx.add_tracking_id(op.id.clone()).await;
        }

        let act = OpEntityAction::new(
            db.clone(),
            nats.clone(),
            self.action.entity_id.clone(),
            self.action.action.clone(),
            self.action.system_id.clone(),
            target_entity.si_storable.billing_account_id.clone(),
            target_entity.si_storable.organization_id.clone(),
            target_entity.si_storable.workspace_id.clone(),
            change_set.id.clone(),
            edit_session.id.clone(),
            String::from(ctx.user_id.as_ref()),
        )
        .await?;
        ctx.add_tracking_id(act.id.clone()).await;

        change_set
            .execute(&db, &nats, false, Some(root_event.id.as_ref()))
            .await?;

        Ok(())
    }
}
