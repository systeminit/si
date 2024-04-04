use std::time::Duration;

use si_data_nats::async_nats::jetstream;

pub const NATS_HEADER_DB_NAME: &str = "X-DB-NAME";
pub const NATS_HEADER_KEY: &str = "X-KEY";
pub const NATS_HEADER_INSTANCE_ID: &str = "X-INSTANCE-ID";

const NATS_EVENTS_STREAM_NAME: &str = "LAYERDB_EVENTS";

// Stream that covers messages across the following subjects:
// ```
// si.layerdb.events.$workspace_pk.$change_set_pk.$table_name.$event_kind
// ```
const NATS_EVENT_STREAM_SUBJECTS: &[&str] = &["si.layerdb.events.*.*.*.*"];

const NATS_ACTIVITIES_STREAM_NAME: &str = "LAYERDB_ACTIVITIES";
const NATS_ACTIVITIES_STREAM_SUBJECTS: &[&str] = &["si.layerdb.activities.>"];

const NATS_REBASER_REQUESTS_WORK_QUEUE_STREAM_NAME: &str = "REBASER_REQUESTS";

/// Returns a Jetstream Stream and creates it if it doesn't yet exist.
pub async fn layerdb_events_stream(
    context: &jetstream::Context,
    prefix: Option<&str>,
) -> Result<jetstream::stream::Stream, jetstream::context::CreateStreamError> {
    let subjects: Vec<_> = NATS_EVENT_STREAM_SUBJECTS
        .iter()
        .map(|suffix| subject::nats_subject(prefix, suffix).to_string())
        .collect();

    let stream = context
        .get_or_create_stream(jetstream::stream::Config {
            name: nats_stream_name(prefix, NATS_EVENTS_STREAM_NAME),
            description: Some("Layerdb events".to_owned()),
            subjects,
            retention: jetstream::stream::RetentionPolicy::Limits,
            discard: jetstream::stream::DiscardPolicy::Old,
            // TODO(fnichol): this likely needs tuning
            max_age: Duration::from_secs(60 * 60 * 6),
            no_ack: true,
            ..Default::default()
        })
        .await?;

    Ok(stream)
}

pub async fn layerdb_activities_stream(
    context: &jetstream::Context,
    prefix: Option<&str>,
) -> Result<jetstream::stream::Stream, jetstream::context::CreateStreamError> {
    let subjects: Vec<_> = NATS_ACTIVITIES_STREAM_SUBJECTS
        .iter()
        .map(|suffix| subject::nats_subject(prefix, suffix).to_string())
        .collect();

    let stream = context
        .get_or_create_stream(jetstream::stream::Config {
            name: nats_stream_name(prefix, NATS_ACTIVITIES_STREAM_NAME),
            description: Some("Layerdb activities".to_owned()),
            subjects,
            retention: jetstream::stream::RetentionPolicy::Limits,
            discard: jetstream::stream::DiscardPolicy::Old,
            // TODO(fnichol): this likely needs tuning
            max_age: Duration::from_secs(60 * 60 * 6),
            ..Default::default()
        })
        .await?;

    Ok(stream)
}

pub async fn rebaser_requests_work_queue_stream(
    context: &jetstream::Context,
    prefix: Option<&str>,
) -> Result<jetstream::stream::Stream, jetstream::context::CreateStreamError> {
    let requests_subject = subject::for_activity_discriminate(
        prefix,
        crate::activities::ActivityPayloadDiscriminants::RebaseRequest,
    );

    let source = jetstream::stream::Source {
        name: nats_stream_name(prefix, NATS_ACTIVITIES_STREAM_NAME),
        filter_subject: Some(requests_subject.to_string()),
        ..Default::default()
    };

    let stream = context
        .get_or_create_stream(jetstream::stream::Config {
            name: nats_stream_name(prefix, NATS_REBASER_REQUESTS_WORK_QUEUE_STREAM_NAME),
            description: Some("Rebaser requests work queue".to_owned()),
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            sources: Some(vec![source]),
            ..Default::default()
        })
        .await?;

    Ok(stream)
}

fn nats_stream_name(prefix: Option<&str>, suffix: impl AsRef<str>) -> String {
    let suffix = suffix.as_ref();

    match prefix {
        Some(prefix) => {
            let mut s = String::with_capacity(prefix.len() + 1 + suffix.len());
            s.push_str(prefix);
            s.push('_');
            s.push_str(suffix);
            s
        }
        None => suffix.to_owned(),
    }
}

pub mod subject {
    use si_data_nats::Subject;

    use crate::{
        activities::{Activity, ActivityPayloadDiscriminants},
        event::LayeredEvent,
    };

    const EVENTS_PREFIX: &str = "si.layerdb.events";
    const ACTIVITIES_PREFIX: &str = "si.layerdb.activities";

    pub fn for_event(prefix: Option<&str>, event: &LayeredEvent) -> Subject {
        // Cuts down on the amount of `String` allocations dealing with Ulids
        let mut buf = [0; ulid::ULID_LEN];

        // A string with enough capacity to avoid multiple reallocations
        let mut suffix = String::with_capacity(
            EVENTS_PREFIX.len() + (2 * ulid::ULID_LEN) + event.payload.db_name.len() + 4,
        );
        suffix.push_str(EVENTS_PREFIX);
        suffix.push('.');
        suffix.push_str(event.metadata.tenancy.workspace_pk.array_to_str(&mut buf));
        suffix.push('.');
        suffix.push_str(event.metadata.tenancy.change_set_id.array_to_str(&mut buf));
        suffix.push('.');
        suffix.push_str(&event.payload.db_name);
        suffix.push('.');
        suffix.push_str(event.event_kind.as_ref());

        nats_subject(prefix, suffix)
    }

    pub fn for_activity(prefix: Option<&str>, activity: &Activity) -> Subject {
        // Cuts down on the amount of `String` allocations dealing with Ulids
        let mut buf = [0; ulid::ULID_LEN];

        // A string with enough capacity to avoid multiple reallocations
        let mut suffix = String::with_capacity(
            ACTIVITIES_PREFIX.len()
                + (2 * ulid::ULID_LEN)
                + activity.payload.to_subject().len()
                + 3,
        );
        suffix.push_str(ACTIVITIES_PREFIX);
        suffix.push('.');
        suffix.push_str(
            activity
                .metadata
                .tenancy
                .workspace_pk
                .array_to_str(&mut buf),
        );
        suffix.push('.');
        suffix.push_str(
            activity
                .metadata
                .tenancy
                .change_set_id
                .array_to_str(&mut buf),
        );
        suffix.push('.');
        suffix.push_str(&activity.payload.to_subject());

        nats_subject(prefix, suffix)
    }

    pub fn for_activity_discriminate(
        prefix: Option<&str>,
        activity_payload_discriminate: ActivityPayloadDiscriminants,
    ) -> Subject {
        // Cuts down on the amount of `String` allocations dealing with Ulids
        let _buf = [0; ulid::ULID_LEN];

        // A string with enough capacity to avoid multiple reallocations
        let mut suffix = String::with_capacity(
            ACTIVITIES_PREFIX.len() + activity_payload_discriminate.to_subject().len() + 5,
        );
        suffix.push_str(ACTIVITIES_PREFIX);
        suffix.push('.');
        suffix.push('*');
        suffix.push('.');
        suffix.push('*');
        suffix.push('.');
        suffix.push_str(&activity_payload_discriminate.to_subject());

        nats_subject(prefix, suffix)
    }

    pub(crate) fn nats_subject(prefix: Option<&str>, suffix: impl AsRef<str>) -> Subject {
        let suffix = suffix.as_ref();

        match prefix {
            Some(prefix) => {
                let mut s = String::with_capacity(prefix.len() + 1 + suffix.len());
                s.push_str(prefix);
                s.push('.');
                s.push_str(suffix);

                Subject::from(s)
            }
            None => Subject::from(suffix),
        }
    }
}
