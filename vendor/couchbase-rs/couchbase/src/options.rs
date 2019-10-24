//! Option arguments for all operations.

use serde_json::to_vec;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::CString;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct GetOptions {
    timeout: Option<Duration>,
}

impl GetOptions {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }
}

#[derive(Debug, Default)]
pub struct GetAndLockOptions {
    timeout: Option<Duration>,
    lock_for: Option<Duration>,
}

impl GetAndLockOptions {
    pub fn new() -> Self {
        Self {
            timeout: None,
            lock_for: None,
        }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }

    pub fn set_lock_for(mut self, lock_for: Duration) -> Self {
        self.lock_for = Some(lock_for);
        self
    }

    pub fn lock_for(&self) -> &Option<Duration> {
        &self.lock_for
    }
}

#[derive(Debug, Default)]
pub struct GetAndTouchOptions {
    timeout: Option<Duration>,
}

impl GetAndTouchOptions {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }
}

#[derive(Debug, Default)]
pub struct InsertOptions {
    timeout: Option<Duration>,
}

impl InsertOptions {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }
}

#[derive(Debug, Default)]
pub struct UpsertOptions {
    timeout: Option<Duration>,
}

impl UpsertOptions {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }
}

#[derive(Debug, Default)]
pub struct ReplaceOptions {
    timeout: Option<Duration>,
    cas: Option<u64>,
}

impl ReplaceOptions {
    pub fn new() -> Self {
        Self {
            timeout: None,
            cas: None,
        }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }

    pub fn set_cas(mut self, cas: u64) -> Self {
        self.cas = Some(cas);
        self
    }

    pub fn cas(&self) -> &Option<u64> {
        &self.cas
    }
}

#[derive(Debug, Default)]
pub struct RemoveOptions {
    timeout: Option<Duration>,
}

impl RemoveOptions {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }
}

// This maps to LCB_N1QL_CONSISTENCY enum in couchbase.h
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ScanConsistency {
    // LCB_N1QL_CONSISTENCY_NONE
    NotBounded = 0,
    // LCB_N1QL_CONSISTENCY_REQUEST
    RequestPlus = 2,
}

impl Default for ScanConsistency {
    fn default() -> Self {
        ScanConsistency::NotBounded
    }
}

#[derive(Debug, Default)]
pub struct QueryOptions {
    timeout: Option<Duration>,
    positional_parameters: Option<Vec<(CString, usize)>>,
    named_parameters: Option<HashMap<(CString, usize), (CString, usize)>>,
    client_context_id: Option<(CString, usize)>,
    scan_consistency: ScanConsistency,
}

impl QueryOptions {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn set_scan_consistency(mut self, scan_consistency: ScanConsistency) -> Self {
        self.scan_consistency = scan_consistency;
        self
    }

    pub fn scan_consistency(&self) -> ScanConsistency {
        self.scan_consistency
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }

    pub fn set_positional_parameters(mut self, params: Vec<Value>) -> Self {
        let mut positional = vec![];
        for param in params {
            let serialized = match to_vec(&param) {
                Ok(v) => v,
                Err(_e) => panic!("Could not encode n1ql positional param"),
            };
            let len = serialized.len();
            positional.push((CString::new(serialized).unwrap(), len));
        }
        self.positional_parameters = Some(positional);
        self
    }

    pub(crate) fn positional_parameters(&self) -> &Option<Vec<(CString, usize)>> {
        &self.positional_parameters
    }

    pub fn set_named_parameters(mut self, params: HashMap<String, Value>) -> Self {
        let mut named = HashMap::new();
        for param in params {
            let serialized = match to_vec(&param.1) {
                Ok(v) => v,
                Err(_e) => panic!("Could not encode n1ql positional param"),
            };
            let len = serialized.len();
            let key_len = param.0.len();
            named.insert(
                (CString::new(param.0).unwrap(), key_len),
                (CString::new(serialized).unwrap(), len),
            );
        }
        self.named_parameters = Some(named);
        self
    }

    pub(crate) fn named_parameters(&self) -> &Option<HashMap<(CString, usize), (CString, usize)>> {
        &self.named_parameters
    }

    pub fn set_client_context_id(mut self, client_context_id: String) -> Self {
        let client_context_id_len = client_context_id.len();
        self.client_context_id = Some((
            CString::new(client_context_id).unwrap(),
            client_context_id_len,
        ));
        self
    }

    pub(crate) fn client_context_id(&self) -> &Option<(CString, usize)> {
        &self.client_context_id
    }
}

#[derive(Debug, Default)]
pub struct AnalyticsOptions {
    timeout: Option<Duration>,
    positional_parameters: Option<Vec<(CString, usize)>>,
    named_parameters: Option<HashMap<(CString, usize), (CString, usize)>>,
}

impl AnalyticsOptions {
    pub fn new() -> Self {
        Self {
            timeout: None,
            positional_parameters: None,
            named_parameters: None,
        }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }

    pub fn set_positional_parameters(mut self, params: Vec<Value>) -> Self {
        let mut positional = vec![];
        for param in params {
            let serialized = match to_vec(&param) {
                Ok(v) => v,
                Err(_e) => panic!("Could not encode n1ql positional param"),
            };
            let len = serialized.len();
            positional.push((CString::new(serialized).unwrap(), len));
        }
        self.positional_parameters = Some(positional);
        self
    }

    pub(crate) fn positional_parameters(&self) -> &Option<Vec<(CString, usize)>> {
        &self.positional_parameters
    }

    pub fn set_named_parameters(mut self, params: HashMap<String, Value>) -> Self {
        let mut named = HashMap::new();
        for param in params {
            let serialized = match to_vec(&param.1) {
                Ok(v) => v,
                Err(_e) => panic!("Could not encode n1ql positional param"),
            };
            let len = serialized.len();
            let key_len = param.0.len();
            named.insert(
                (CString::new(param.0).unwrap(), key_len),
                (CString::new(serialized).unwrap(), len),
            );
        }
        self.named_parameters = Some(named);
        self
    }

    pub(crate) fn named_parameters(&self) -> &Option<HashMap<(CString, usize), (CString, usize)>> {
        &self.named_parameters
    }
}

#[derive(Debug, Default)]
pub struct TouchOptions {
    timeout: Option<Duration>,
}

impl TouchOptions {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }
}

#[derive(Debug, Default)]
pub struct UnlockOptions {
    timeout: Option<Duration>,
}

impl UnlockOptions {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }
}

#[derive(Debug, Default)]
pub struct ExistsOptions {
    timeout: Option<Duration>,
}

impl ExistsOptions {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }
}

#[derive(Debug, Default)]
pub struct LookupInOptions {
    timeout: Option<Duration>,
}

impl LookupInOptions {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }
}

#[derive(Debug, Default)]
pub struct MutateInOptions {
    timeout: Option<Duration>,
    cas: Option<u64>,
}

impl MutateInOptions {
    pub fn new() -> Self {
        Self {
            timeout: None,
            cas: None,
        }
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn timeout(&self) -> &Option<Duration> {
        &self.timeout
    }

    pub fn set_cas(mut self, cas: u64) -> Self {
        self.cas = Some(cas);
        self
    }

    pub fn cas(&self) -> &Option<u64> {
        &self.cas
    }
}
