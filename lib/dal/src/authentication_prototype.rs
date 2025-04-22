use serde::{
    Deserialize,
    Serialize,
};
pub use si_id::AuthenticationPrototypeId;

use crate::{
    FuncId,
    SchemaVariantId,
};

// TODO(nick): remove this once import can just create the edge.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationPrototype {
    pub id: AuthenticationPrototypeId,
    pub func_id: FuncId,
    pub schema_variant_id: SchemaVariantId,
}
