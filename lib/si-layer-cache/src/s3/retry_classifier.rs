//! Retry classifier for S3 NoSuchKey errors to handle eventual consistency.
//!
//! This classifier integrates with AWS SDK's retry mechanism to automatically
//! retry GetObject operations that return NoSuchKey errors. This is necessary
//! because S3 has eventual consistency - an object that was just written may
//! not be immediately visible in subsequent read operations.
//!
//! The classifier runs with higher priority than default classifiers, allowing
//! it to override their `NoActionIndicated` decision for NoSuchKey errors
//! specifically, without affecting retry behavior for other error types.

use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_smithy_runtime_api::client::{
    interceptors::context::InterceptorContext,
    retries::classifiers::{
        ClassifyRetry,
        RetryAction,
        RetryClassifierPriority,
    },
};

/// Retry classifier that marks S3 NoSuchKey errors as retryable.
///
/// This classifier detects `GetObjectError::NoSuchKey` errors and marks them
/// as transient errors, triggering the SDK's retry mechanism. This handles
/// the eventual consistency case where an object was just written but is not
/// yet visible.
///
/// The classifier uses type-safe matching on GetObjectError::NoSuchKey.
///
/// # Priority
///
/// This classifier runs with higher priority (AFTER default classifiers),
/// allowing it to override their `NoActionIndicated` decision for NoSuchKey
/// specifically. It returns `NoActionIndicated` for all other errors,
/// deferring to defaults.
#[derive(Debug, Clone, Default)]
pub struct NoSuchKeyRetryClassifier;

impl NoSuchKeyRetryClassifier {
    /// Create a new NoSuchKeyRetryClassifier
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl ClassifyRetry for NoSuchKeyRetryClassifier {
    fn classify_retry(&self, ctx: &InterceptorContext) -> RetryAction {
        // Check for a result
        let output_or_error = ctx.output_or_error();

        // Check for an error
        let error = match output_or_error {
            Some(Ok(_)) | None => return RetryAction::NoActionIndicated,
            Some(Err(err)) => err,
        };

        if let Some(operation_error) = error.as_operation_error() {
            if let Some(get_object_error) = operation_error.downcast_ref::<GetObjectError>() {
                if matches!(get_object_error, GetObjectError::NoSuchKey(_)) {
                    return RetryAction::transient_error();
                }
            }
        }

        // Not a NoSuchKey error - let other classifiers handle it
        RetryAction::NoActionIndicated
    }

    fn name(&self) -> &'static str {
        "NoSuchKeyRetryClassifier"
    }

    fn priority(&self) -> RetryClassifierPriority {
        // Run AFTER default classifiers so we can override their NoActionIndicated
        // decision for NoSuchKey errors specifically
        RetryClassifierPriority::run_after(
            RetryClassifierPriority::modeled_as_retryable_classifier(),
        )
    }
}

#[cfg(test)]
mod tests {
    use aws_smithy_runtime_api::client::{
        interceptors::context::{
            Error,
            Input,
        },
        orchestrator::OrchestratorError,
    };

    use super::*;

    // Helper to create an InterceptorContext with a specific error
    fn create_context_with_error(error: GetObjectError) -> InterceptorContext {
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(error))));
        ctx
    }

    #[test]
    fn test_nosuchkey_is_retryable() {
        let classifier = NoSuchKeyRetryClassifier::new();
        let error = GetObjectError::NoSuchKey(
            aws_sdk_s3::types::error::NoSuchKey::builder()
                .message("The specified key does not exist.")
                .build(),
        );
        let ctx = create_context_with_error(error);

        let result = classifier.classify_retry(&ctx);

        assert!(
            matches!(result, RetryAction::RetryIndicated(_)),
            "NoSuchKey should be retryable, got: {result:?}",
        );
    }

    #[test]
    fn test_other_errors_not_retryable() {
        let classifier = NoSuchKeyRetryClassifier::new();
        let error = GetObjectError::InvalidObjectState(
            aws_sdk_s3::types::error::InvalidObjectState::builder()
                .message("Object is archived")
                .build(),
        );
        let ctx = create_context_with_error(error);

        let result = classifier.classify_retry(&ctx);

        assert!(
            matches!(result, RetryAction::NoActionIndicated),
            "InvalidObjectState should return NoActionIndicated, got: {result:?}",
        );
    }
}
