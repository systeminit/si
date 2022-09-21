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

#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub use deadpool_cyclone::{instance::cyclone::LocalUdsInstance, Instance};
#[cfg(feature = "server")]
pub use server::{
    Config, ConfigBuilder, ConfigError, ConfigFile, CycloneSpec, CycloneStream, Server,
    ServerError, ShutdownHandle, StandardConfig, StandardConfigFile,
};

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub use client::{Client, ClientError, ClientResult};
#[cfg(feature = "client")]
pub use cyclone_core::{
    CodeGenerated, CodeGenerationRequest, CodeGenerationResultSuccess, CommandRunRequest,
    CommandRunResultSuccess, ComponentKind, ComponentView, EncryptionKey, EncryptionKeyError,
    FunctionResult, FunctionResultFailure, OutputStream, QualificationCheckComponent,
    QualificationCheckRequest, QualificationCheckResultSuccess, QualificationSubCheck,
    QualificationSubCheckStatus, ResolverFunctionComponent, ResolverFunctionRequest,
    ResolverFunctionResultSuccess, ResourceView, SensitiveContainer, SystemView,
    WorkflowResolveRequest, WorkflowResolveResultSuccess,
};

const NATS_QUALIFICATION_CHECK_DEFAULT_SUBJECT: &str = "veritech.fn.qualificationcheck";
const NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT: &str = "veritech.fn.resolverfunction";
const NATS_CODE_GENERATION_DEFAULT_SUBJECT: &str = "veritech.fn.codegeneration";
const NATS_WORKFLOW_RESOLVE_DEFAULT_SUBJECT: &str = "veritech.fn.workflowresolve";
const NATS_COMMAND_RUN_DEFAULT_SUBJECT: &str = "veritech.fn.commandrun";

pub(crate) const FINAL_MESSAGE_HEADER_KEY: &str = "X-Final-Message";

pub(crate) fn reply_mailbox_for_output(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.output")
}

pub(crate) fn reply_mailbox_for_result(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.result")
}

pub(crate) fn nats_qualification_check_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_QUALIFICATION_CHECK_DEFAULT_SUBJECT)
}

pub(crate) fn nats_resolver_function_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT)
}

pub(crate) fn nats_code_generation_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_CODE_GENERATION_DEFAULT_SUBJECT)
}

pub(crate) fn nats_workflow_resolve_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_WORKFLOW_RESOLVE_DEFAULT_SUBJECT)
}

pub(crate) fn nats_command_run_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_COMMAND_RUN_DEFAULT_SUBJECT)
}

pub(crate) fn nats_subject(prefix: Option<&str>, suffix: impl AsRef<str>) -> String {
    let suffix = suffix.as_ref();
    match prefix {
        Some(prefix) => format!("{prefix}.{suffix}"),
        None => suffix.to_string(),
    }
}
