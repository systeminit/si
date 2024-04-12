use serde::{Deserialize, Serialize};
use si_pkg::{AttrFuncInputSpec, SiPropFuncSpec, SiPropFuncSpecKind};

use crate::prop::PropPath;

/// The definition for the source of the information for a prop or a socket in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ValueFrom {
    InputSocket { socket_name: String },
    OutputSocket { socket_name: String },
    Prop { prop_path: Vec<String> },
}

impl ValueFrom {
    pub fn to_spec(&self) -> AttrFuncInputSpec {
        match self {
            ValueFrom::InputSocket { socket_name } => AttrFuncInputSpec::InputSocket {
                name: "identity".to_string(),
                socket_name: socket_name.to_owned(),
                unique_id: None,
                deleted: false,
            },
            ValueFrom::Prop { prop_path } => AttrFuncInputSpec::Prop {
                name: "identity".to_string(),
                prop_path: PropPath::new(prop_path).into(),
                unique_id: None,
                deleted: false,
            },
            ValueFrom::OutputSocket { socket_name } => AttrFuncInputSpec::OutputSocket {
                name: "identity".to_string(),
                socket_name: socket_name.to_owned(),
                unique_id: None,
                deleted: false,
            },
        }
    }
}

/// The definition for the source of the data for prop under "/root/"si" in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiPropValueFrom {
    kind: SiPropFuncSpecKind,
    value_from: ValueFrom,
}

impl SiPropValueFrom {
    pub fn to_spec(&self, identity_func_unique_id: &str) -> SiPropFuncSpec {
        SiPropFuncSpec {
            kind: self.kind,
            func_unique_id: identity_func_unique_id.to_owned(),
            inputs: vec![self.value_from.to_spec()],
            unique_id: None,
            deleted: false,
        }
    }
}
