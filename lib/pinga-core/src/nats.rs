use si_data_nats::{
    async_nats,
    jetstream,
};

const NATS_WORK_QUEUE_STREAM_NAME: &str = "PINGA_JOBS";
const NATS_WORK_QUEUE_STREAM_SUBJECTS: &[&str] = &["pinga.jobs.>"];

pub async fn pinga_work_queue(
    context: &jetstream::Context,
) -> Result<async_nats::jetstream::stream::Stream, async_nats::jetstream::context::CreateStreamError>
{
    let prefix = context.metadata().subject_prefix();

    let subjects: Vec<_> = NATS_WORK_QUEUE_STREAM_SUBJECTS
        .iter()
        .map(|suffix| nats_std::subject::prefixed(prefix, suffix).to_string())
        .collect();

    let stream = context
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: nats_std::jetstream::prefixed(prefix, NATS_WORK_QUEUE_STREAM_NAME),
            description: Some("Pinga work queue of jobs".to_owned()),
            retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
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

    const INCOMING_SUBJECT: &str = "pinga.jobs.*.*.*";
    const SUBJECT_PREFIX: &str = "pinga.jobs";

    #[inline]
    pub fn incoming(prefix: Option<&str>) -> Subject {
        nats_std::subject::prefixed(prefix, INCOMING_SUBJECT)
    }

    #[inline]
    pub fn pinga_job(
        prefix: Option<&str>,
        workspace_id: &str,
        change_set_id: &str,
        args: &str,
    ) -> Subject {
        nats_std::subject::prefixed(
            prefix,
            format!("{SUBJECT_PREFIX}.{workspace_id}.{change_set_id}.{args}",),
        )
    }
}
