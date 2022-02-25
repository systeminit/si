use crate::{sensitive_container::ListSecrets, ComponentView, SensitiveString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ResolverFunctionComponent,
    pub code_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionComponent {
    pub data: ComponentView,
    pub parents: Vec<ComponentView>,
    // TODO: add widget data here (for example select's options)
}

impl ListSecrets for ResolverFunctionRequest {
    fn list_secrets(&self) -> Vec<SensitiveString> {
        let mut secrets = self.component.data.list_secrets();
        secrets.extend(
            self.component
                .parents
                .iter()
                .flat_map(ListSecrets::list_secrets),
        );
        secrets
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionResultSuccess {
    pub execution_id: String,
    pub data: Value,
    pub unset: bool,
    #[serde(default = "crate::timestamp")]
    pub timestamp: u64,
}
