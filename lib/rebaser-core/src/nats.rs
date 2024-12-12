use si_data_nats::{async_nats, jetstream};

use anyhow::Result;

const NATS_REBASER_REQUESTS_STREAM_NAME: &str = "REBASER_REQUESTS";
const NATS_REBASER_REQUESTS_STREAM_SUBJECTS: &[&str] = &["rebaser.requests.>"];
const NATS_REBASER_TASKS_STREAM_NAME: &str = "REBASER_TASKS";
const NATS_REBASER_TASKS_STREAM_SUBJECTS: &[&str] = &["rebaser.tasks.>"];

pub const NATS_HEADER_REPLY_INBOX_NAME: &str = "X-Reply-Inbox";

pub async fn rebaser_tasks_jetstream_stream(
    context: &jetstream::Context,
) -> Result<async_nats::jetstream::stream::Stream> {
    let prefix = context.metadata().subject_prefix();

    let subjects: Vec<_> = NATS_REBASER_TASKS_STREAM_SUBJECTS
        .iter()
        .map(|suffix| subject::nats_subject(prefix, suffix).to_string())
        .collect();

    let stream = context
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: nats_stream_name(prefix, NATS_REBASER_TASKS_STREAM_NAME),
            description: Some("Rebaser tasks".to_owned()),
            retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
            discard: async_nats::jetstream::stream::DiscardPolicy::New,
            max_messages_per_subject: 1,
            discard_new_per_subject: true,
            allow_direct: true,
            subjects,
            ..Default::default()
        })
        .await?;

    Ok(stream)
}

pub async fn rebaser_requests_jetstream_stream(
    context: &jetstream::Context,
) -> Result<async_nats::jetstream::stream::Stream> {
    let prefix = context.metadata().subject_prefix();

    let subjects: Vec<_> = NATS_REBASER_REQUESTS_STREAM_SUBJECTS
        .iter()
        .map(|suffix| subject::nats_subject(prefix, suffix).to_string())
        .collect();

    let stream = context
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: nats_stream_name(prefix, NATS_REBASER_REQUESTS_STREAM_NAME),
            description: Some("Rebaser requests".to_owned()),
            retention: async_nats::jetstream::stream::RetentionPolicy::Limits,
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

pub mod subject {
    use si_data_nats::Subject;

    const REQUESTS_SUBJECT_PREFIX: &str = "rebaser.requests";
    const TASKS_SUBJECT_PREFIX: &str = "rebaser.tasks";

    // Targetting subjects:
    // - `rebaser.tasks.$workspace_id.$change_set_id.$task_kind` (task for a change set)
    //
    // Possible future message subjects:
    // - `rebaser.tasks.$workspace_id.$task_kind` (task for a workspace)
    // - `rebaser.tasks.$task_kind` (task for an entire deployment)
    const TASKS_INCOMING_SUBJECT: &str = "rebaser.tasks.*.*.*";

    #[inline]
    pub fn tasks_incoming(prefix: Option<&str>) -> Subject {
        nats_subject(prefix, TASKS_INCOMING_SUBJECT)
    }

    #[inline]
    pub fn enqueue_updates_for_change_set(
        prefix: Option<&str>,
        workspace_id: &str,
        change_set_id: &str,
    ) -> Subject {
        nats_subject(
            prefix,
            format!(
                "{REQUESTS_SUBJECT_PREFIX}.{}.{}",
                workspace_id, change_set_id,
            ),
        )
    }

    #[inline]
    pub fn process_task_for_change_set(
        prefix: Option<&str>,
        workspace_id: &str,
        change_set_id: &str,
    ) -> Subject {
        nats_subject(
            prefix,
            format!(
                "{TASKS_SUBJECT_PREFIX}.{}.{}.process",
                workspace_id, change_set_id,
            ),
        )
    }

    pub(crate) fn nats_subject(prefix: Option<&str>, suffix: impl AsRef<str>) -> Subject {
        let suffix = suffix.as_ref();
        match prefix {
            Some(prefix) => Subject::from(format!("{prefix}.{suffix}")),
            None => Subject::from(suffix),
        }
    }
}
