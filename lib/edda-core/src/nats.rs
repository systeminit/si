use si_data_nats::{
    async_nats,
    jetstream,
};

const NATS_REQUESTS_STREAM_NAME: &str = "EDDA_REQUESTS";
const NATS_REQUESTS_STREAM_SUBJECTS: &[&str] = &["edda.requests.>"];
const NATS_TASKS_STREAM_NAME: &str = "EDDA_TASKS";
const NATS_TASKS_STREAM_SUBJECTS: &[&str] = &["edda.tasks.>"];

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
    use strum::AsRefStr;

    #[remain::sorted]
    #[derive(AsRefStr, Debug, PartialEq)]
    #[strum(serialize_all = "snake_case")]
    pub enum Scope {
        ChangeSet,
        Deployment,
        Workspace,
    }

    // Tasks subjects:
    // - `edda.tasks.$scope.[$scope_segments...]` (general pattern, grouped under a `$scope`)
    // - `edda.tasks.deployment.$task_kind` (task for an entire deployment)
    // - `edda.tasks.workspace.$workspace_id.$task_kind` (task for a workspace)
    // - `edda.tasks.change_set.$workspace_id.$change_set_id.$task_kind` (task for a change set)
    const TASKS_SUBJECT_PREFIX: &str = "edda.tasks";
    const TASKS_INCOMING_SUBJECT: &str = "edda.tasks.>";

    // Requests subjects:
    // - `edda.requests.$scope.[$scope_segments...]` (general pattern, grouped under a `$scope`)
    // - `edda.requests.deployment` (request for an entire deployment)
    // - `edda.requests.workspace.$workspace_id` (request for a workspace)
    // - `edda.requests.change_set.$workspace_id.$change_set_id` (request for a change set)
    const REQUESTS_SUBJECT_PREFIX: &str = "edda.requests";

    // Updates subjects:
    // - `edda.updates.$scope.[$scope_segments...]` (general pattern, grouped under a `$scope`)
    // - `edda.updates.deployment.$kind` (update for an entire deployment)
    // - `edda.updates.workspace.$workspace_id.$kind` (update for a workspace)
    // - `edda.updates.change_set.$workspace_id.$change_set_id.$kind` (update for a change set)
    const UPDATES_SUBJECT_PREFIX: &str = "edda.updates";

    #[inline]
    pub fn tasks_incoming(prefix: Option<&str>) -> Subject {
        nats_std::subject::prefixed(prefix, TASKS_INCOMING_SUBJECT)
    }

    #[inline]
    pub fn request_for_deployment(prefix: Option<&str>) -> Subject {
        nats_std::subject::prefixed(
            prefix,
            format!("{REQUESTS_SUBJECT_PREFIX}.{}", Scope::Deployment.as_ref()),
        )
    }

    #[inline]
    pub fn request_for_workspace(prefix: Option<&str>, workspace_id: &str) -> Subject {
        nats_std::subject::prefixed(
            prefix,
            format!(
                "{REQUESTS_SUBJECT_PREFIX}.{}.{workspace_id}",
                Scope::Workspace.as_ref()
            ),
        )
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
                "{REQUESTS_SUBJECT_PREFIX}.{}.{workspace_id}.{change_set_id}",
                Scope::ChangeSet.as_ref()
            ),
        )
    }

    #[inline]
    pub fn process_task_for_deployment(prefix: Option<&str>) -> Subject {
        nats_std::subject::prefixed(
            prefix,
            format!(
                "{TASKS_SUBJECT_PREFIX}.{}.process",
                Scope::Deployment.as_ref()
            ),
        )
    }

    #[inline]
    pub fn process_task_for_workspace(prefix: Option<&str>, workspace_id: &str) -> Subject {
        nats_std::subject::prefixed(
            prefix,
            format!(
                "{TASKS_SUBJECT_PREFIX}.{}.{workspace_id}.process",
                Scope::Workspace.as_ref()
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
                "{TASKS_SUBJECT_PREFIX}.{}.{workspace_id}.{change_set_id}.process",
                Scope::ChangeSet.as_ref()
            ),
        )
    }

    #[inline]
    pub fn all_workspace_updates_for_all_workspaces(prefix: Option<&str>) -> Subject {
        nats_std::subject::prefixed(
            prefix,
            format!("{UPDATES_SUBJECT_PREFIX}.{}.*.*", Scope::Workspace.as_ref()),
        )
    }

    #[inline]
    pub fn all_workspace_updates_for_workspace(
        prefix: Option<&str>,
        workspace_id: &str,
    ) -> Subject {
        nats_std::subject::prefixed(
            prefix,
            format!(
                "{UPDATES_SUBJECT_PREFIX}.{}.{workspace_id}.*",
                Scope::Workspace.as_ref()
            ),
        )
    }

    #[inline]
    pub fn workspace_update_for(prefix: Option<&str>, workspace_id: &str, kind: &str) -> Subject {
        nats_std::subject::prefixed(
            prefix,
            format!(
                "{UPDATES_SUBJECT_PREFIX}.{}.{workspace_id}.{kind}",
                Scope::Workspace.as_ref()
            ),
        )
    }
}
