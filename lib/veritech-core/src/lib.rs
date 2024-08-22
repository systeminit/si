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

use si_data_nats::{async_nats, jetstream};

mod crypto;

pub use crypto::{
    decrypt_value_tree, encrypt_value_tree, VeritechValueDecryptError, VeritechValueEncryptError,
};

const NATS_WORK_QUEUE_STREAM_NAME: &str = "VERITECH_REQUESTS";
const NATS_WORK_QUEUE_STREAM_SUBJECTS: &[&str] = &["veritech.requests.>"];

pub const NATS_ACTION_RUN_DEFAULT_SUBJECT_SUFFIX: &str = "actionrun";
pub const NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT_SUFFIX: &str = "resolverfunction";
pub const NATS_SCHEMA_VARIANT_DEFINITION_DEFAULT_SUBJECT_SUFFIX: &str = "schemavariantdefinition";
pub const NATS_VALIDATION_DEFAULT_SUBJECT_SUFFIX: &str = "validation";

pub const NATS_KILL_EXECUTION_DEFAULT_SUBJECT: &str = "veritech.meta.killexecution";

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
        .map(|suffix| subject::nats_subject(prefix, suffix).to_string())
        .collect();

    let stream = context
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: nats_stream_name(prefix, NATS_WORK_QUEUE_STREAM_NAME),
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

fn nats_stream_name(prefix: Option<&str>, suffix: impl AsRef<str>) -> String {
    let suffix = suffix.as_ref();

    match prefix {
        Some(prefix) => format!("{prefix}_{suffix}"),
        None => suffix.to_owned(),
    }
}

pub fn reply_mailbox_for_output(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.output")
}

pub fn reply_mailbox_for_result(reply_mailbox: &str) -> String {
    format!("{reply_mailbox}.result")
}

pub mod subject {
    use si_data_nats::Subject;

    use crate::NATS_KILL_EXECUTION_DEFAULT_SUBJECT;

    const INCOMING_SUBJECT: &str = "veritech.requests.*.*.*";
    const SUBJECT_PREFIX: &str = "veritech.requests";

    #[inline]
    pub fn incoming(prefix: Option<&str>) -> Subject {
        nats_subject(prefix, INCOMING_SUBJECT)
    }

    #[inline]
    pub fn veritech_request(
        prefix: Option<&str>,
        workspace_id: &str,
        change_set_id: &str,
        kind: &str,
    ) -> Subject {
        nats_subject(
            prefix,
            format!(
                "{SUBJECT_PREFIX}.{}.{}.{}",
                workspace_id, change_set_id, kind,
            ),
        )
    }

    #[inline]
    pub fn veritech_kill_request(prefix: Option<&str>) -> Subject {
        nats_subject(prefix, NATS_KILL_EXECUTION_DEFAULT_SUBJECT)
    }

    pub(crate) fn nats_subject(prefix: Option<&str>, suffix: impl AsRef<str>) -> Subject {
        let suffix = suffix.as_ref();
        match prefix {
            Some(prefix) => Subject::from(format!("{prefix}.{suffix}")),
            None => Subject::from(suffix),
        }
    }
}
