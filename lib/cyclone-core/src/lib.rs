#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::module_name_repetitions
)]

mod action_run;
mod before;
mod canonical_command;
mod component_view;
mod kill_execution;
mod liveness;
mod management;
pub mod process;
mod progress;
mod readiness;
mod remote_shell;
mod request;
mod resolver_function;
mod schema_variant_definition;
mod sensitive_container;
mod validation;

pub use action_run::{
    ActionRunRequest,
    ActionRunResultSuccess,
    ResourceStatus,
};
pub use before::BeforeFunction;
pub use canonical_command::{
    CanonicalCommand,
    CanonicalCommandError,
};
pub use component_view::{
    ComponentKind,
    ComponentView,
    ComponentViewWithGeometry,
};
pub use kill_execution::KillExecutionRequest;
pub use liveness::{
    LivenessStatus,
    LivenessStatusParseError,
};
pub use management::{
    ManagementFuncStatus,
    ManagementRequest,
    ManagementResultSuccess,
};
pub use progress::{
    FunctionResult,
    FunctionResultFailure,
    FunctionResultFailureError,
    FunctionResultFailureErrorKind,
    Message,
    OutputStream,
    ProgressMessage,
};
pub use readiness::{
    ReadinessStatus,
    ReadinessStatusParseError,
};
pub use remote_shell::{
    RemoteShellConnectionInfo,
    RemoteShellRequest,
    RemoteShellResultSuccess,
    RemoteShellStatus,
};
pub use request::{
    CycloneRequest,
    CycloneRequestable,
};
pub use resolver_function::{
    ResolverFunctionComponent,
    ResolverFunctionRequest,
    ResolverFunctionResponseType,
    ResolverFunctionResultSuccess,
};
pub use schema_variant_definition::{
    SchemaVariantDefinitionRequest,
    SchemaVariantDefinitionResultSuccess,
};
pub use sensitive_container::SensitiveContainer;
pub use si_crypto::SensitiveStrings;
pub use validation::{
    ValidationRequest,
    ValidationResultSuccess,
};
