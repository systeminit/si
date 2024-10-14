use std::ops;

use spicedb_client::builder::{RelationshipBuilder, RelationshipFilterBuilder};
use spicedb_grpc::authzed::api::v1::{self, ObjectReference, SubjectReference};

/// ZedToken is used to provide causality metadata between Write and Check requests.
///
/// See the authzed.api.v1.Consistency message for more information.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZedToken(String);

impl ops::Deref for ZedToken {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl From<v1::ZedToken> for ZedToken {
    fn from(value: v1::ZedToken) -> Self {
        Self(value.token)
    }
}

#[derive(Clone, Debug)]
pub struct ReadSchemaResponse {
    /// schema_text is the textual form of the current schema in the system
    pub schema_text: String,

    /// read_at is the ZedToken at which the schema was read.
    read_at: Option<ZedToken>,
}

impl ReadSchemaResponse {
    pub fn schema_text(&self) -> &str {
        &self.schema_text
    }

    pub fn read_at(&self) -> Option<&ZedToken> {
        self.read_at.as_ref()
    }
}

impl From<v1::ReadSchemaResponse> for ReadSchemaResponse {
    fn from(value: v1::ReadSchemaResponse) -> Self {
        Self {
            schema_text: value.schema_text,
            read_at: value.read_at.map(|value| value.into()),
        }
    }
}

pub type Relationships = Vec<Relationship>;
#[derive(Clone, Debug)]
pub struct Relationship(pub(crate) v1::Relationship);

impl Relationship {
    pub fn new(
        object_type: impl ToString,
        object_id: impl ToString,
        relation: impl ToString,
        subject_type: impl ToString,
        subject_id: impl ToString,
    ) -> Self {
        Self(<v1::Relationship as RelationshipBuilder>::new(
            object_type,
            object_id,
            relation,
            subject_type,
            subject_id,
        ))
    }

    pub fn into_request(self) -> v1::ReadRelationshipsRequest {
        let inner = self.0;
        let mut builder = <v1::ReadRelationshipsRequest as RelationshipFilterBuilder>::new();

        if let Some(resource) = inner.resource {
            builder.resource_type(resource.object_type);
        }

        builder.relation(inner.relation);

        builder
    }

    pub(crate) fn inner(self) -> v1::Relationship {
        self.0
    }

    pub(crate) fn into_relationship_update(
        self,
        operation: v1::relationship_update::Operation,
    ) -> v1::RelationshipUpdate {
        spicedb_grpc::authzed::api::v1::RelationshipUpdate {
            operation: operation.into(),
            relationship: Some(self.inner()),
        }
    }
}

impl From<v1::Relationship> for Relationship {
    fn from(value: v1::Relationship) -> Self {
        Relationship(value)
    }
}

#[derive(Clone, Debug)]
pub struct Permission {
    resource_type: String,
    resource_id: String,
    permission: String,
    subject_type: String,
    subject_id: String,
}

impl Permission {
    pub fn new(
        resource_type: impl ToString,
        resource_id: impl ToString,
        permission: impl ToString,
        subject_type: impl ToString,
        subject_id: impl ToString,
    ) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            permission: permission.to_string(),
            subject_type: subject_type.to_string(),
            subject_id: subject_id.to_string(),
        }
    }

    pub(crate) fn into_request(self) -> v1::CheckPermissionRequest {
        v1::CheckPermissionRequest {
            consistency: None,
            resource: Some(ObjectReference {
                object_type: self.resource_type,
                object_id: self.resource_id,
            }),
            permission: self.permission,
            subject: Some(SubjectReference {
                object: Some(ObjectReference {
                    object_type: self.subject_type,
                    object_id: self.subject_id,
                }),
                optional_relation: "".to_string(),
            }),
            context: None,
            with_tracing: false,
        }
    }

    pub(crate) fn has_permission(permissionship: i32) -> bool {
        i32::from(v1::check_permission_response::Permissionship::HasPermission) == permissionship
    }
}
