use deadpool_cyclone::{
    ActionRunRequest, ReconciliationRequest, ResolverFunctionRequest,
    SchemaVariantDefinitionRequest, ValidationRequest,
};
use nats_subscriber::Subscription;
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use veritech_core::{
    nats_action_run_subject, nats_reconciliation_subject, nats_resolver_function_subject,
    nats_schema_variant_definition_subject, nats_validation_subject,
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

    pub async fn action_run(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscription<ActionRunRequest>> {
        let subject = nats_action_run_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for command run requests"
        );
        Subscription::create(subject)
            .queue_name("action")
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

    pub async fn schema_variant_definition(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscription<SchemaVariantDefinitionRequest>> {
        let subject = nats_schema_variant_definition_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for schema_variant_definition requests"
        );
        Subscription::create(subject)
            .queue_name("schema_variant_definition")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }
}
