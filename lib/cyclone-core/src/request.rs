use std::collections::HashSet;

use serde::{
    Deserialize,
    Serialize,
};
use si_crypto::SensitiveStrings;
use si_std::SensitiveString;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycloneRequest<R>
where
    R: CycloneRequestable,
{
    request: R,
    sensitive_strings: HashSet<SensitiveString>,
}

impl<R> CycloneRequest<R>
where
    R: CycloneRequestable,
{
    pub fn from_parts(request: R, sensitive_strings: SensitiveStrings) -> Self {
        Self {
            request,
            sensitive_strings: sensitive_strings.into(),
        }
    }

    pub fn websocket_path(&self) -> &str {
        self.request.websocket_path()
    }

    pub fn into_parts(self) -> (R, SensitiveStrings) {
        (self.request, self.sensitive_strings.into())
    }
}

pub trait CycloneRequestable {
    type Response;

    fn execution_id(&self) -> &str;
    fn kind(&self) -> &str;
    fn websocket_path(&self) -> &str;
    fn inc_run_metric(&self);
    fn dec_run_metric(&self);
}
