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

const NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT: &str = "veritech.fn.resolverfunction";
const NATS_VALIDATION_DEFAULT_SUBJECT: &str = "veritech.fn.validation";
const NATS_WORKFLOW_RESOLVE_DEFAULT_SUBJECT: &str = "veritech.fn.workflowresolve";
const NATS_COMMAND_RUN_DEFAULT_SUBJECT: &str = "veritech.fn.commandrun";
const NATS_CONCILIATION_DEFAULT_SUBJECT: &str = "veritech.fn.reconciliation";

pub const FINAL_MESSAGE_HEADER_KEY: &str = "X-Final-Message";

pub fn reply_mailbox_for_output(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.output")
}

pub fn reply_mailbox_for_result(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.result")
}

pub fn nats_resolver_function_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT)
}

pub fn nats_validation_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_VALIDATION_DEFAULT_SUBJECT)
}

pub fn nats_workflow_resolve_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_WORKFLOW_RESOLVE_DEFAULT_SUBJECT)
}

pub fn nats_command_run_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_COMMAND_RUN_DEFAULT_SUBJECT)
}

pub fn nats_reconciliation_subject(prefix: Option<&str>) -> String {
    nats_subject(prefix, NATS_CONCILIATION_DEFAULT_SUBJECT)
}

pub fn nats_subject(prefix: Option<&str>, suffix: impl AsRef<str>) -> String {
    let suffix = suffix.as_ref();
    match prefix {
        Some(prefix) => format!("{prefix}.{suffix}"),
        None => suffix.to_string(),
    }
}
