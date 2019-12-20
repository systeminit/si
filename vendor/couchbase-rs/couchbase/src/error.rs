//! Error handling structures and functionality.

use crate::instance;

use couchbase_sys::*;
use std::{convert, error, fmt, io};

use futures;
use serde_json;

/// Defines all possible errors that can result as part of interacting with the SDK.
///
/// Note that most of these types directly correspond to their equivalents in
/// `libcouchbase` but might have been renamed and/or removed since they are not
/// needed in a higher level context. Also, don't rely on the ordering of this enum
/// since it might be rearranged or expanded at a later point.
///
/// To make interaction simpler, conversion traits have been implemented for the
/// underlying libcouchbase type (`lcb_status`) as well as for IO errors. This enum
/// also implements the generic rust `Error` trait and can be treated as such throughout
/// the application context.
#[derive(Debug)]
pub enum CouchbaseError {
    /// Call succeded.
    Success,
    IoError(std::io::Error),
    /// Raised when there has been a problem with dispatching the rust future.
    FutureError(String),
    /// Decoding from json or another type failed.
    DecodingError(serde_json::Error),
    InternalChannelSendError(String),
    /// Encoding into json failed.
    EncodingError,
    /// This error is received when connecting or reconnecting to the cluster.
    ///
    /// If received during initial bootstrap then it should be considered a fatal errror.
    /// This error should not be visible after initial bootstrap. This error may also be
    /// received if CCCP bootstrap is used and the bucket does not exist.
    AuthFailed,
    /// This error is a result of trying to perform an arithmetic operation on an item
    /// which has an existing value that cannot be parsed as a number.
    DeltaBadval,
    /// This error indicates that the key and value exceeded the constraints within the
    /// server.
    ///
    /// The current constraints are 150 bytes for the key and 20MB for the value.
    TooBig,
    /// This error indicates that the server is currently busy.
    ServerBusy,
    /// An internal error within the SDK, this might be the result of a bug.
    Internal,
    /// If returned from an API call, it indicates invalid values were passed
    /// to the function.
    ///
    /// If received within a callback, it indicates that a malformed packet was sent to
    /// the server.
    InvalidValue,
    /// This error indicates that the server has no more memory left to store or modify
    /// the document.
    NoMemoryLeft,
    InvalidRange,
    /// Indicates a generic error.
    Generic,
    /// This error indicates that the server could not perform the requested operation right now.
    ///
    /// This is usually due to memory and/or resource constraints on the server. This error may also
    /// be returned if a key has been locked and an operation has been performed on it without
    /// unlocking the item.
    TemporaryFailure,
    /// The key already exists in the cluster.
    ///
    /// This error code is received as a result of an insert operation in which the key already
    /// exists. It is also received for other operations in which a CAS was specified but has
    /// changed on the server.
    KeyExists,
    /// The bucket does not contain the document for the given key.
    KeyDoesNotExist,
    /// Indicates that through libcouchbase the used I/O plugin could not be located.
    PluginLibraryNotFound,
    /// Indicates that through libcouchbase the I/O plugin does not contain a proper
    /// initialization routine.
    PluginInitializerNotFound,
    /// This is a generic error code returned for various forms of socketoperation failures.
    NetworkFailure,
    /// Error code received if the command was forwarded to the wrong server (for example,
    /// during a rebalance) and the library settings are configured that the command should
    /// not be remapped to a new server.
    NotMyVBucket,
    /// Received as a response to an `append` or `prepend on a document that did not exist
    /// in the cluster.
    ///
    /// Equivalent to `KeyDoesNotExist`.
    NotStored,
    /// Returned from API calls if a specific operation is valid but is unsupported
    /// in the current version or state of the library.
    ///
    /// May also be received in a callback if the cluster does not support the operation.
    NotSupported,
    /// Received if the cluster does not know about the command. Similar to `NotSupported`.
    UnknownCommand,
    /// Received if the hostname specified could not be found.
    ///
    /// It may also be received if a socket could not be created to the host supplied.
    UnknownHost,
    /// Received if the server replied with an unexpected response.
    ProtocolError,
    /// Received for operations which did not receive a reply from the server within the
    /// timeout limit.
    TimedOut,
    /// Generic error while establishing a TCP connection.
    ConnectError,
    /// Received on initial bootstrap if the bucket does not exist.
    ///
    /// Note that for CCCP bootstrap, `AuthFailied` will be received instead.
    BucketDoesNotExist,
    /// Libcouchbase could not allocate memory for internal structures, woops!
    MemoryAllocationFailure,
    /// Client could not schedule the request. This is typically received when
    /// an operation is requested before the initial bootstrap has completed.
    RequestNotScheduled,
    /// Bad handle type for this operation.
    ///
    /// Pne cannot perform administrative operations on a data handle, or data operations on
    /// a cluster handle.
    BadHandle,
    /// You found a server bug, congratulations!
    ServerBug,
    /// The used version of libcouchbase cannot load the specified plugin.
    PluginVersionMismatch,
    /// Hostname specified for URI is in an invalid format.
    InvalidHostFormat,
    /// Illegal character used.
    InvalidChar,
    /// Received in response to the durability API call, if the amount of nodes
    /// or replicas to persist/replicate to exceed the total number of replicas the
    /// bucket was configured with.
    InvalidDurabilityRequirement,
    /// Received in scheduling if a command with the same key was specified more
    /// than once. Some commands will accept this, but others (notably `observe`) will not.
    DuplicateCommands,
    /// This error is received from API calls if the master node for the vBucket
    /// the key has been hashed to is not present.
    ///
    /// This will happen in the result of a node failover where no replica exists to
    /// replace it.
    NoMatchingServer,
    /// Received during initial creation if an environment variable was specified with
    /// an incorrect or invalid value.
    BadEnvironment,
    /// Client (libcouchbase) is busy - this is an internal error.
    ClientBusy,
    /// Received if the username does not match the bucket.
    InvalidUsername,
    /// The contents of the configuration cache file were invalid.
    ConfigCacheInvalid,
    /// Received during initial bootstrap if the library was configured to force
    /// the usage of a specific SASL mechanism and the server did not support this
    /// mechanism.
    SaslMechUnavailable,
    /// Received in the HTTP callback if the response was redirected too many times.
    TooManyRedirects,
    ///  May be received in operation callbacks if the cluster toplogy changed
    /// and the library could not remap the command to a new node.
    ///
    /// This may be because the internal structure lacked sufficient information to
    /// recreate the packet, or because the configuration settings indicated that the
    /// command should not be retried.
    MapChanged,
    /// Returned from libcouchbase functions if an incomplete packet was passed.
    IncompletePacket,
    /// Mapped directly to the system `ECONNREFUSED` errno. This is received
    /// if an initial connection to the node could not be established.
    ///
    /// Hint: Check your firewall settings and ensure the specified service is online.
    ConnectionRefused,
    /// Returned if the socket connection was gracefully closed, but the library wasn't
    /// expecting it.
    ///
    /// This may happen if the system is being shut down.
    SocketShutdown,
    /// Returned in a callback if the socket connection was forcefully reset.
    ///
    /// Equivalent to the system `ECONNRESET`.
    ConnectionReset,
    /// Returned if the library could not allocated a local socket due to TCP local port
    /// exhaustion.
    ///
    /// This means you have either found a bug in the library or are creating too many TCP
    /// connections. Keep in mind that a TCP connection will still occupy a slot in your
    /// system socket table even after it has been closed (and will thus appear in a
    /// `TIME_WAIT` state).
    PortAllocationFailed,
    /// Returned if the library could not allocate a new file descriptor for a
    /// socket or other resource.
    ///
    /// This may be more common on systems (such as Mac OS X) which have relatively low
    /// limits for file descriptors. To raise the file descriptor limit, refer to the
    /// `ulimit -n` command.
    FileDescriptorLimitReached,
    ///  Returned if the host or subnet containing a node could not be contacted.
    ///
    /// This may be a result of a bad routing table or being physically disconnected from
    /// the network.
    NetworkUnreachable,
    /// An unrecognized setting was used for the control functions.
    ControlCommandUnknown,
    /// An invalid operation was supplied for a setting to the control functions.
    ///
    /// This will happen if you try to write to a read-only setting, or retrieve a value
    /// which may only be set. Refer to the documentation for an individual setting
    /// to see what modes it supports.
    ControlCommandUnsupported,
    ///  A malformed argument was passed to the control functions for the given setting.
    ///
    /// See the documentation for the setting to see what arguments it supports and
    /// how they are to be supplied.
    ControlCommandBadArgument,
    /// An empty key was passed to an operation. Most commands do not accept empty keys.
    EmptyKey,
    ///  A problem with the SSL system was encountered.
    ///
    /// This error will only be thrown if something internal to the SSL library failed (for
    /// example, a bad certificate or bad user input); otherwise a network error will be
    /// thrown if an SSL connection was terminated.
    EncryptionError,
    /// The certificate the server sent cannot be verified.
    ///
    /// This is a possible case of a man-in-the-middle attack, but also of forgetting to
    /// supply the path to the CA authority to the library.
    EncryptionCannotVerify,
    /// Internal failure for not properly scheduling operations.
    InternalScheduleFailure,
    /// An optional client feature was requested, but the current configuration
    /// does not allow it to be used.
    ///
    /// This might be because it is not available on a particular platform/architecture/operating
    /// system/configuration, or it has been disabled at the time the library was built.
    ClientFeatureUnavailable,
    /// An option was passed to a command which is incompatible with other
    /// options.
    ///
    /// This may happen if two fields are mutually exclusive.
    OptionsConflict,
    /// Received if an operation failed because of a negative HTTP status code.
    HttpError,
    /// Scheduling error received if mutation tokens were enabled, but there is no available
    /// mutation token for the key.
    DurabilityNoMutationTokens,
    /// The server replied with an unrecognized status code.
    UnknownStatusCode,
    /// The server replied that the given mutation has been lost.
    MutationLost,
    /// The Subdocument path does not exist.
    SubdocPathDoesNotExist,
    /// Type of element in sub-document path conflicts with type in document.
    SubdocPathMismatch,
    /// Malformed sub-document path.
    SubdocPathMalformed,
    /// Sub-document contains too many components.
    SubdocPathTooBig,
    /// Existing document contains too many levels of nesting.
    SubdocExistingValueToeep,
    /// Subdocument operation would invalidate the JSON.
    SubdocCannotInsert,
    /// Existing document is not valid JSON.
    SubdocExistingNotJson,
    /// The existing numeric value is too large.
    SubdocNumericValueTooLarge,
    /// Delta must be numeric, within the 64 bit signed range, and non-zero.
    SubdocBadDelta,
    /// The given path already exists in the document.
    SubdocPathExists,
    /// Could not execute one or more multi lookups or mutations.
    SubdocMultiFailure,
    /// Value is too deep to insert.
    SubdocValueTooDeepToInsert,
    /// A badly formatted packet was sent to the server. Please report this in a bug.
    InvalidPacket,
    /// Missing subdocument path.
    SubdocEmptyPath,
    /// Unknown subdocument command.
    SubdocUnknownCommand,
    /// No commands specified.
    NoCommandsSpecified,
    /// Query execution failed. Inspect raw response object for information.
    QueryError,
    /// Generic temporary error received from server.
    GenericTmpError,
    /// Generic subdocument error received from server.
    GenericSubdocError,
    /// Generic constraint error received from server.
    GenericConstraintError,
    /// Invalid reply received from nameserver.
    NameserverError,
    /// Not authorized for operation.
    NotAuthorized,
    /// An invalid combination of commands was specified.
    SubdocInvalidCombo,
    /// Specified key was successfully found, but one or more path operations failed.
    SubdocMultiPathFailure,
    /// The operation completed successfully, but operated on a deleted document.
    SubdocSuccessDeleted,
    /// The combination of the subdoc flags for the xattrs is not valid.
    SubdocXattrInvalidFlagCombo,
    /// Only a single xattr key may be accessed at the same time.
    SubdocXattrInvalidKeyCombo,
    /// The server has no knowledge of the requested macro.
    SubdocXattrUnknownMacro,
    /// The server has no knowledge of the requested virtual xattr.
    SubdocXattrUnknownVattr,
    /// Virtual xattrs cannot be modified.
    SubdocXattrCantModifyVattr,
    /// Specified key was found as a deleted document, but one or more path operations failed.
    SubdocMultiPathFailureDeleted,
    /// According to the spec all xattr commands should come first, followed by the commands for the document body.
    SubdocInvalidXattrOrder,
    /// Unknown collection.
    CollectionUnknown,
    /// Operation attempted and requires that the collections manifest is set.
    CollectionNoManifest,
    /// Bucket Manifest update could not be applied to vbucket(s).
    CollectionCannotApplyManifest,
    /// Client has a collection's manifest which is from the future.
    CollectionManifestIsAhead,
    /// The given durability level is invalid.
    DurabilityInvalidLevel,
    /// Valid request, but given durability requirements are impossible to achieve,
    /// because insufficient configured replicas are connected.
    ///
    /// Assuming level=majority and C=number of configured nodes, durability becomes impossible if
    /// floor((C + 1) / 2) nodes or greater are offline.
    DurabilityImpossible,
    /// Returned if an attempt is made to mutate a key which already has a SyncWrite pending.
    ///
    /// Client would typically retry (possibly with backoff). Similar to Locked.
    DurabilitySyncWriteInProgress,
    /// The SyncWrite request has not completed in the specified time and has ambiguous result -
    /// it may succeed or fail but the final value is not yet known.
    DurabilitySyncWriteAmbiguous,
    /// If the rust binding doesn't know about an error its contained here.
    UnknownLibcouchbaseError(u32),
    RowsConsumed,
    MetaConsumed,
    FutureCanceled(futures::channel::oneshot::Canceled),
}

impl CouchbaseError {
    fn as_str(&self) -> &'static str {
        match *self {
            CouchbaseError::RowsConsumed => { "Rows already consumed in result" },
            CouchbaseError::MetaConsumed => { "Meta already consumed in result" },
            CouchbaseError::FutureCanceled(_) => { "Future is canceled" },
            CouchbaseError::IoError(_) => { "IO Error" },
            CouchbaseError::Success => { "Success" },
            CouchbaseError::InternalChannelSendError(_) => { "Error sending to internal instance channel" },
            CouchbaseError::FutureError(_) => {
                "Could not dispatch the rust future"
            },
            CouchbaseError::DecodingError(_) => "Decoding failed",
            CouchbaseError::EncodingError => "Encoding failed",
            CouchbaseError::AuthFailed => {
                "Authentication failed. You may have provided an invalid username/password \
                 combination"
            }
            CouchbaseError::DeltaBadval => {
                "The value requested to be incremented is not stored as a number"
            }
            CouchbaseError::TooBig => "The object requested is too big to store in the server",
            CouchbaseError::ServerBusy => "The server is busy. Try again later",
            CouchbaseError::Internal => "Internal libcouchbase error",
            CouchbaseError::InvalidValue => "Invalid input/arguments",
            CouchbaseError::NoMemoryLeft => "The server is out of memory. Try again later",
            CouchbaseError::InvalidRange => "Invalid range",
            CouchbaseError::Generic => "Generic error",
            CouchbaseError::TemporaryFailure => {
                "Temporary failure received from server. Try again later"
            }
            CouchbaseError::KeyExists => {
                "The key already exists in the server. If you have supplied a CAS then the key \
                 exists with a CAS value different than specified"
            }
            CouchbaseError::KeyDoesNotExist => "The key does not exist on the server",
            CouchbaseError::PluginLibraryNotFound => "Could not locate plugin library",
            CouchbaseError::PluginInitializerNotFound => "Required plugin initializer not found",
            CouchbaseError::NetworkFailure => "Generic network failure",
            CouchbaseError::NotMyVBucket => {
                "The server which received this command claims it is not hosting this key"
            }
            CouchbaseError::NotStored => {
                "Item not stored (did you try to append/prepend to a missing key?)"
            }
            CouchbaseError::NotSupported => "Operation not supported",
            CouchbaseError::UnknownCommand => "Unknown command",
            CouchbaseError::UnknownHost => "DNS/Hostname lookup failed",
            CouchbaseError::ProtocolError => {
                "Data received on socket was not in the expected format"
            }
            CouchbaseError::TimedOut => {
                "Client-Side timeout exceeded for operation. Inspect network conditions or \
                 increase the timeout"
            }
            CouchbaseError::ConnectError => "Error while establishing TCP connection",
            CouchbaseError::BucketDoesNotExist => "The bucket requested does not exist",
            CouchbaseError::MemoryAllocationFailure => {
                "Memory allocation for libcouchbase failed. Severe problems ahead"
            }
            CouchbaseError::RequestNotScheduled => {
                "Client not bootstrapped. Ensure bootstrap/connect was attempted and was successful"
            }
            CouchbaseError::BadHandle => {
                "Bad handle type for operation. You cannot perform administrative operations on a \
                 data handle, or data operations on a cluster handle"
            }
            CouchbaseError::ServerBug => "Encountered a server bug",
            CouchbaseError::PluginVersionMismatch => {
                "This version of libcouchbase cannot load the specified plugin"
            }
            CouchbaseError::InvalidHostFormat => {
                "Hostname specified for URI is in an invalid format"
            }
            CouchbaseError::InvalidChar => "Illegal character",
            CouchbaseError::InvalidDurabilityRequirement => {
                "Durability constraints requires more nodes/replicas than the cluster \
                 configuration allows. Durability constraints will never be satisfied"
            }
            CouchbaseError::DuplicateCommands => {
                "The same key was specified more than once in the command list"
            }
            CouchbaseError::NoMatchingServer => {
                "The node the request was mapped to does not exist in the current cluster map. \
                 This may be the result of a failover"
            }
            CouchbaseError::BadEnvironment => {
                "The value for an environment variable recognized by libcouchbase was specified in \
                 an incorrect format."
            }
            CouchbaseError::ClientBusy => "Busy. This is an internal error",
            CouchbaseError::InvalidUsername => {
                "The username must match the bucket name for data access"
            }
            CouchbaseError::ConfigCacheInvalid => {
                "The contents of the configuration cache file were invalid. Configuration will be \
                 fetched from the network"
            }
            CouchbaseError::SaslMechUnavailable => {
                "The requested SASL mechanism was not supported by the server. Either upgrade the \
                 server or change the mechanism requirements"
            }
            CouchbaseError::TooManyRedirects => "Maximum allowed number of redirects reached.",
            CouchbaseError::MapChanged => {
                "The cluster map has changed and this operation could not be completed or retried \
                 internally. Try this operation again"
            }
            CouchbaseError::IncompletePacket => "Incomplete packet was passed to forward function",
            CouchbaseError::ConnectionRefused => {
                "The remote host refused the connection. Is the service up?"
            }
            CouchbaseError::SocketShutdown => "The remote host closed the connection",
            CouchbaseError::ConnectionReset => {
                "The connection was forcibly reset by the remote host"
            }
            CouchbaseError::PortAllocationFailed => {
                "Could not assign a local port for this socket. For client sockets this means \
                 there are too many TCP sockets open"
            }
            CouchbaseError::FileDescriptorLimitReached => {
                "The system or process has reached its maximum number of file descriptors"
            }
            CouchbaseError::NetworkUnreachable => {
                "The remote host was unreachable - is your network OK?"
            }
            CouchbaseError::ControlCommandUnknown => "Control code passed was unrecognized",
            CouchbaseError::ControlCommandUnsupported => {
                "Invalid modifier for cntl operation (e.g. tried to read a write-only value)"
            }
            CouchbaseError::ControlCommandBadArgument => {
                "Argument passed to cntl was badly formatted"
            }
            CouchbaseError::EmptyKey => "An empty key was passed to an operation",
            CouchbaseError::EncryptionError => {
                "A generic error related to the SSL subsystem was encountered. Enable logging to \
                 see more details"
            }
            CouchbaseError::EncryptionCannotVerify => {
                "Client could not verify server's certificate"
            }
            CouchbaseError::InternalScheduleFailure => {
                "Internal error used for destroying unscheduled command data"
            }
            CouchbaseError::ClientFeatureUnavailable => {
                "The requested feature is not supported by the client, either because of settings \
                 in the configured instance, or because of options disabled at the time the \
                 library was compiled"
            }
            CouchbaseError::OptionsConflict => {
                "The operation structure contains conflicting options"
            }
            CouchbaseError::HttpError => "HTTP Operation failed. Inspect status code for details",
            CouchbaseError::DurabilityNoMutationTokens => {
                "The given item does not have a mutation token associated with it. this is either \
                 because fetching mutation tokens was not enabled, or you are trying to check on \
                 something not stored by this instance"
            }
            CouchbaseError::UnknownStatusCode => {
                "The server replied with an unrecognized status code. A newer version of this \
                 library may be able to decode it"
            }
            CouchbaseError::MutationLost => {
                "The given mutation has been permanently lost due to the node failing before \
                 replication"
            }
            CouchbaseError::SubdocPathDoesNotExist => "Sub-document path does not exist",
            CouchbaseError::SubdocPathMismatch => {
                "Type of element in sub-document path conflicts with type in document"
            }
            CouchbaseError::SubdocPathMalformed => "Malformed sub-document path",
            CouchbaseError::SubdocPathTooBig => "Sub-document contains too many components",
            CouchbaseError::SubdocExistingValueToeep => {
                "Existing document contains too many levels of nesting"
            }
            CouchbaseError::SubdocCannotInsert => "Subdocument operation would invalidate the JSON",
            CouchbaseError::SubdocExistingNotJson => "Existing document is not valid JSON",
            CouchbaseError::SubdocNumericValueTooLarge => "The existing numeric value is too large",
            CouchbaseError::SubdocBadDelta => {
                "The existing numeric value is too largeDelta must be numeric, within the 64 bit \
                 signed range, and non-zero"
            }
            CouchbaseError::SubdocPathExists => "The given path already exists in the document",
            CouchbaseError::SubdocMultiFailure => {
                "Could not execute one or more multi lookups or mutations"
            }
            CouchbaseError::SubdocValueTooDeepToInsert => "Value is too deep to insert",
            CouchbaseError::InvalidPacket => {
                "A badly formatted packet was sent to the server. Please report this in a bug"
            }
            CouchbaseError::SubdocEmptyPath => "Missing subdocument path",
            CouchbaseError::SubdocUnknownCommand => "Unknown subdocument command",
            CouchbaseError::NoCommandsSpecified => "No commands specified",
            CouchbaseError::QueryError => {
                "Query execution failed. Inspect raw response object for information"
            }
            CouchbaseError::GenericTmpError => "Generic temporary error received from server",
            CouchbaseError::GenericSubdocError => "Generic subdocument error received from server",
            CouchbaseError::GenericConstraintError => {
                "Generic constraint error received from server"
            }
            CouchbaseError::NameserverError => "Invalid reply received from nameserver",
            CouchbaseError::NotAuthorized => "Not authorized for operation",
            CouchbaseError::SubdocInvalidCombo => "An invalid combination of commands was specified",
            CouchbaseError::SubdocMultiPathFailure => "Specified key was successfully found, but one or more path operations failed",
            CouchbaseError::SubdocSuccessDeleted => "The operation completed successfully, but operated on a deleted document",
            CouchbaseError::SubdocXattrInvalidFlagCombo => "The combination of the subdoc flags for the xattrs is not valid",
            CouchbaseError::SubdocXattrInvalidKeyCombo => "Only a single xattr key may be accessed at the same time",
            CouchbaseError::SubdocXattrUnknownMacro => "The server has no knowledge of the requested macro",
            CouchbaseError::SubdocXattrUnknownVattr => "The server has no knowledge of the requested virtual xattr ",
            CouchbaseError:: SubdocXattrCantModifyVattr => "Virtual xattrs cannot be modified",
            CouchbaseError::SubdocMultiPathFailureDeleted => "Specified key was found as a deleted document, but one or more path operations failed",
            CouchbaseError::SubdocInvalidXattrOrder => "According to the spec all xattr commands should come first, followed by the commands for the document body",
            CouchbaseError::CollectionUnknown => "Unknown collection",
            CouchbaseError::CollectionNoManifest => "Operation attempted and requires that the collections manifest is set",
            CouchbaseError::CollectionCannotApplyManifest => "Bucket Manifest update could not be applied to vbucket(s)",
            CouchbaseError::CollectionManifestIsAhead => "Collections manifest of SDK is ahead of Server's",
            CouchbaseError::DurabilityInvalidLevel => "Invalid durability level was specified",
            CouchbaseError::DurabilityImpossible => "Given durability requirements are impossible to achieve",
            CouchbaseError::DurabilitySyncWriteInProgress => "There is a synchronous mutation pending for given key",
            CouchbaseError::DurabilitySyncWriteAmbiguous => "Synchronous mutation has not completed in the specified time and has ambiguous result",
            CouchbaseError::UnknownLibcouchbaseError(_) => {
                "Unknown error code from libcouchbase received"
            }
        }
    }
}

impl error::Error for CouchbaseError {
    fn description(&self) -> &str {
        self.as_str()
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl fmt::Display for CouchbaseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let msg = match &*self {
            CouchbaseError::FutureError(e) => format!("Future Error: {}", e),
            CouchbaseError::DecodingError(e) => format!("Decoding Error: {}", e),
            CouchbaseError::FutureCanceled(e) => format!("Future Canceled Error: {}", e),
            CouchbaseError::IoError(e) => format!("IO Error: {}", e),
            CouchbaseError::InternalChannelSendError(e) => format!("Channel Send Error: {}", e),
            err => format!("{}", err.as_str()),
        };
        write!(fmt, "{}", msg)
    }
}

impl convert::From<std::sync::mpsc::SendError<Box<dyn instance::request::InstanceRequest>>>
    for CouchbaseError
{
    fn from(
        err: std::sync::mpsc::SendError<Box<dyn instance::request::InstanceRequest>>,
    ) -> CouchbaseError {
        CouchbaseError::InternalChannelSendError(format!("{}", err))
    }
}

impl convert::From<std::io::Error> for CouchbaseError {
    fn from(err: std::io::Error) -> CouchbaseError {
        CouchbaseError::IoError(err)
    }
}

impl convert::From<serde_json::Error> for CouchbaseError {
    fn from(err: serde_json::Error) -> CouchbaseError {
        CouchbaseError::DecodingError(err)
    }
}

impl convert::From<futures::channel::oneshot::Canceled> for CouchbaseError {
    fn from(err: futures::channel::oneshot::Canceled) -> CouchbaseError {
        CouchbaseError::FutureCanceled(err)
    }
}

impl convert::From<CouchbaseError> for io::Error {
    fn from(err: CouchbaseError) -> Self {
        match err {
            CouchbaseError::KeyDoesNotExist => io::Error::new(io::ErrorKind::NotFound, err),
            CouchbaseError::KeyExists => io::Error::new(io::ErrorKind::AlreadyExists, err),
            CouchbaseError::TimedOut => io::Error::new(io::ErrorKind::TimedOut, err),
            CouchbaseError::ConnectionRefused => {
                io::Error::new(io::ErrorKind::ConnectionRefused, err)
            }
            CouchbaseError::ConnectionReset => io::Error::new(io::ErrorKind::ConnectionReset, err),
            _ => io::Error::new(io::ErrorKind::Other, err),
        }
    }
}

#[allow(non_upper_case_globals)]
impl convert::From<lcb_STATUS> for CouchbaseError {
    fn from(err: lcb_STATUS) -> Self {
        match err {
            lcb_STATUS_LCB_AUTH_ERROR => CouchbaseError::AuthFailed,
            lcb_STATUS_LCB_DELTA_BADVAL => CouchbaseError::DeltaBadval,
            lcb_STATUS_LCB_E2BIG => CouchbaseError::TooBig,
            lcb_STATUS_LCB_EBUSY => CouchbaseError::ServerBusy,
            lcb_STATUS_LCB_EINTERNAL => CouchbaseError::Internal,
            lcb_STATUS_LCB_EINVAL => CouchbaseError::InvalidValue,
            lcb_STATUS_LCB_ENOMEM => CouchbaseError::NoMemoryLeft,
            lcb_STATUS_LCB_ERANGE => CouchbaseError::InvalidRange,
            lcb_STATUS_LCB_ERROR => CouchbaseError::Generic,
            lcb_STATUS_LCB_ETMPFAIL => CouchbaseError::TemporaryFailure,
            lcb_STATUS_LCB_KEY_EEXISTS => CouchbaseError::KeyExists,
            lcb_STATUS_LCB_KEY_ENOENT => CouchbaseError::KeyDoesNotExist,
            lcb_STATUS_LCB_DLOPEN_FAILED => CouchbaseError::PluginLibraryNotFound,
            lcb_STATUS_LCB_DLSYM_FAILED => CouchbaseError::PluginInitializerNotFound,
            lcb_STATUS_LCB_NETWORK_ERROR => CouchbaseError::NetworkFailure,
            lcb_STATUS_LCB_NOT_MY_VBUCKET => CouchbaseError::NotMyVBucket,
            lcb_STATUS_LCB_NOT_STORED => CouchbaseError::NotStored,
            lcb_STATUS_LCB_NOT_SUPPORTED => CouchbaseError::NotSupported,
            lcb_STATUS_LCB_UNKNOWN_COMMAND => CouchbaseError::UnknownCommand,
            lcb_STATUS_LCB_UNKNOWN_HOST => CouchbaseError::UnknownHost,
            lcb_STATUS_LCB_PROTOCOL_ERROR => CouchbaseError::ProtocolError,
            lcb_STATUS_LCB_ETIMEDOUT => CouchbaseError::TimedOut,
            lcb_STATUS_LCB_CONNECT_ERROR => CouchbaseError::ConnectError,
            lcb_STATUS_LCB_BUCKET_ENOENT => CouchbaseError::BucketDoesNotExist,
            lcb_STATUS_LCB_CLIENT_ENOMEM => CouchbaseError::MemoryAllocationFailure,
            lcb_STATUS_LCB_CLIENT_ENOCONF => CouchbaseError::RequestNotScheduled,
            lcb_STATUS_LCB_EBADHANDLE => CouchbaseError::BadHandle,
            lcb_STATUS_LCB_SERVER_BUG => CouchbaseError::ServerBug,
            lcb_STATUS_LCB_PLUGIN_VERSION_MISMATCH => CouchbaseError::PluginVersionMismatch,
            lcb_STATUS_LCB_INVALID_HOST_FORMAT => CouchbaseError::InvalidHostFormat,
            lcb_STATUS_LCB_INVALID_CHAR => CouchbaseError::InvalidChar,
            lcb_STATUS_LCB_DURABILITY_ETOOMANY => CouchbaseError::InvalidDurabilityRequirement,
            lcb_STATUS_LCB_DUPLICATE_COMMANDS => CouchbaseError::DuplicateCommands,
            lcb_STATUS_LCB_NO_MATCHING_SERVER => CouchbaseError::NoMatchingServer,
            lcb_STATUS_LCB_BAD_ENVIRONMENT => CouchbaseError::BadEnvironment,
            lcb_STATUS_LCB_BUSY => CouchbaseError::ClientBusy,
            lcb_STATUS_LCB_INVALID_USERNAME => CouchbaseError::InvalidUsername,
            lcb_STATUS_LCB_CONFIG_CACHE_INVALID => CouchbaseError::ConfigCacheInvalid,
            lcb_STATUS_LCB_SASLMECH_UNAVAILABLE => CouchbaseError::SaslMechUnavailable,
            lcb_STATUS_LCB_TOO_MANY_REDIRECTS => CouchbaseError::TooManyRedirects,
            lcb_STATUS_LCB_MAP_CHANGED => CouchbaseError::MapChanged,
            lcb_STATUS_LCB_INCOMPLETE_PACKET => CouchbaseError::IncompletePacket,
            lcb_STATUS_LCB_ECONNREFUSED => CouchbaseError::ConnectionRefused,
            lcb_STATUS_LCB_ESOCKSHUTDOWN => CouchbaseError::SocketShutdown,
            lcb_STATUS_LCB_ECONNRESET => CouchbaseError::ConnectionReset,
            lcb_STATUS_LCB_ECANTGETPORT => CouchbaseError::PortAllocationFailed,
            lcb_STATUS_LCB_EFDLIMITREACHED => CouchbaseError::FileDescriptorLimitReached,
            lcb_STATUS_LCB_ENETUNREACH => CouchbaseError::NetworkUnreachable,
            lcb_STATUS_LCB_ECTL_UNKNOWN => CouchbaseError::ControlCommandUnknown,
            lcb_STATUS_LCB_ECTL_UNSUPPMODE => CouchbaseError::ControlCommandUnsupported,
            lcb_STATUS_LCB_ECTL_BADARG => CouchbaseError::ControlCommandBadArgument,
            lcb_STATUS_LCB_EMPTY_KEY => CouchbaseError::EmptyKey,
            lcb_STATUS_LCB_SSL_ERROR => CouchbaseError::EncryptionError,
            lcb_STATUS_LCB_SSL_CANTVERIFY => CouchbaseError::EncryptionCannotVerify,
            lcb_STATUS_LCB_SCHEDFAIL_INTERNAL => CouchbaseError::InternalScheduleFailure,
            lcb_STATUS_LCB_CLIENT_FEATURE_UNAVAILABLE => CouchbaseError::ClientFeatureUnavailable,
            lcb_STATUS_LCB_OPTIONS_CONFLICT => CouchbaseError::OptionsConflict,
            lcb_STATUS_LCB_HTTP_ERROR => CouchbaseError::HttpError,
            lcb_STATUS_LCB_DURABILITY_NO_MUTATION_TOKENS => {
                CouchbaseError::DurabilityNoMutationTokens
            }
            lcb_STATUS_LCB_UNKNOWN_MEMCACHED_ERROR => CouchbaseError::UnknownStatusCode,
            lcb_STATUS_LCB_MUTATION_LOST => CouchbaseError::MutationLost,
            lcb_STATUS_LCB_SUBDOC_PATH_ENOENT => CouchbaseError::SubdocPathDoesNotExist,
            lcb_STATUS_LCB_SUBDOC_PATH_MISMATCH => CouchbaseError::SubdocPathMismatch,
            lcb_STATUS_LCB_SUBDOC_PATH_EINVAL => CouchbaseError::SubdocPathMalformed,
            lcb_STATUS_LCB_SUBDOC_PATH_E2BIG => CouchbaseError::SubdocPathTooBig,
            lcb_STATUS_LCB_SUBDOC_DOC_E2DEEP => CouchbaseError::SubdocExistingValueToeep,
            lcb_STATUS_LCB_SUBDOC_VALUE_CANTINSERT => CouchbaseError::SubdocCannotInsert,
            lcb_STATUS_LCB_SUBDOC_DOC_NOTJSON => CouchbaseError::SubdocExistingNotJson,
            lcb_STATUS_LCB_SUBDOC_NUM_ERANGE => CouchbaseError::SubdocNumericValueTooLarge,
            lcb_STATUS_LCB_SUBDOC_BAD_DELTA => CouchbaseError::SubdocBadDelta,
            lcb_STATUS_LCB_SUBDOC_PATH_EEXISTS => CouchbaseError::SubdocPathExists,
            lcb_STATUS_LCB_SUBDOC_MULTI_FAILURE => CouchbaseError::SubdocMultiFailure,
            lcb_STATUS_LCB_SUBDOC_VALUE_E2DEEP => CouchbaseError::SubdocValueTooDeepToInsert,
            lcb_STATUS_LCB_EINVAL_MCD => CouchbaseError::InvalidPacket,
            lcb_STATUS_LCB_EMPTY_PATH => CouchbaseError::SubdocEmptyPath,
            lcb_STATUS_LCB_UNKNOWN_SDCMD => CouchbaseError::SubdocUnknownCommand,
            lcb_STATUS_LCB_ENO_COMMANDS => CouchbaseError::NoCommandsSpecified,
            lcb_STATUS_LCB_QUERY_ERROR => CouchbaseError::QueryError,
            lcb_STATUS_LCB_GENERIC_TMPERR => CouchbaseError::GenericTmpError,
            lcb_STATUS_LCB_GENERIC_SUBDOCERR => CouchbaseError::GenericSubdocError,
            lcb_STATUS_LCB_GENERIC_CONSTRAINT_ERR => CouchbaseError::GenericConstraintError,
            lcb_STATUS_LCB_NAMESERVER_ERROR => CouchbaseError::NameserverError,
            lcb_STATUS_LCB_NOT_AUTHORIZED => CouchbaseError::NotAuthorized,
            lcb_STATUS_LCB_MAX_ERROR => panic!("MAX_ERROR is internal!"),
            lcb_STATUS_LCB_SUCCESS => CouchbaseError::Success,
            lcb_STATUS_LCB_AUTH_CONTINUE => {
                panic!("AUTH_CONTINUE is internal and not to be exposed!")
            }
            lcb_STATUS_LCB_SUBDOC_INVALID_COMBO => CouchbaseError::SubdocInvalidCombo,
            lcb_STATUS_LCB_SUBDOC_MULTI_PATH_FAILURE => CouchbaseError::SubdocMultiPathFailure,
            lcb_STATUS_LCB_SUBDOC_SUCCESS_DELETED => CouchbaseError::SubdocSuccessDeleted,
            lcb_STATUS_LCB_SUBDOC_XATTR_INVALID_FLAG_COMBO => {
                CouchbaseError::SubdocXattrInvalidFlagCombo
            }
            lcb_STATUS_LCB_SUBDOC_XATTR_INVALID_KEY_COMBO => {
                CouchbaseError::SubdocXattrInvalidKeyCombo
            }
            lcb_STATUS_LCB_SUBDOC_XATTR_UNKNOWN_MACRO => CouchbaseError::SubdocXattrUnknownMacro,
            lcb_STATUS_LCB_SUBDOC_XATTR_UNKNOWN_VATTR => CouchbaseError::SubdocXattrUnknownVattr,
            lcb_STATUS_LCB_SUBDOC_XATTR_CANT_MODIFY_VATTR => {
                CouchbaseError::SubdocXattrCantModifyVattr
            }
            lcb_STATUS_LCB_SUBDOC_MULTI_PATH_FAILURE_DELETED => {
                CouchbaseError::SubdocMultiPathFailureDeleted
            }
            lcb_STATUS_LCB_SUBDOC_INVALID_XATTR_ORDER => CouchbaseError::SubdocInvalidXattrOrder,
            lcb_STATUS_LCB_COLLECTION_UNKNOWN => CouchbaseError::CollectionUnknown,
            lcb_STATUS_LCB_COLLECTION_NO_MANIFEST => CouchbaseError::CollectionNoManifest,
            lcb_STATUS_LCB_COLLECTION_CANNOT_APPLY_MANIFEST => {
                CouchbaseError::CollectionCannotApplyManifest
            }
            lcb_STATUS_LCB_COLLECTION_MANIFEST_IS_AHEAD => {
                CouchbaseError::CollectionManifestIsAhead
            }
            lcb_STATUS_LCB_DURABILITY_INVALID_LEVEL => CouchbaseError::DurabilityInvalidLevel,
            lcb_STATUS_LCB_DURABILITY_IMPOSSIBLE => CouchbaseError::DurabilityImpossible,
            lcb_STATUS_LCB_DURABILITY_SYNC_WRITE_IN_PROGRESS => {
                CouchbaseError::DurabilitySyncWriteInProgress
            }
            lcb_STATUS_LCB_DURABILITY_SYNC_WRITE_AMBIGUOUS => {
                CouchbaseError::DurabilitySyncWriteAmbiguous
            }
            e => CouchbaseError::UnknownLibcouchbaseError(e),
        }
    }
}
