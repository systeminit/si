use si_data_nats::{
    async_nats,
    jetstream,
};

const NATS_REQUESTS_STREAM_NAME: &str = "EDDA_REQUESTS";
const NATS_REQUESTS_STREAM_SUBJECTS: &[&str] = &["edda.requests.>"];
const NATS_TASKS_STREAM_NAME: &str = "EDDA_TASKS";
const NATS_TASKS_STREAM_SUBJECTS: &[&str] = &["edda.tasks.>"];

pub const NATS_HEADER_REPLY_INBOX_NAME: &str = "X-Reply-Inbox";

pub async fn edda_tasks_jetstream_stream(
    context: &jetstream::Context,
) -> Result<async_nats::jetstream::stream::Stream, async_nats::jetstream::context::CreateStreamError>
{
    let prefix = context.metadata().subject_prefix();

    let subjects: Vec<_> = NATS_TASKS_STREAM_SUBJECTS
        .iter()
        .map(|suffix| nats_std::subject::prefixed(prefix, suffix).to_string())
        .collect();

    let stream = context
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: nats_std::jetstream::prefixed(prefix, NATS_TASKS_STREAM_NAME),
            description: Some("Edda tasks".to_owned()),
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

pub async fn edda_requests_jetstream_stream(
    context: &jetstream::Context,
) -> Result<async_nats::jetstream::stream::Stream, async_nats::jetstream::context::CreateStreamError>
{
    let prefix = context.metadata().subject_prefix();

    let subjects: Vec<_> = NATS_REQUESTS_STREAM_SUBJECTS
        .iter()
        .map(|suffix| nats_std::subject::prefixed(prefix, suffix).to_string())
        .collect();

    let stream = context
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: nats_std::jetstream::prefixed(prefix, NATS_REQUESTS_STREAM_NAME),
            description: Some("Edda requests".to_owned()),
            retention: async_nats::jetstream::stream::RetentionPolicy::Limits,
            discard: async_nats::jetstream::stream::DiscardPolicy::New,
            allow_direct: true,
            subjects,
            ..Default::default()
        })
        .await?;

    Ok(stream)
}

pub mod subject {
    use si_data_nats::Subject;

    const REQUESTS_SUBJECT_PREFIX: &str = "edda.requests";
    const TASKS_SUBJECT_PREFIX: &str = "edda.tasks";

    // Targetting subjects:
    // - `edda.tasks.$workspace_id.$change_set_id.$task_kind` (task for a change set)
    //
    // Possible future message subjects:
    // - `edda.tasks.$workspace_id.$task_kind` (task for a workspace)
    // - `edda.tasks.$task_kind` (task for an entire deployment)
    const TASKS_INCOMING_SUBJECT: &str = "edda.tasks.*.*.*";

    #[inline]
    pub fn tasks_incoming(prefix: Option<&str>) -> Subject {
        nats_std::subject::prefixed(prefix, TASKS_INCOMING_SUBJECT)
    }

    #[inline]
    pub fn request_for_change_set(
        prefix: Option<&str>,
        workspace_id: &str,
        change_set_id: &str,
    ) -> Subject {
        nats_std::subject::prefixed(
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
        nats_std::subject::prefixed(
            prefix,
            format!(
                "{TASKS_SUBJECT_PREFIX}.{}.{}.process",
                workspace_id, change_set_id,
            ),
        )
    }
}
