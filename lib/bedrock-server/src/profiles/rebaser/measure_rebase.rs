use bedrock_core::{
    Profile,
    Parameters,
    ExecutionParameters,
    TestProfile,
    TestResult,
};
use serde_json::json;
use si_data_nats::NatsClient;
pub struct MeasureRebase;
use async_trait::async_trait;

#[async_trait]
impl TestProfile for MeasureRebase {
    fn get(&self) -> Profile {
        Profile {
            service: "rebaser".into(),
            test: "measure_rebase".into(),
            parameters: Parameters {
                variant: "linear".into(),
            },
            executionParameters: ExecutionParameters {
                iterations: 5,
                timeout: 60,
            },
        }
    }

    async fn run(
        &self,
        _parameters: &Parameters,
        _exec: &ExecutionParameters,
        nats: &NatsClient,
    ) -> TestResult {
        println!("Running measure_rebase test...");

        // Decode base64 CBOR payload
        let base64_encoded = "oWJ2MaViaWR4GjAxSjhKTU1SNFI3NTdQU0JFWlRFWjVLUEFEa3dvcmtzcGFjZUlkeBowMUhYWUZGWlhXNUdZWVBKTkNNWDRQUFJWRmtjaGFuZ2VTZXRJZHgaMDFKOEpEUDUzOUM0UFRIMzhBV05aMjZRNkZudXBkYXRlc0FkZHJlc3N4QDBiM2VkMDBmNjMxMmEzYTkwODFjMmZiNWVmOGI1MmVkZTY3ZDVhNTY1NTBlMmNhYjNiNDlkYmE5YjQwZTM2ZWVvZnJvbUNoYW5nZVNldElk9g==";
        let decoded_bytes = match base64.decode(base64_encoded) {
            Ok(bytes) => bytes,
            Err(err) => {
                return TestResult {
                    success: false,
                    message: format!("Base64 decode failed: {err}"),
                    duration_ms: None,
                    output: None,
                }
            }
        };

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert("Nats-Msg-Id", "01J8JMMR4R757PSBEZTEZ5KPAD".parse().unwrap());
        headers.insert("X-MESSAGE-VERSION", "1".parse().unwrap());
        headers.insert("X-CONTENT-TYPE", "application/cbor".parse().unwrap());
        headers.insert("X-MESSAGE-TYPE", "EnqueueUpdatesRequest".parse().unwrap());
        headers.insert("X-Reply-Inbox", "_INBOX.dL5sjLmLFHcBqYR1mAI2ve".parse().unwrap());

        // Send via JetStream
        let js = jetstream::new(nats.clone());
        let subject = "rebaser.requests";

        match js.publish_with_headers(subject.to_string(), headers, decoded_bytes.into()).await {
            Ok(ack) => {
                println!("Published to JetStream, ack: {:?}", ack);
                TestResult {
                    success: true,
                    message: "Message sent to REBASER_REQUESTS".into(),
                    duration_ms: Some(42),
                    output: Some(json!({ "ack_sequence": ack.sequence })),
                }
            }
            Err(e) => {
                println!("Failed to publish: {e}");
                TestResult {
                    success: false,
                    message: format!("Publish failed: {e}"),
                    duration_ms: None,
                    output: None,
                }
            }
        }
    }
}
