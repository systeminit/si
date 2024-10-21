use std::ops;

use spicedb_client::{
    builder::{ReadRelationshipsRequestBuilder, RelationshipBuilder, RelationshipFilterBuilder},
    types::ConsistencyRequirement,
};
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

#[derive(Clone, Debug)]
pub struct PermissionsObject {
    r#type: String,
    id: String,
}

impl PermissionsObject {
    pub fn new(r#type: impl ToString, id: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            r#type: r#type.to_string(),
        }
    }

    pub fn empty() -> Self {
        Self {
            id: "".to_string(),
            r#type: "".to_string(),
        }
    }

    pub fn r#type(&self) -> &str {
        &self.r#type
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

pub type Relationships = Vec<Relationship>;

#[derive(Clone, Debug)]
pub struct Relationship {
    object: PermissionsObject,
    relation: String,
    subject: PermissionsObject,
    zed_token: Option<ZedToken>,
}

impl Relationship {
    pub fn new(
        object: PermissionsObject,
        relation: impl ToString,
        subject: PermissionsObject,
        zed_token: Option<ZedToken>,
    ) -> Self {
        Self {
            object,
            relation: relation.to_string(),
            subject,
            zed_token,
        }
    }

    pub fn into_read_request(self) -> v1::ReadRelationshipsRequest {
        let mut builder = <v1::ReadRelationshipsRequest as RelationshipFilterBuilder>::new();
        let object = self.object();
        let relation = self.relation();
        let requirement = match self.zed_token() {
            Some(z) => ConsistencyRequirement::AtLeastAsFresh(v1::ZedToken {
                token: z.to_string(),
            }),
            None => ConsistencyRequirement::MinimizeLatency(true),
        };

        builder
            .resource_type(object.r#type())
            .resource_id(object.id())
            .relation(relation)
            .consistency(requirement);

        builder
    }

    pub(crate) fn into_relationship_update(
        self,
        operation: v1::relationship_update::Operation,
    ) -> v1::RelationshipUpdate {
        let object = self.object();
        let relation = self.relation();
        let subject = self.subject();
        spicedb_grpc::authzed::api::v1::RelationshipUpdate {
            operation: operation.into(),
            relationship: Some(v1::Relationship::new(
                object.r#type(),
                object.id(),
                relation,
                subject.r#type(),
                subject.id(),
            )),
        }
    }

    pub fn object(&self) -> &PermissionsObject {
        &self.object
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn subject(&self) -> &PermissionsObject {
        &self.subject
    }

    pub fn set_zed_token(&mut self, zed_token: Option<ZedToken>) {
        self.zed_token = zed_token;
    }

    pub fn zed_token(&self) -> Option<&ZedToken> {
        self.zed_token.as_ref()
    }
}

impl From<v1::Relationship> for Relationship {
    fn from(value: v1::Relationship) -> Self {
        let (obj_type, obj_id) = match value.resource {
            Some(o) => (o.object_type, o.object_id),
            None => (String::new(), String::new()),
        };

        let (sub_type, sub_id) = match value.subject {
            Some(s) => {
                if let Some(obj) = s.object {
                    (obj.object_type, obj.object_id)
                } else {
                    (String::new(), String::new())
                }
            }
            None => (String::new(), String::new()),
        };
        Relationship::new(
            PermissionsObject::new(obj_type, obj_id),
            value.relation,
            PermissionsObject::new(sub_type, sub_id),
            None,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Permission {
    resource: PermissionsObject,
    permission: String,
    subject: PermissionsObject,
    zed_token: Option<ZedToken>,
}

impl Permission {
    pub fn new(
        resource: PermissionsObject,
        permission: impl ToString,
        subject: PermissionsObject,
        zed_token: Option<ZedToken>,
    ) -> Self {
        Self {
            resource,
            permission: permission.to_string(),
            subject,
            zed_token,
        }
    }

    pub(crate) fn into_request(self) -> v1::CheckPermissionRequest {
        let requirement = match self.zed_token {
            Some(z) => ConsistencyRequirement::AtLeastAsFresh(v1::ZedToken { token: z.0 }),
            None => ConsistencyRequirement::MinimizeLatency(true),
        };
        v1::CheckPermissionRequest {
            consistency: Some(v1::Consistency {
                requirement: Some(requirement),
            }),
            resource: Some(ObjectReference {
                object_type: self.resource.r#type,
                object_id: self.resource.id,
            }),
            permission: self.permission,
            subject: Some(SubjectReference {
                object: Some(ObjectReference {
                    object_type: self.subject.r#type,
                    object_id: self.subject.id,
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

    pub fn set_zed_token(&mut self, zed_token: Option<ZedToken>) {
        self.zed_token = zed_token;
    }
}
