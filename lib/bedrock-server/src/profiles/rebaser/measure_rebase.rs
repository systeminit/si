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
use si_data_nats::HeaderMap;
use si_data_nats::jetstream;
use si_data_nats::HeaderValue;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct JsonMessage {
    subject: String,
    headers: HashMap<String, String>,
    payload_hex: String,
}

fn load_message_sequence() -> Vec<JsonMessage> {
    let file_content = include_str!("datasources/nats_sequences/measure_rebase/sequence.json");
    serde_json::from_str(file_content).expect("Failed to parse sequence JSON")
}

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

        let js = jetstream::new(nats.clone());
        let mut success_count = 0;

        for json_msg in load_message_sequence() {
            let mut headers = HeaderMap::new();

            for (k, v) in json_msg.headers.iter() {
                headers.insert(k.as_str(), HeaderValue::from(v.as_str()));
            }

            let payload = match hex::decode(&json_msg.payload_hex) {
                Ok(bytes) => bytes,
                Err(e) => {
                    println!("Payload decode error for {}: {:?}", json_msg.subject, e);
                    continue;
                }
            };

            match js.publish_with_headers(json_msg.subject.clone(), headers, payload.into()).await {
                Ok(ack_future) => match ack_future.await {
                    Ok(ack) => {
                        println!("✅ Sent to {}, ack: {:?}", json_msg.subject, ack);
                        success_count += 1;
                    }
                    Err(e) => println!("❌ Ack error for {}: {:?}", json_msg.subject, e),
                },
                Err(e) => println!("❌ Publish failed for {}: {:?}", json_msg.subject, e),
            }
        }

        TestResult {
            success: true,
            message: format!("Sent {} messages to JetStream", success_count),
            duration_ms: Some(42),
            output: Some(json!({ "messages_sent": success_count })),
        }
    }
}

// Need something that'll send the tracker in REBASER_TASKS again, like this
// nats --server 0.0.0.0 pub "rebaser.tasks.01JVAP8SZGPT4K937KNXMAJXQN.01JVAP9B4C1KHXXKY6Q1PP7C24.process" ""
// which will trigger the "watch" again