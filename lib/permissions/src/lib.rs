use std::result;

use si_data_spicedb::{
    Relationship,
    Relationships,
    SpiceDBObject,
    SpiceDbClient,
    SpiceDbError,
    ZedToken,
};
use si_events::{
    UserPk,
    WorkspacePk,
};
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Builder must contain object, permission, and subject.")]
    PermissionBuilder,
    #[error(
        "All of the following fields are required for this call: {:?}",
        required_fields
    )]
    RelationBuilder { required_fields: Vec<String> },
    #[error("spicedb client error: {0}")]
    SpiceDb(#[from] Box<SpiceDbError>),
}

impl From<SpiceDbError> for Error {
    fn from(value: SpiceDbError) -> Self {
        Box::new(value).into()
    }
}

type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, strum::Display, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum ObjectType {
    User,
    Workspace,
}

#[derive(Clone, Copy, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    Approve,
    Manage,
}

#[derive(Clone, Copy, strum::Display, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum Relation {
    Approver,
    Owner,
}

/// RelationBuilder allows defining a relationship in SpiceDb.
/// Relationships work by saying object -> relation -> subject,
/// so `workspace 123` has `approver` of `user 456`.
/// The object, relation, and subject must be set.
///
/// # Examples
/// ```no_run
/// RelationBuilder::new()
///     .object(ObjectType::Workspace, workspace_id.clone())
///     .relation(Relation::Approver)
///     .subject(ObjectType::User, user_id.clone())
///     .create(client))
///     .await?;
/// ```
pub struct RelationBuilder {
    object: Option<SpiceDBObject>,
    relation: Option<Relation>,
    subject: Option<SpiceDBObject>,
    zed_token: Option<ZedToken>,
}

impl RelationBuilder {
    pub fn new() -> Self {
        Self {
            object: None,
            relation: None,
            subject: None,
            zed_token: None,
        }
    }

    pub fn object(mut self, object_type: ObjectType, id: impl ToString) -> Self {
        self.object = Some(SpiceDBObject::new(object_type, id));
        self
    }

    pub fn workspace_object(self, id: WorkspacePk) -> Self {
        self.object(ObjectType::Workspace, id)
    }

    pub fn relation(mut self, relation: Relation) -> Self {
        self.relation = Some(relation);
        self
    }

    pub fn subject(mut self, object_type: ObjectType, id: impl ToString) -> Self {
        self.subject = Some(SpiceDBObject::new(object_type, id));
        self
    }

    pub fn user_subject(self, id: UserPk) -> Self {
        self.subject(ObjectType::User, id)
    }

    pub fn zed_token(mut self, token: ZedToken) -> Self {
        self.zed_token = Some(token.clone());
        self
    }

    /// Creates a new relationship in SpiceDb
    pub async fn create(&self, client: &mut SpiceDbClient) -> Result<Option<ZedToken>> {
        match self.check() {
            Ok(relationship) => client
                .create_relationships(vec![relationship])
                .await
                .map_err(Into::into),
            Err(err) => Err(err),
        }
    }

    /// Deletes an existing relationship in SpiceDb
    pub async fn delete(&self, client: &mut SpiceDbClient) -> Result<Option<ZedToken>> {
        match self.check() {
            Ok(relationship) => client
                .delete_relationships(vec![relationship])
                .await
                .map_err(Into::into),
            Err(err) => Err(err),
        }
    }

    /// Reads existing relations in SpiceDb for a given object and relation
    pub async fn read(&self, client: &mut SpiceDbClient) -> Result<Relationships> {
        match (self.object.clone(), self.relation) {
            (Some(object), Some(relation)) => client
                .read_relationship(Relationship::new(
                    object,
                    relation,
                    SpiceDBObject::empty(),
                    self.zed_token.clone(),
                ))
                .await
                .map_err(Into::into),
            _ => Err(Error::RelationBuilder {
                required_fields: vec!["object".to_string(), "relation".to_string()],
            }),
        }
    }

    fn check(&self) -> Result<Relationship> {
        match (self.object.clone(), self.relation, self.subject.clone()) {
            (Some(object), Some(relation), Some(subject)) => Ok(Relationship::new(
                object,
                relation,
                subject,
                self.zed_token.clone(),
            )),
            _ => Err(Error::RelationBuilder {
                required_fields: vec![
                    "object".to_string(),
                    "relation".to_string(),
                    "subject".to_string(),
                ],
            }),
        }
    }
}

impl Default for RelationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// PermissionBuilder allows checking a permission in SpiceDb.
/// Permissions checks work by saying object -> permission -> subject,
/// so `workspace 123` allows `approve` for `user 456`.
/// The object, permission, and subject must be set.
///
/// # Examples
/// ```no_run
/// if (PermissionBuilder::new()
///     .object(ObjectType::Workspace, workspace_id.clone())
///     .permission(Permission::Approve)
///     .subject(ObjectType::User, user_id.clone())
///     .has_permission(client)
///     .await?) { do_thing() }
/// ```
pub struct PermissionBuilder {
    object: Option<SpiceDBObject>,
    permission: Option<Permission>,
    subject: Option<SpiceDBObject>,
    zed_token: Option<ZedToken>,
}

impl PermissionBuilder {
    pub fn new() -> Self {
        Self {
            object: None,
            permission: None,
            subject: None,
            zed_token: None,
        }
    }

    pub fn object(mut self, object_type: ObjectType, id: impl ToString) -> Self {
        self.object = Some(SpiceDBObject::new(object_type, id));
        self
    }

    pub fn workspace_object(self, id: WorkspacePk) -> Self {
        self.object(ObjectType::Workspace, id)
    }

    pub fn permission(mut self, permission: Permission) -> Self {
        self.permission = Some(permission);
        self
    }

    pub fn subject(mut self, object_type: ObjectType, id: impl ToString) -> Self {
        self.subject = Some(SpiceDBObject::new(object_type, id));
        self
    }

    pub fn user_subject(self, id: UserPk) -> Self {
        self.subject(ObjectType::User, id)
    }

    pub fn zed_token(mut self, token: ZedToken) -> Self {
        self.zed_token = Some(token.clone());
        self
    }

    /// Checks if the given subject has the given permission in the given object
    pub async fn has_permission(&self, client: &mut SpiceDbClient) -> Result<bool> {
        match self.check_has_permission() {
            Ok(perms) => client.check_permissions(perms).await.map_err(Into::into),
            Err(err) => Err(err),
        }
    }

    fn check_has_permission(&self) -> Result<si_data_spicedb::Permission> {
        match (self.object.clone(), self.permission, self.subject.clone()) {
            (Some(object), Some(permission), Some(subject)) => {
                Ok(si_data_spicedb::Permission::new(
                    object,
                    permission,
                    subject,
                    self.zed_token.clone(),
                ))
            }
            _ => Err(Error::PermissionBuilder),
        }
    }
}

impl Default for PermissionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
