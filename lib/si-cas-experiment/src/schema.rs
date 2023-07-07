use ulid::Ulid;

use crate::{ContentHash, OriginId};

pub type SchemaId = Ulid;
pub type SchemaPk = Ulid;

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub id: SchemaId,
    pub pk: SchemaPk,
    pub name: String,
    pub origin_id: OriginId,
    pub content_hash: ContentHash,
}

impl Schema {
    pub fn new(name: impl Into<String>) -> Schema {
        let name = name.into();
        let id = SchemaId::new();
        let pk = SchemaPk::new();
        let origin_id = OriginId::new();
        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        hasher.update(origin_id.to_string().as_bytes());
        let content_hash = hasher.finalize();

        Schema {
            name,
            id,
            pk,
            origin_id,
            content_hash,
        }
    }

    pub fn id(&self) -> SchemaId {
        self.id
    }

    pub fn pk(&self) -> SchemaPk {
        self.pk
    }
}
