use nats_subscriber::Subscriber;
use si_data_nats::NatsClient;
use si_pool_noodle::{
    ActionRunRequest, ReconciliationRequest, ResolverFunctionRequest,
    SchemaVariantDefinitionRequest, ValidationRequest,
};
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
    ) -> Result<Subscriber<ResolverFunctionRequest>> {
        let subject = nats_resolver_function_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for resolver function requests"
        );
        Subscriber::create(subject)
            .queue_name("resolver")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }

    pub async fn validation(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscriber<ValidationRequest>> {
        let subject = nats_validation_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for validation requests"
        );
        Subscriber::create(subject)
            .queue_name("validation")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }

    pub async fn action_run(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscriber<ActionRunRequest>> {
        let subject = nats_action_run_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for command run requests"
        );
        Subscriber::create(subject)
            .queue_name("action")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }

    pub async fn reconciliation(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscriber<ReconciliationRequest>> {
        let subject = nats_reconciliation_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for reconciliation requests"
        );
        Subscriber::create(subject)
            .queue_name("reconciliation")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }

    pub async fn schema_variant_definition(
        nats: &NatsClient,
        subject_prefix: Option<&str>,
    ) -> Result<Subscriber<SchemaVariantDefinitionRequest>> {
        let subject = nats_schema_variant_definition_subject(subject_prefix);
        debug!(
            messaging.destination = &subject.as_str(),
            "subscribing for schema_variant_definition requests"
        );
        Subscriber::create(subject)
            .queue_name("schema_variant_definition")
            .check_for_reply_mailbox()
            .start(nats)
            .await
    }
}
