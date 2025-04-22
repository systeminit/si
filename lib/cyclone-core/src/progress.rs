use serde::{Deserialize, Serialize, de::DeserializeOwned};
use strum::Display;

/// A line of output, streamed from an executing function.
///
/// An instance of this type typically maps to a single line of output from a process--either on
/// standard output or standard error.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
pub struct OutputStream {
    /// The stream name.
    ///
    /// Typically set to `stdout`/`stderr` for process oriented output, but currently remains
    /// free-form.
    pub stream: String,
    /// An identifier for the execution of a particular function.
    ///
    /// Every function execution is given an indentifier, so that at least around execution time
    /// (i.e. possibly not forever and for all time), all output with the same execution ID can be
    /// reasonably assumed to be generated from the same function.
    pub execution_id: String,
    /// A "loglevel" tag for the output line.
    ///
    /// Level mimics the log level used in logging and tracing frameworks so level values such as
    /// `"info"`, `"debug"` are suitable but currently remains free-form.
    pub level: String,
    /// An option tag to help group together output.
    ///
    /// Group can be used upstream (i.e. a frontend UI) to group sets of `OutputStream`s together.
    pub group: Option<String>,
    /// The contents of the output line.
    pub message: String,
    /// A timestamp in seconds since UNIX epoch.
    ///
    /// The timestamp generated locally when the message was created.
    pub timestamp: u64,
}

/// A message produced as a function is executing.
///
/// A `ProgressMessage` is a way to track and follow how an execution is progressing. Such messages
/// can be produced up until a result is generated.
#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProgressMessage {
    /// A heartbeat message.
    ///
    /// This message can be used to signal "execution presence" (that is, the producer of such
    /// message is still alive and making progress).
    Heartbeat,
    /// An `OutputStream` message.
    OutputStream(OutputStream),
}

#[remain::sorted]
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Message<R> {
    Fail(Fail),
    Finish,
    Heartbeat,
    OutputStream(OutputStream),
    Result(FunctionResult<R>),
    Start,
}

impl<R> Message<R> {
    pub fn fail(message: impl Into<String>) -> Self {
        Self::Fail(Fail {
            message: message.into(),
        })
    }
}

impl<R> Message<R>
where
    R: DeserializeOwned,
{
    pub fn deserialize_from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl<R> Message<R>
where
    R: Serialize,
{
    pub fn serialize_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[remain::sorted]
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum FunctionResult<S> {
    Failure(FunctionResultFailure),
    Success(S),
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
pub struct FunctionResultFailure {
    execution_id: String,
    error: FunctionResultFailureError,
    timestamp: u64,
}

impl FunctionResultFailure {
    /// Creates a [`FunctionResultFailure`].
    pub fn new(
        execution_id: impl Into<String>,
        error: FunctionResultFailureError,
        timestamp: u64,
    ) -> Self {
        Self {
            execution_id: execution_id.into(),
            error,
            timestamp,
        }
    }

    /// This kind of [`FunctionResultFailure`] occurs when the veritech server encounters an error.
    pub fn new_for_veritech_server_error(
        execution_id: impl Into<String>,
        message: impl Into<String>,
        timestamp: u64,
    ) -> Self {
        Self {
            execution_id: execution_id.into(),
            error: FunctionResultFailureError {
                kind: FunctionResultFailureErrorKind::VeritechServer,
                message: message.into(),
            },
            timestamp,
        }
    }

    /// Returns a reference to the "execution_id".
    pub fn execution_id(&self) -> &String {
        &self.execution_id
    }

    /// Returns a reference to the "error".
    pub fn error(&self) -> &FunctionResultFailureError {
        &self.error
    }
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Display)]
pub enum FunctionResultFailureErrorKind {
    ActionFieldWrongType,
    InvalidReturnType,
    KilledExecution,
    UserCodeException(String),
    VeritechServer,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
pub struct FunctionResultFailureError {
    pub kind: FunctionResultFailureErrorKind,
    pub message: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Fail {
    pub message: String,
}
