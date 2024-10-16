use si_data_spicedb::{PermissionsObject, Relationship, SpiceDbClient, SpiceDbError};
use std::result;
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Builder must contain object, permission, and subject.")]
    PermissionBuilder,
    #[error("Builder must contain object, relation, and subject.")]
    RelationBuilder,
    #[error("spicedb client error: {0}")]
    SpiceDb(#[from] SpiceDbError),
}

type Result<T> = result::Result<T, Error>;

#[derive(strum::Display, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum ObjectType {
    User,
    Workspace,
}

#[derive(Clone, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    Approve,
}

#[derive(Clone, strum::Display, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum Relation {
    Approver,
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
    object: Option<PermissionsObject>,
    relation: Option<Relation>,
    subject: Option<PermissionsObject>,
}

impl RelationBuilder {
    pub fn new() -> Self {
        Self {
            object: None,
            relation: None,
            subject: None,
        }
    }

    pub fn object(mut self, object_type: ObjectType, id: impl ToString) -> Self {
        self.object = Some(PermissionsObject::new(object_type, id));
        self
    }

    pub fn relation(mut self, relation: Relation) -> Self {
        self.relation = Some(relation);
        self
    }

    pub fn subject(mut self, object_type: ObjectType, id: impl ToString) -> Self {
        self.subject = Some(PermissionsObject::new(object_type, id));
        self
    }

    /// Creates a new relationship in SpiceDb
    pub async fn create(&self, mut client: SpiceDbClient) -> Result<()> {
        match self.check() {
            Ok(relationship) => {
                client
                    .create_relationships(vec![relationship])
                    .await
                    .map_err(Error::SpiceDb)?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Deletes an existing relationship in SpiceDb
    pub async fn delete(&self, mut client: SpiceDbClient) -> Result<()> {
        match self.check() {
            Ok(relationship) => {
                client
                    .delete_relationships(vec![relationship])
                    .await
                    .map_err(Error::SpiceDb)?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn check(&self) -> Result<Relationship> {
        match (
            self.object.clone(),
            self.relation.clone(),
            self.subject.clone(),
        ) {
            (Some(object), Some(relation), Some(subject)) => {
                Ok(Relationship::new(object, relation, subject))
            }
            _ => Err(Error::RelationBuilder),
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
    object: Option<PermissionsObject>,
    permission: Option<Permission>,
    subject: Option<PermissionsObject>,
}

impl PermissionBuilder {
    pub fn new() -> Self {
        Self {
            object: None,
            permission: None,
            subject: None,
        }
    }

    pub fn object(mut self, object_type: ObjectType, id: impl ToString) -> Self {
        self.object = Some(PermissionsObject::new(object_type, id));
        self
    }

    pub fn permission(mut self, permission: Permission) -> Self {
        self.permission = Some(permission);
        self
    }

    pub fn subject(mut self, object_type: ObjectType, id: impl ToString) -> Self {
        self.subject = Some(PermissionsObject::new(object_type, id));
        self
    }

    /// Checks if the given subject has the given permission in the given object
    pub async fn has_permission(&self, mut client: SpiceDbClient) -> Result<bool> {
        match self.check() {
            Ok(perms) => Ok(client
                .check_permissions(perms)
                .await
                .map_err(Error::SpiceDb)?),
            Err(err) => Err(err),
        }
    }

    fn check(&self) -> Result<si_data_spicedb::Permission> {
        match (
            self.object.clone(),
            self.permission.clone(),
            self.subject.clone(),
        ) {
            (Some(object), Some(permission), Some(subject)) => Ok(
                si_data_spicedb::Permission::new(object, permission, subject),
            ),
            _ => Err(Error::PermissionBuilder),
        }
    }
}

impl Default for PermissionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
