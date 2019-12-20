use crate::error::CouchbaseError;
use crate::instance::decrement_outstanding_requests;
use crate::instance::InstanceCookie;
use crate::options::*;
use crate::result::*;
use crate::subdoc::*;
use couchbase_sys::*;
use futures::channel::{mpsc, oneshot};
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::ptr;
use std::slice::from_raw_parts;
use std::time::Duration;
use uuid::Uuid;

type CouchbaseResult<T> = Result<T, CouchbaseError>;

pub trait InstanceRequest: Send + 'static {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE);
}

/// Special, internal request instructing the instance event loop to shutdown.
#[derive(Debug)]
pub struct ShutdownRequest {}

impl ShutdownRequest {
    pub fn new() -> Self {
        ShutdownRequest {}
    }
}

impl InstanceRequest for ShutdownRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let instance_cookie_ptr: *const c_void = unsafe { lcb_get_cookie(instance) };
        let mut instance_cookie =
            unsafe { Box::from_raw(instance_cookie_ptr as *mut Box<InstanceCookie>) };
        instance_cookie.set_shutdown();
        Box::into_raw(instance_cookie);
    }
}

#[derive(Debug)]
pub struct GetRequest {
    sender: oneshot::Sender<CouchbaseResult<GetResult>>,
    id: String,
    options: Option<GetOptions>,
}

impl GetRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<GetResult>>,
        id: String,
        options: Option<GetOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            options,
        }
    }
}

impl InstanceRequest for GetRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");
        let mut command: *mut lcb_CMDGET = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdget_create(&mut command);
            lcb_cmdget_key(command, id_encoded.as_ptr(), id_len);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdget_timeout(command, timeout.as_millis() as u32);
                }
            }
            lcb_get(instance, cookie, command);
            lcb_cmdget_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct GetAndLockRequest {
    sender: oneshot::Sender<CouchbaseResult<GetResult>>,
    id: String,
    options: Option<GetAndLockOptions>,
}

impl GetAndLockRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<GetResult>>,
        id: String,
        options: Option<GetAndLockOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            options,
        }
    }
}

impl InstanceRequest for GetAndLockRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");
        let mut command: *mut lcb_CMDGET = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdget_create(&mut command);
            lcb_cmdget_key(command, id_encoded.as_ptr(), id_len);

            let mut locktime = 30;
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdget_timeout(command, timeout.as_millis() as u32);
                }
                if let Some(lt) = options.lock_for() {
                    locktime = lt.as_secs() as u32;
                }
            }
            lcb_cmdget_expiration(command, locktime);
            lcb_get(instance, cookie, command);
            lcb_cmdget_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct GetAndTouchRequest {
    sender: oneshot::Sender<CouchbaseResult<GetResult>>,
    id: String,
    expiration: Duration,
    options: Option<GetAndTouchOptions>,
}

impl GetAndTouchRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<GetResult>>,
        id: String,
        expiration: Duration,
        options: Option<GetAndTouchOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            expiration,
            options,
        }
    }
}

impl InstanceRequest for GetAndTouchRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");
        let mut command: *mut lcb_CMDGET = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdget_create(&mut command);
            lcb_cmdget_key(command, id_encoded.as_ptr(), id_len);
            lcb_cmdget_expiration(command, self.expiration.as_secs() as u32);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdget_timeout(command, timeout.as_millis() as u32);
                }
            }
            lcb_get(instance, cookie, command);
            lcb_cmdget_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct UpsertRequest {
    sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
    id: String,
    content: Vec<u8>,
    flags: u32,
    options: Option<UpsertOptions>,
}

impl UpsertRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
        id: String,
        content: Vec<u8>,
        flags: u32,
        options: Option<UpsertOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            content,
            flags,
            options,
        }
    }
}

impl InstanceRequest for UpsertRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");

        let mut command: *mut lcb_CMDSTORE = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;

        let value_len = self.content.len();
        let value = CString::new(self.content).expect("Could not turn value into lcb format");

        unsafe {
            lcb_cmdstore_create(&mut command, lcb_STORE_OPERATION_LCB_STORE_UPSERT);
            lcb_cmdstore_key(command, id_encoded.as_ptr(), id_len);
            lcb_cmdstore_flags(command, self.flags);
            lcb_cmdstore_value(command, value.into_raw() as *const c_char, value_len);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdstore_timeout(command, timeout.as_millis() as u32);
                }
            }
            lcb_store(instance, cookie, command);
            lcb_cmdstore_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct InsertRequest {
    sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
    id: String,
    content: Vec<u8>,
    flags: u32,
    options: Option<InsertOptions>,
}

impl InsertRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
        id: String,
        content: Vec<u8>,
        flags: u32,
        options: Option<InsertOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            content,
            flags,
            options,
        }
    }
}

impl InstanceRequest for InsertRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");

        let mut command: *mut lcb_CMDSTORE = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;

        let value_len = self.content.len();
        let value = CString::new(self.content).expect("Could not turn value into lcb format");

        unsafe {
            lcb_cmdstore_create(&mut command, lcb_STORE_OPERATION_LCB_STORE_ADD);
            lcb_cmdstore_key(command, id_encoded.as_ptr(), id_len);
            lcb_cmdstore_flags(command, self.flags);
            lcb_cmdstore_value(command, value.into_raw() as *const c_char, value_len);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdstore_timeout(command, timeout.as_millis() as u32);
                }
            }
            lcb_store(instance, cookie, command);
            lcb_cmdstore_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct ReplaceRequest {
    sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
    id: String,
    content: Vec<u8>,
    flags: u32,
    options: Option<ReplaceOptions>,
}

impl ReplaceRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
        id: String,
        content: Vec<u8>,
        flags: u32,
        options: Option<ReplaceOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            content,
            flags,
            options,
        }
    }
}

impl InstanceRequest for ReplaceRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");

        let mut command: *mut lcb_CMDSTORE = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;

        let value_len = self.content.len();
        let value = CString::new(self.content).expect("Could not turn value into lcb format");

        unsafe {
            lcb_cmdstore_create(&mut command, lcb_STORE_OPERATION_LCB_STORE_REPLACE);
            lcb_cmdstore_key(command, id_encoded.as_ptr(), id_len);
            lcb_cmdstore_flags(command, self.flags);
            lcb_cmdstore_value(command, value.into_raw() as *const c_char, value_len);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdstore_timeout(command, timeout.as_millis() as u32);
                }
                if let Some(cas) = options.cas() {
                    lcb_cmdstore_cas(command, *cas);
                }
            }
            lcb_store(instance, cookie, command);
            lcb_cmdstore_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct RemoveRequest {
    sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
    id: String,
    options: Option<RemoveOptions>,
}

impl RemoveRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
        id: String,
        options: Option<RemoveOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            options,
        }
    }
}

impl InstanceRequest for RemoveRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");
        let mut command: *mut lcb_CMDREMOVE = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdremove_create(&mut command);
            lcb_cmdremove_key(command, id_encoded.as_ptr(), id_len);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdremove_timeout(command, timeout.as_millis() as u32);
                }
            }
            lcb_remove(instance, cookie, command);
            lcb_cmdremove_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct TouchRequest {
    sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
    id: String,
    expiration: Duration,
    options: Option<TouchOptions>,
}

impl TouchRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
        id: String,
        expiration: Duration,
        options: Option<TouchOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            expiration,
            options,
        }
    }
}

impl InstanceRequest for TouchRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");
        let mut command: *mut lcb_CMDTOUCH = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdtouch_create(&mut command);
            lcb_cmdtouch_key(command, id_encoded.as_ptr(), id_len);
            lcb_cmdtouch_expiration(command, self.expiration.as_secs() as u32);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdtouch_timeout(command, timeout.as_millis() as u32);
                }
            }
            lcb_touch(instance, cookie, command);
            lcb_cmdtouch_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct UnlockRequest {
    sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
    id: String,
    cas: u64,
    options: Option<UnlockOptions>,
}

impl UnlockRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<MutationResult>>,
        id: String,
        cas: u64,
        options: Option<UnlockOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            cas,
            options,
        }
    }
}

impl InstanceRequest for UnlockRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");
        let mut command: *mut lcb_CMDUNLOCK = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdunlock_create(&mut command);
            lcb_cmdunlock_key(command, id_encoded.as_ptr(), id_len);
            lcb_cmdunlock_cas(command, self.cas);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdunlock_timeout(command, timeout.as_millis() as u32);
                }
            }
            lcb_unlock(instance, cookie, command);
            lcb_cmdunlock_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct ExistsRequest {
    sender: oneshot::Sender<CouchbaseResult<ExistsResult>>,
    id: String,
    options: Option<ExistsOptions>,
}

impl ExistsRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<ExistsResult>>,
        id: String,
        options: Option<ExistsOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            options,
        }
    }
}

impl InstanceRequest for ExistsRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");
        let mut command: *mut lcb_CMDEXISTS = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdexists_create(&mut command);
            lcb_cmdexists_key(command, id_encoded.as_ptr(), id_len);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdexists_timeout(command, timeout.as_millis() as u32);
                }
            }
            lcb_exists(instance, cookie, command);
            lcb_cmdexists_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct QueryRequest {
    sender: oneshot::Sender<CouchbaseResult<QueryResult>>,
    rows_sender: mpsc::UnboundedSender<Vec<u8>>,
    rows_receiver: mpsc::UnboundedReceiver<Vec<u8>>,
    meta_sender: oneshot::Sender<Vec<u8>>,
    meta_receiver: oneshot::Receiver<Vec<u8>>,
    statement: String,
    options: Option<QueryOptions>,
}

impl QueryRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<QueryResult>>,
        statement: String,
        options: Option<QueryOptions>,
    ) -> Self {
        let (meta_sender, meta_receiver) = oneshot::channel();
        let (rows_sender, rows_receiver) = mpsc::unbounded();
        Self {
            sender,
            rows_sender,
            rows_receiver,
            meta_sender,
            meta_receiver,
            statement,
            options,
        }
    }

    unsafe fn add_default_client_context_id(command: *mut lcb_CMDN1QL) {
        let uuid = format!("{}", Uuid::new_v4());
        let len = uuid.len();
        let client_context_id = CString::new(uuid).unwrap();
        lcb_cmdn1ql_client_context_id(command, client_context_id.as_ptr(), len);
    }
}

impl InstanceRequest for QueryRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let statement_len = self.statement.len();
        let statement_encoded = CString::new(self.statement).expect("Could not encode Statement");
        let mut command: *mut lcb_CMDN1QL = ptr::null_mut();

        let sender_boxed = Box::new(QueryCookie {
            result: Some(self.sender),
            rows_sender: self.rows_sender,
            rows_receiver: Some(self.rows_receiver),
            meta_sender: self.meta_sender,
            meta_receiver: Some(self.meta_receiver),
        });
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdn1ql_create(&mut command);
            lcb_cmdn1ql_statement(command, statement_encoded.as_ptr(), statement_len);
            if let Some(options) = self.options {
                // NOTE: When the rest of SDK 3.0 is implemented, this will need to have a check
                // for the `consistentWith(MutationState)`. For now, we aren't implmenting that
                // feature.
                lcb_cmdn1ql_consistency(command, options.scan_consistency() as u32);

                if let Some(timeout) = options.timeout() {
                    lcb_cmdn1ql_timeout(command, timeout.as_millis() as u32);
                }
                if let Some(params) = options.positional_parameters() {
                    for param in params {
                        lcb_cmdn1ql_positional_param(command, param.0.as_ptr(), param.1);
                    }
                }
                if let Some(params) = options.named_parameters() {
                    for param in params {
                        let key = param.0;
                        let value = param.1;
                        lcb_cmdn1ql_named_param(
                            command,
                            key.0.as_ptr(),
                            key.1,
                            value.0.as_ptr(),
                            value.1,
                        );
                    }
                }

                if let Some(client_context_id) = options.client_context_id() {
                    lcb_cmdn1ql_client_context_id(
                        command,
                        client_context_id.0.as_ptr(),
                        client_context_id.1,
                    );
                } else {
                    QueryRequest::add_default_client_context_id(command);
                }
            } else {
                QueryRequest::add_default_client_context_id(command);
            }

            lcb_cmdn1ql_callback(command, Some(n1ql_callback));
            lcb_n1ql(instance, cookie, command);
            lcb_cmdn1ql_destroy(command);
        }
    }
}

struct QueryCookie {
    result: Option<oneshot::Sender<CouchbaseResult<QueryResult>>>,
    rows_sender: mpsc::UnboundedSender<Vec<u8>>,
    rows_receiver: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    meta_sender: oneshot::Sender<Vec<u8>>,
    meta_receiver: Option<oneshot::Receiver<Vec<u8>>>,
}

unsafe extern "C" fn n1ql_callback(
    instance: *mut lcb_INSTANCE,
    _cbtype: i32,
    res: *const lcb_RESPN1QL,
) {
    let mut row_len: usize = 0;
    let mut row_ptr: *const c_char = ptr::null();
    lcb_respn1ql_row(res, &mut row_ptr, &mut row_len);
    let row = from_raw_parts(row_ptr as *const u8, row_len);

    let mut cookie_ptr: *mut c_void = ptr::null_mut();
    lcb_respn1ql_cookie(res, &mut cookie_ptr);
    let mut cookie = Box::from_raw(cookie_ptr as *mut QueryCookie);

    if cookie.result.is_some() {
        cookie
            .result
            .take()
            .expect("Could not take result!")
            .send(Ok(QueryResult::new(
                cookie.rows_receiver.take().unwrap(),
                cookie.meta_receiver.take().unwrap(),
            )))
            .expect("Could not complete Future!");
    }

    if lcb_respn1ql_is_final(res) != 0 {
        decrement_outstanding_requests(instance);
        cookie
            .meta_sender
            .send(row.to_vec())
            .expect("Could not send meta");
    } else {
        cookie
            .rows_sender
            .unbounded_send(row.to_vec())
            .expect("Could not send row");
        Box::into_raw(cookie);
    }
}

#[derive(Debug)]
pub struct AnalyticsRequest {
    sender: oneshot::Sender<CouchbaseResult<AnalyticsResult>>,
    rows_sender: mpsc::UnboundedSender<Vec<u8>>,
    rows_receiver: mpsc::UnboundedReceiver<Vec<u8>>,
    meta_sender: oneshot::Sender<Vec<u8>>,
    meta_receiver: oneshot::Receiver<Vec<u8>>,
    statement: String,
    options: Option<AnalyticsOptions>,
}

impl AnalyticsRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<AnalyticsResult>>,
        statement: String,
        options: Option<AnalyticsOptions>,
    ) -> Self {
        let (meta_sender, meta_receiver) = oneshot::channel();
        let (rows_sender, rows_receiver) = mpsc::unbounded();
        Self {
            sender,
            rows_sender,
            rows_receiver,
            meta_sender,
            meta_receiver,
            statement,
            options,
        }
    }
}

impl InstanceRequest for AnalyticsRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let statement_len = self.statement.len();
        let statement_encoded = CString::new(self.statement).expect("Could not encode Statement");
        let mut command: *mut lcb_CMDANALYTICS = ptr::null_mut();

        let sender_boxed = Box::new(AnalyticsCookie {
            result: Some(self.sender),
            rows_sender: self.rows_sender,
            rows_receiver: Some(self.rows_receiver),
            meta_sender: self.meta_sender,
            meta_receiver: Some(self.meta_receiver),
        });
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdanalytics_create(&mut command);
            lcb_cmdanalytics_statement(command, statement_encoded.as_ptr(), statement_len);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdanalytics_timeout(command, timeout.as_millis() as u32);
                }
                if let Some(params) = options.positional_parameters() {
                    for param in params {
                        lcb_cmdanalytics_positional_param(command, param.0.as_ptr(), param.1);
                    }
                }
                if let Some(params) = options.named_parameters() {
                    for param in params {
                        let key = param.0;
                        let value = param.1;
                        lcb_cmdanalytics_named_param(
                            command,
                            key.0.as_ptr(),
                            key.1,
                            value.0.as_ptr(),
                            value.1,
                        );
                    }
                }
            }
            lcb_cmdanalytics_callback(command, Some(analytics_callback));
            lcb_analytics(instance, cookie, command);
            lcb_cmdanalytics_destroy(command);
        }
    }
}

struct AnalyticsCookie {
    result: Option<oneshot::Sender<CouchbaseResult<AnalyticsResult>>>,
    rows_sender: mpsc::UnboundedSender<Vec<u8>>,
    rows_receiver: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    meta_sender: oneshot::Sender<Vec<u8>>,
    meta_receiver: Option<oneshot::Receiver<Vec<u8>>>,
}

unsafe extern "C" fn analytics_callback(
    instance: *mut lcb_INSTANCE,
    _cbtype: i32,
    res: *const lcb_RESPANALYTICS,
) {
    let mut row_len: usize = 0;
    let mut row_ptr: *const c_char = ptr::null();
    lcb_respanalytics_row(res, &mut row_ptr, &mut row_len);
    let row = from_raw_parts(row_ptr as *const u8, row_len);

    let mut cookie_ptr: *mut c_void = ptr::null_mut();
    lcb_respanalytics_cookie(res, &mut cookie_ptr);
    let mut cookie = Box::from_raw(cookie_ptr as *mut AnalyticsCookie);

    if cookie.result.is_some() {
        cookie
            .result
            .take()
            .expect("Could not take result!")
            .send(Ok(AnalyticsResult::new(
                cookie.rows_receiver.take().unwrap(),
                cookie.meta_receiver.take().unwrap(),
            )))
            .expect("Could not complete Future!");
    }

    if lcb_respanalytics_is_final(res) != 0 {
        decrement_outstanding_requests(instance);
        cookie
            .meta_sender
            .send(row.to_vec())
            .expect("Could not send meta");
    } else {
        cookie
            .rows_sender
            .unbounded_send(row.to_vec())
            .expect("Could not send row");
        Box::into_raw(cookie);
    }
}

#[derive(Debug)]
pub struct LookupInRequest {
    sender: oneshot::Sender<CouchbaseResult<LookupInResult>>,
    id: String,
    specs: Vec<LookupInSpec>,
    options: Option<LookupInOptions>,
}

impl LookupInRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<LookupInResult>>,
        id: String,
        specs: Vec<LookupInSpec>,
        options: Option<LookupInOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            specs,
            options,
        }
    }
}

impl InstanceRequest for LookupInRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");
        let mut command: *mut lcb_CMDSUBDOC = ptr::null_mut();
        let mut ops: *mut lcb_SUBDOCOPS = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdsubdoc_create(&mut command);
            lcb_cmdsubdoc_key(command, id_encoded.as_ptr(), id_len);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdsubdoc_timeout(command, timeout.as_millis() as u32);
                }
            }

            lcb_subdocops_create(&mut ops, self.specs.len());
            let mut idx = 0;
            for spec in &self.specs {
                let flags = 0;
                match spec.command_type() {
                    SubdocLookupCommandType::Get => {
                        lcb_subdocops_get(ops, idx, flags, spec.path().as_ptr(), spec.path_len());
                    }
                    SubdocLookupCommandType::Count => {
                        lcb_subdocops_get_count(
                            ops,
                            idx,
                            flags,
                            spec.path().as_ptr(),
                            spec.path_len(),
                        );
                    }
                    SubdocLookupCommandType::Exists => {
                        lcb_subdocops_exists(
                            ops,
                            idx,
                            flags,
                            spec.path().as_ptr(),
                            spec.path_len(),
                        );
                    }
                    SubdocLookupCommandType::GetDoc => {
                        lcb_subdocops_fulldoc_get(ops, idx, flags);
                    }
                }
                idx += 1;
            }
            lcb_cmdsubdoc_operations(command, ops);
            lcb_subdoc(instance, cookie, command);
            lcb_subdocops_destroy(ops);
            lcb_cmdsubdoc_destroy(command);
        }
    }
}

#[derive(Debug)]
pub struct MutateInRequest {
    sender: oneshot::Sender<CouchbaseResult<MutateInResult>>,
    id: String,
    specs: Vec<MutateInSpec>,
    options: Option<MutateInOptions>,
}

impl MutateInRequest {
    pub fn new(
        sender: oneshot::Sender<CouchbaseResult<MutateInResult>>,
        id: String,
        specs: Vec<MutateInSpec>,
        options: Option<MutateInOptions>,
    ) -> Self {
        Self {
            sender,
            id,
            specs,
            options,
        }
    }
}

impl InstanceRequest for MutateInRequest {
    fn encode(self: Box<Self>, instance: *mut lcb_INSTANCE) {
        let id_len = self.id.len();
        let id_encoded = CString::new(self.id).expect("Could not encode ID");
        let mut command: *mut lcb_CMDSUBDOC = ptr::null_mut();
        let mut ops: *mut lcb_SUBDOCOPS = ptr::null_mut();

        let sender_boxed = Box::new(self.sender);
        let cookie = Box::into_raw(sender_boxed) as *mut c_void;
        unsafe {
            lcb_cmdsubdoc_create(&mut command);
            lcb_cmdsubdoc_key(command, id_encoded.as_ptr(), id_len);
            if let Some(options) = self.options {
                if let Some(timeout) = options.timeout() {
                    lcb_cmdsubdoc_timeout(command, timeout.as_millis() as u32);
                }
                if let Some(cas) = options.cas() {
                    lcb_cmdsubdoc_cas(command, *cas);
                }
            }

            lcb_subdocops_create(&mut ops, self.specs.len());

            let mut idx = 0;
            for spec in &self.specs {
                let flags = 0;
                match spec.command_type() {
                    SubdocMutationCommandType::Insert => {
                        lcb_subdocops_dict_add(
                            ops,
                            idx,
                            flags,
                            spec.path().as_ptr(),
                            spec.path_len(),
                            spec.content().as_ptr(),
                            spec.content_len(),
                        );
                    }
                    SubdocMutationCommandType::Upsert => {
                        lcb_subdocops_dict_upsert(
                            ops,
                            idx,
                            flags,
                            spec.path().as_ptr(),
                            spec.path_len(),
                            spec.content().as_ptr(),
                            spec.content_len(),
                        );
                    }
                    SubdocMutationCommandType::Replace => {
                        lcb_subdocops_replace(
                            ops,
                            idx,
                            flags,
                            spec.path().as_ptr(),
                            spec.path_len(),
                            spec.content().as_ptr(),
                            spec.content_len(),
                        );
                    }
                    SubdocMutationCommandType::Remove => {
                        lcb_subdocops_remove(
                            ops,
                            idx,
                            flags,
                            spec.path().as_ptr(),
                            spec.path_len(),
                        );
                    }
                }
                idx += 1;
            }

            lcb_cmdsubdoc_operations(command, ops);
            lcb_subdoc(instance, cookie, command);
            lcb_subdocops_destroy(ops);
            lcb_cmdsubdoc_destroy(command);
        }
    }
}
