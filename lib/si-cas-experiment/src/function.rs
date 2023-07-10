use ulid::Ulid;

use crate::{OriginId, ContentHash};

pub type FunctionId = Ulid;
pub type FunctionPk = Ulid;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub id: FunctionId,
    pub pk: FunctionPk,
    pub name: String,
    pub origin_id: OriginId,
    pub content_hash: ContentHash,
}

impl Function {
    pub fn new(name: impl Into<String>) -> Function {
        let name = name.into();
        let id = FunctionId::new();
        let pk = FunctionPk::new();
        let origin_id = OriginId::new();
        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        hasher.update(origin_id.to_string().as_bytes());
        let content_hash = hasher.finalize();

        Function {
            name,
            id,
            pk,
            origin_id,
            content_hash,
        }
    }
    pub fn create_copy(func:Function) -> Function{
        let name = func.name;
        let id = FunctionId::new();
        let pk = FunctionPk::new();
        let origin_id = func.origin_id;
        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        hasher.update(origin_id.to_string().as_bytes());
        let content_hash = hasher.finalize();

        Function {
            name,
            id,
            pk,
            origin_id,
            content_hash
        }
    }

    pub fn id(&self) -> FunctionId {
        self.id
    }

    pub fn pk(&self) -> FunctionPk {
        self.pk
    }
}

