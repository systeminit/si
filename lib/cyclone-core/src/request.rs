use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use si_crypto::SensitiveStrings;
use si_std::SensitiveString;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycloneRequest<R> {
    request: R,
    sensitive_strings: HashSet<SensitiveString>,
}

impl<R> CycloneRequest<R> {
    pub fn from_parts(request: R, sensitive_strings: SensitiveStrings) -> Self {
        Self {
            request,
            sensitive_strings: sensitive_strings.into(),
        }
    }

    pub fn into_parts(self) -> (R, SensitiveStrings) {
        (self.request, self.sensitive_strings.into())
    }
}
