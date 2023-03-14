use deadpool_cyclone::{
    CommandRunRequest, ResolverFunctionRequest, ValidationRequest, WorkflowResolveRequest,
};
use nats_subscriber::Subscription;
use nats_subscriber::SubscriptionConfig;
use nats_subscriber::SubscriptionConfigKeyOption;
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use veritech_core::{
    nats_command_run_subject, nats_resolver_function_subject, nats_validation_subject,
    nats_workflow_resolve_subject,
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
        Subscription::new(
            nats,
            SubscriptionConfig {
                subject,
                queue_name: Some("resolver".into()),
                final_message_header_key: SubscriptionConfigKeyOption::DoNotUseKey,
                check_for_reply_mailbox: true,
            },
        )
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
        Subscription::new(
            nats,
            SubscriptionConfig {
                subject,
                queue_name: Some("validation".into()),
                final_message_header_key: SubscriptionConfigKeyOption::DoNotUseKey,
                check_for_reply_mailbox: true,
            },
        )
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
        Subscription::new(
            nats,
            SubscriptionConfig {
                subject,
                queue_name: Some("workflow".into()),
                final_message_header_key: SubscriptionConfigKeyOption::DoNotUseKey,
                check_for_reply_mailbox: true,
            },
        )
        .await
    }

    pub async fn command_run(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscription<CommandRunRequest>> {
        let subject = nats_command_run_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for command resolve requests"
        );
        Subscription::new(
            nats,
            SubscriptionConfig {
                subject,
                queue_name: Some("command".into()),
                final_message_header_key: SubscriptionConfigKeyOption::DoNotUseKey,
                check_for_reply_mailbox: true,
            },
        )
        .await
    }
}
