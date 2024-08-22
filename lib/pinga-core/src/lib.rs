use si_data_nats::{async_nats, jetstream};

const NATS_WORK_QUEUE_STREAM_NAME: &str = "PINGA_JOBS";
const NATS_WORK_QUEUE_STREAM_SUBJECTS: &[&str] = &["pinga.jobs.>"];

pub const REPLY_INBOX_HEADER_NAME: &str = "X-Reply-Inbox";

pub async fn pinga_work_queue(
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
            description: Some("Pinga work queue of jobs ".to_owned()),
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

pub mod subject {
    use si_data_nats::Subject;

    const INCOMING_SUBJECT: &str = "pinga.jobs.*.*.*";
    const SUBJECT_PREFIX: &str = "pinga.jobs";

    #[inline]
    pub fn incoming(prefix: Option<&str>) -> Subject {
        nats_subject(prefix, INCOMING_SUBJECT)
    }

    #[inline]
    pub fn pinga_job(
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

    pub(crate) fn nats_subject(prefix: Option<&str>, suffix: impl AsRef<str>) -> Subject {
        let suffix = suffix.as_ref();
        match prefix {
            Some(prefix) => Subject::from(format!("{prefix}.{suffix}")),
            None => Subject::from(suffix),
        }
    }
}
