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

use cyclone_core::{
    ActionRunRequest,
    KillExecutionRequest,
    ManagementRequest,
    ResolverFunctionRequest,
    SchemaVariantDefinitionRequest,
    ValidationRequest,
};
use si_data_nats::{
    Subject,
    async_nats,
    jetstream,
};

mod crypto;

pub use crypto::{
    VeritechValueDecryptError,
    VeritechValueEncryptError,
    decrypt_value_tree,
    encrypt_value_tree,
};

const NATS_WORK_QUEUE_STREAM_NAME: &str = "VERITECH_REQUESTS";
const NATS_WORK_QUEUE_STREAM_SUBJECTS: &[&str] = &["veritech.requests.>"];

const NATS_ACTION_RUN_DEFAULT_SUBJECT_SUFFIX: &str = "actionrun";
const NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT_SUFFIX: &str = "resolverfunction";
const NATS_SCHEMA_VARIANT_DEFINITION_DEFAULT_SUBJECT_SUFFIX: &str = "schemavariantdefinition";
const NATS_VALIDATION_DEFAULT_SUBJECT_SUFFIX: &str = "validation";
const NATS_MANAGEMENT_DEFAULT_SUBJECT_SUFFIX: &str = "management";

const NATS_KILL_EXECUTION_DEFAULT_SUBJECT: &str = "veritech.meta.killexecution";

const INCOMING_SUBJECT: &str = "veritech.requests.*.*.*";
const SUBJECT_PREFIX: &str = "veritech.requests";

pub const REPLY_INBOX_HEADER_NAME: &str = "X-Reply-Inbox";
pub const FINAL_MESSAGE_HEADER_KEY: &str = "X-Final-Message";

// NOTE(nick,fletcher): we can probably take this type formalization a step further, but this is
// essentially the "FuncRunId" from the "dal".
pub type ExecutionId = String;

pub async fn veritech_work_queue(
    context: &jetstream::Context,
    prefix: Option<&str>,
) -> Result<async_nats::jetstream::stream::Stream, async_nats::jetstream::context::CreateStreamError>
{
    let subjects: Vec<_> = NATS_WORK_QUEUE_STREAM_SUBJECTS
        .iter()
        .map(|suffix| nats_std::subject::prefixed(prefix, suffix).to_string())
        .collect();

    let stream = context
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: nats_std::jetstream::prefixed(prefix, NATS_WORK_QUEUE_STREAM_NAME),
            description: Some("Veritech work queue of requests".to_owned()),
            retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
            discard: async_nats::jetstream::stream::DiscardPolicy::New,
            allow_direct: true,
            subjects,
            ..Default::default()
        })
        .await?;

    Ok(stream)
}

pub fn reply_mailbox_for_output(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.output")
}

pub fn reply_mailbox_for_result(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.result")
}

pub trait GetNatsSubjectFor {
    fn subject_suffix(&self) -> &str;
    fn nats_subject(
        &self,
        prefix: Option<&str>,
        workspace_id: Option<&str>,
        change_set_id: Option<&str>,
    ) -> Subject {
        let subject_with_workspace_and_change_set = format!(
            "{SUBJECT_PREFIX}.{}.{}.{}",
            workspace_id.unwrap_or("NONE"),
            change_set_id.unwrap_or("NONE"),
            self.subject_suffix()
        );
        nats_std::subject::prefixed(prefix, subject_with_workspace_and_change_set)
    }
}

impl GetNatsSubjectFor for ActionRunRequest {
    fn subject_suffix(&self) -> &str {
        NATS_ACTION_RUN_DEFAULT_SUBJECT_SUFFIX
    }
}

impl GetNatsSubjectFor for KillExecutionRequest {
    fn subject_suffix(&self) -> &str {
        NATS_KILL_EXECUTION_DEFAULT_SUBJECT
    }

    fn nats_subject(
        &self,
        prefix: Option<&str>,
        _workspace_id: Option<&str>,
        _change_set_id: Option<&str>,
    ) -> Subject {
        nats_std::subject::prefixed(prefix, self.subject_suffix())
    }
}

impl GetNatsSubjectFor for ManagementRequest {
    fn subject_suffix(&self) -> &str {
        NATS_MANAGEMENT_DEFAULT_SUBJECT_SUFFIX
    }
}

impl GetNatsSubjectFor for ResolverFunctionRequest {
    fn subject_suffix(&self) -> &str {
        NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT_SUFFIX
    }
}

impl GetNatsSubjectFor for SchemaVariantDefinitionRequest {
    fn subject_suffix(&self) -> &str {
        NATS_SCHEMA_VARIANT_DEFINITION_DEFAULT_SUBJECT_SUFFIX
    }
}

impl GetNatsSubjectFor for ValidationRequest {
    fn subject_suffix(&self) -> &str {
        NATS_VALIDATION_DEFAULT_SUBJECT_SUFFIX
    }
}

#[derive(Debug, Clone)]
pub enum VeritechRequest {
    ActionRun(ActionRunRequest),
    KillExecution(KillExecutionRequest),
    Management(Box<ManagementRequest>),
    Resolver(ResolverFunctionRequest), // Resolvers are JsAttribute functions
    SchemaVariantDefinition(SchemaVariantDefinitionRequest),
    Validation(ValidationRequest),
}

#[derive(Debug, thiserror::Error)]
pub enum VeritechRequestError {
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("subject suffix {0} not a recognized veritech request type")]
    UnknownSubjectSuffix(String),
}

impl VeritechRequest {
    pub fn from_subject_and_payload(
        subject: &str,
        payload: &[u8],
    ) -> Result<Self, VeritechRequestError> {
        Ok(match subject {
            NATS_ACTION_RUN_DEFAULT_SUBJECT_SUFFIX => {
                Self::ActionRun(serde_json::from_slice(payload)?)
            }
            NATS_KILL_EXECUTION_DEFAULT_SUBJECT => {
                Self::KillExecution(serde_json::from_slice(payload)?)
            }
            NATS_MANAGEMENT_DEFAULT_SUBJECT_SUFFIX => {
                Self::Management(Box::new(serde_json::from_slice(payload)?))
            }
            NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT_SUFFIX => {
                Self::Resolver(serde_json::from_slice(payload)?)
            }
            NATS_SCHEMA_VARIANT_DEFINITION_DEFAULT_SUBJECT_SUFFIX => {
                Self::SchemaVariantDefinition(serde_json::from_slice(payload)?)
            }
            NATS_VALIDATION_DEFAULT_SUBJECT_SUFFIX => {
                Self::Validation(serde_json::from_slice(payload)?)
            }
            _ => {
                return Err(VeritechRequestError::UnknownSubjectSuffix(
                    subject.to_string(),
                ));
            }
        })
    }

    pub fn subject_suffix(&self) -> &str {
        match self {
            VeritechRequest::ActionRun(action_run_request) => action_run_request.subject_suffix(),
            VeritechRequest::KillExecution(kill_execution_request) => {
                kill_execution_request.subject_suffix()
            }
            VeritechRequest::Management(management_request) => management_request.subject_suffix(),
            VeritechRequest::Resolver(resolver_function_request) => {
                resolver_function_request.subject_suffix()
            }
            VeritechRequest::SchemaVariantDefinition(schema_variant_definition_request) => {
                schema_variant_definition_request.subject_suffix()
            }
            VeritechRequest::Validation(validation_request) => validation_request.subject_suffix(),
        }
    }

    pub fn execution_id(&self) -> &str {
        match self {
            VeritechRequest::ActionRun(action_run_request) => &action_run_request.execution_id,
            VeritechRequest::KillExecution(kill_execution_request) => {
                &kill_execution_request.execution_id
            }
            VeritechRequest::Management(management_request) => &management_request.execution_id,
            VeritechRequest::Resolver(resolver_function_request) => {
                &resolver_function_request.execution_id
            }
            VeritechRequest::SchemaVariantDefinition(schema_variant_definition_request) => {
                &schema_variant_definition_request.execution_id
            }
            VeritechRequest::Validation(validation_request) => &validation_request.execution_id,
        }
    }
}

#[inline]
pub fn incoming_subject(prefix: Option<&str>) -> Subject {
    nats_std::subject::prefixed(prefix, INCOMING_SUBJECT)
}
