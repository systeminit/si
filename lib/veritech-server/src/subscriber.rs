use deadpool_cyclone::{
    CommandRunRequest, ReconciliationRequest, ResolverFunctionRequest, ValidationRequest,
    WorkflowResolveRequest,
};
use nats_subscriber::Subscription;
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use veritech_core::{
    nats_command_run_subject, nats_reconciliation_subject, nats_resolver_function_subject,
    nats_validation_subject, nats_workflow_resolve_subject,
};

type Result<T> = std::result::Result<T, nats_subscriber::SubscriberError>;

pub struct FunctionSubscriber;

impl FunctionSubscriber {
    pub async fn resolver_function(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscription<ResolverFunctionRequest>> {
        let subject = nats_resolver_function_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for resolver function requests"
        );
        Subscription::create(subject)
            .queue_name("resolver")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }

    pub async fn validation(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscription<ValidationRequest>> {
        let subject = nats_validation_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for validation requests"
        );
        Subscription::create(subject)
            .queue_name("validation")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }

    pub async fn workflow_resolve(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscription<WorkflowResolveRequest>> {
        let subject = nats_workflow_resolve_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for workflow resolve requests"
        );
        Subscription::create(subject)
            .queue_name("workflow")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }

    pub async fn command_run(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscription<CommandRunRequest>> {
        let subject = nats_command_run_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for command run requests"
        );
        Subscription::create(subject)
            .queue_name("command")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }

    pub async fn reconciliation(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscription<ReconciliationRequest>> {
        let subject = nats_reconciliation_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for reconciliation requests"
        );
        Subscription::create(subject)
            .queue_name("reconciliation")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }
}
