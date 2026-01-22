use std::{
    fmt,
    marker::PhantomData,
    time::Duration,
};

use async_nats::jetstream::stream::Stream as JsStream;

pub type KeyExtractorFn<K> = fn(&str, Option<&str>, &str) -> Option<K>;
pub type ConsumerNameFn<K> = fn(&K, &str) -> String;
pub type ConsumerFilterSubjectFn<K> = fn(Option<&str>, &K, &str) -> String;

pub struct FairSchedulingConfig<K> {
    pub(crate) service_name: String,
    pub(crate) tasks_stream: JsStream,
    pub(crate) requests_stream: JsStream,
    pub(crate) subject_prefix: Option<String>,
    pub(crate) key_extractor: KeyExtractorFn<K>,
    pub(crate) consumer_name_fn: ConsumerNameFn<K>,
    pub(crate) consumer_filter_subject_fn: ConsumerFilterSubjectFn<K>,
    pub(crate) inactive_threshold: Duration,
    pub(crate) task_listener_name: String,
    pub(crate) tasks_filter_subject: String,
    _key_marker: PhantomData<K>,
}

impl<K> fmt::Debug for FairSchedulingConfig<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FairSchedulingConfig")
            .field("service_name", &self.service_name)
            .field("subject_prefix", &self.subject_prefix)
            .field("inactive_threshold", &self.inactive_threshold)
            .field("task_listener_name", &self.task_listener_name)
            .field("tasks_filter_subject", &self.tasks_filter_subject)
            .finish_non_exhaustive()
    }
}

impl FairSchedulingConfig<String> {
    /// Creates a config for workspace-partitioned fair scheduling.
    ///
    /// Uses standard subject conventions where workspace_id is extracted from:
    /// - Tasks: `{prefix?}.{service}.tasks.{workspace_id}[.{rest}]`
    /// - Requests: `{prefix?}.{service}.requests.{workspace_id}.>`
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = FairSchedulingConfig::for_workspace_partitioning(
    ///     "veritech",
    ///     tasks_stream,
    ///     requests_stream,
    ///     prefix,
    /// );
    /// ```
    pub fn for_workspace_partitioning(
        service_name: impl Into<String>,
        tasks_stream: JsStream,
        requests_stream: JsStream,
        subject_prefix: Option<String>,
    ) -> Self {
        let service_name = service_name.into();

        let tasks_filter_subject =
            with_optional_prefix(subject_prefix.as_deref(), format!("{service_name}.tasks.*"));

        Self {
            service_name: service_name.clone(),
            tasks_stream,
            requests_stream,
            subject_prefix,
            key_extractor: default_workspace_key_extractor,
            consumer_name_fn: default_workspace_consumer_name,
            consumer_filter_subject_fn: default_workspace_filter_subject,
            inactive_threshold: Duration::from_secs(300),
            task_listener_name: format!("{service_name}-task-listener"),
            tasks_filter_subject,
            _key_marker: PhantomData,
        }
    }
}

fn with_optional_prefix(prefix: Option<&str>, base: impl AsRef<str>) -> String {
    match prefix {
        Some(p) => format!("{}.{}", p, base.as_ref()),
        None => base.as_ref().to_string(),
    }
}

/// Extracts workspace_id from position 2 (no prefix) or 3 (with prefix).
///
/// Works for subjects like: `{prefix?}.{service}.tasks.{workspace_id}[.{rest}]`
fn default_workspace_key_extractor(
    subject: &str,
    prefix: Option<&str>,
    _service_name: &str,
) -> Option<String> {
    let parts: Vec<&str> = subject.split('.').collect();
    let idx = if prefix.is_some() { 3 } else { 2 };
    parts.get(idx).map(|s| s.to_string())
}

/// Generates consumer name: `{service}-ws-{workspace_id}`
fn default_workspace_consumer_name(workspace_id: &String, service_name: &str) -> String {
    format!("{service_name}-ws-{workspace_id}")
}

/// Generates filter subject: `{prefix?}.{service}.requests.{workspace_id}.>`
fn default_workspace_filter_subject(
    prefix: Option<&str>,
    workspace_id: &String,
    service_name: &str,
) -> String {
    with_optional_prefix(prefix, format!("{service_name}.requests.{workspace_id}.>"))
}

#[cfg(test)]
mod tests {
    use super::{
        default_workspace_consumer_name,
        default_workspace_filter_subject,
        default_workspace_key_extractor,
    };

    #[test]
    fn test_workspace_key_extractor_no_prefix() {
        // Subject format: {service}.tasks.{workspace_id}
        let subject = "veritech.tasks.workspace-123";
        let result = default_workspace_key_extractor(subject, None, "veritech");

        assert_eq!(result, Some("workspace-123".to_string()));
    }

    #[test]
    fn test_workspace_key_extractor_with_prefix() {
        // Subject format: {prefix}.{service}.tasks.{workspace_id}
        let subject = "si.veritech.tasks.workspace-456";
        let result = default_workspace_key_extractor(subject, Some("si"), "veritech");

        assert_eq!(result, Some("workspace-456".to_string()));
    }

    #[test]
    fn test_workspace_key_extractor_with_trailing_parts() {
        // Subject format can have additional parts after workspace_id
        let subject = "veritech.tasks.workspace-789.extra.parts";
        let result = default_workspace_key_extractor(subject, None, "veritech");

        assert_eq!(result, Some("workspace-789".to_string()));
    }

    #[test]
    fn test_workspace_key_extractor_invalid_subject() {
        // Too few parts
        let subject = "veritech.tasks";
        let result = default_workspace_key_extractor(subject, None, "veritech");

        assert_eq!(result, None);
    }

    #[test]
    fn test_workspace_key_extractor_empty_workspace_id() {
        // Empty workspace ID should still be extracted (validation is elsewhere)
        let subject = "veritech.tasks..something";
        let result = default_workspace_key_extractor(subject, None, "veritech");

        assert_eq!(result, Some("".to_string()));
    }

    #[test]
    fn test_consumer_name_generation() {
        let workspace_id = "workspace-123";
        let service_name = "veritech";

        let result = default_workspace_consumer_name(&workspace_id.to_string(), service_name);

        assert_eq!(result, "veritech-ws-workspace-123");
    }

    #[test]
    fn test_consumer_name_with_special_characters() {
        let workspace_id = "workspace-test_123";
        let service_name = "my-service";

        let result = default_workspace_consumer_name(&workspace_id.to_string(), service_name);

        assert_eq!(result, "my-service-ws-workspace-test_123");
    }

    #[test]
    fn test_filter_subject_no_prefix() {
        let workspace_id = "workspace-123";
        let service_name = "veritech";

        let result =
            default_workspace_filter_subject(None, &workspace_id.to_string(), service_name);

        assert_eq!(result, "veritech.requests.workspace-123.>");
    }

    #[test]
    fn test_filter_subject_with_prefix() {
        let workspace_id = "workspace-456";
        let service_name = "veritech";

        let result =
            default_workspace_filter_subject(Some("si"), &workspace_id.to_string(), service_name);

        assert_eq!(result, "si.veritech.requests.workspace-456.>");
    }

    #[test]
    fn test_filter_subject_wildcard_suffix() {
        // Verify the wildcard suffix is correctly added
        let result = default_workspace_filter_subject(None, &"ws1".to_string(), "svc");

        assert!(
            result.ends_with(".>"),
            "Filter subject must end with wildcard"
        );
    }

    #[test]
    fn test_key_extractor_consistency_with_subject_helpers() {
        // Verify that the extractor can parse subjects generated by the helpers
        let workspace_id = "test-workspace";

        // Simulate what task_subject_for_workspace generates
        let subject = format!("veritech.tasks.{workspace_id}");

        // Should be extractable
        let extracted = default_workspace_key_extractor(&subject, None, "veritech");
        assert_eq!(extracted, Some(workspace_id.to_string()));
    }
}
