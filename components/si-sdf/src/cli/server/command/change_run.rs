use crate::cli::server::{CommandContext, ServerResult};
use crate::data::{NatsConn, PgPool, PgTxn};
use crate::models::{
    ops::OpEntityAction, ops::OpEntitySet, ChangeSet, EditSession, Entity, Event, Node, NodeError,
};
use crate::veritech::Veritech;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChangeRunError {
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("pg error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
}

pub type ChangeRunResult<T> = Result<T, ChangeRunError>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntityActionCommand {
    pub entity_id: String,
    pub system_id: String,
    pub action: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntitySetCommand {
    entity_id: String,
    path: Vec<String>,
    value: serde_json::Value,
}

impl EntitySetCommand {
    pub fn new(entity_id: impl Into<String>, path: Vec<String>, value: serde_json::Value) -> Self {
        let entity_id = entity_id.into();
        Self {
            entity_id,
            path,
            value,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeActionCommand {
    pub node_id: String,
    pub system_id: String,
    pub action: String,
}

impl NodeActionCommand {
    pub async fn into_entity_action_command(
        self,
        txn: &PgTxn<'_>,
    ) -> ChangeRunResult<EntityActionCommand> {
        let node = Node::get(&txn, self.node_id).await?;
        let entity_id = node.object_id.clone();

        Ok(EntityActionCommand {
            entity_id,
            system_id: self.system_id,
            action: self.action,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeSetCommand {
    node_id: String,
    path: Vec<String>,
    value: serde_json::Value,
}

impl NodeSetCommand {
    pub fn new(node_id: impl Into<String>, path: Vec<String>, value: serde_json::Value) -> Self {
        let node_id = node_id.into();
        Self {
            node_id,
            path,
            value,
        }
    }

    pub async fn into_entity_set_command(
        self,
        txn: &PgTxn<'_>,
    ) -> ChangeRunResult<EntitySetCommand> {
        let node = Node::get(&txn, self.node_id).await?;
        let entity_id = node.object_id.clone();

        Ok(EntitySetCommand {
            entity_id,
            path: self.path,
            value: self.value,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeChangeRun {
    action: NodeActionCommand,
    set_commands: Vec<NodeSetCommand>,
}

impl NodeChangeRun {
    pub fn new(
        node_id: impl Into<String>,
        system_id: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        let node_id = node_id.into();
        let system_id = system_id.into();
        let action = action.into();

        Self {
            action: NodeActionCommand {
                node_id,
                system_id,
                action,
            },
            set_commands: Vec::new(),
        }
    }

    pub fn add_set_command(&mut self, node_set_command: NodeSetCommand) -> &mut Self {
        self.set_commands.push(node_set_command);
        self
    }

    pub async fn into_change_run(self, txn: &PgTxn<'_>) -> ChangeRunResult<ChangeRun> {
        let action = self.action.into_entity_action_command(&txn).await?;

        let mut set_commands = Vec::with_capacity(self.set_commands.len());
        for set_command in self.set_commands.into_iter() {
            set_commands.push(set_command.into_entity_set_command(&txn).await?);
        }

        Ok(ChangeRun {
            action,
            set_commands,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRun {
    action: EntityActionCommand,
    set_commands: Vec<EntitySetCommand>,
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

    pub fn add_set_command(&mut self, entity_set_command: EntitySetCommand) -> &mut Self {
        self.set_commands.push(entity_set_command);
        self
    }

    pub async fn execute(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        ctx: &CommandContext,
    ) -> ServerResult<()> {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        let target_entity = Entity::get_any(&txn, &self.action.entity_id).await?;

        let mut root_event = Event::cli_change_run(
            &txn,
            &nats,
            &target_entity,
            &self.action.action,
            &self.action.system_id,
            None,
        )
        .await?;
        ctx.set_root_event(root_event.clone()).await;
        root_event.save(&txn, &nats).await?;

        let mut change_set = ChangeSet::new(
            &txn,
            &nats,
            None,
            target_entity.si_storable.workspace_id.clone(),
        )
        .await?;
        ctx.add_tracking_id(change_set.id.clone()).await;

        let edit_session = EditSession::new(
            &txn,
            &nats,
            None,
            change_set.id.clone(),
            change_set.si_storable.workspace_id.clone(),
        )
        .await?;
        ctx.add_tracking_id(edit_session.id.clone()).await;

        for set_command in self.set_commands.iter() {
            let op = OpEntitySet::new(
                &txn,
                &nats,
                set_command.entity_id.clone(),
                set_command.path.clone(),
                set_command.value.clone(),
                None,
                target_entity.si_storable.workspace_id.clone(),
                change_set.id.clone(),
                edit_session.id.clone(),
            )
            .await?;
            ctx.add_tracking_id(op.id.clone()).await;
        }

        let act = OpEntityAction::new(
            &txn,
            &nats,
            self.action.entity_id.clone(),
            self.action.action.clone(),
            self.action.system_id.clone(),
            target_entity.si_storable.workspace_id.clone(),
            change_set.id.clone(),
            edit_session.id.clone(),
        )
        .await?;
        ctx.add_tracking_id(act.id.clone()).await;

        change_set
            .execute(&txn, &nats, &veritech, false, Some(root_event.id.as_ref()))
            .await?;

        txn.commit().await?;
        nats.commit().await?;

        Ok(())
    }
}
