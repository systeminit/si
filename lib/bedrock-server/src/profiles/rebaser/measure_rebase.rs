use bedrock_core::{
    Profile,
    Parameters,
    ExecutionParameters,
    TestProfile,
    TestResult,
};
use std::path::PathBuf;
use serde_json::json;
use si_data_nats::NatsClient;
pub struct MeasureRebase;
use async_trait::async_trait;
use si_data_nats::HeaderMap;
use si_data_nats::jetstream;
use si_data_nats::HeaderValue;
use std::collections::HashMap;
use serde::Deserialize;
use tokio::time::{sleep, Duration};
use std::fs;

#[derive(Debug, Deserialize)]
struct JsonMessage {
    subject: String,
    headers: HashMap<String, String>,
    payload_hex: String,
}
fn load_message_sequence(variant: &str) -> Vec<JsonMessage> {
    let current_file = PathBuf::from(file!());
    let base_dir = current_file
        .parent().unwrap() // i.e. measure_rebase.rs
        .join("datasources")
        .join(variant)
        .join("nats_sequences")
        .join("measure_rebase")
        .join("sequence.json");

    println!("🔍 Loading sequence from: {}", base_dir.display());

    let file_content = fs::read_to_string(&base_dir)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", base_dir.display(), e));

    serde_json::from_str(&file_content)
        .expect("Failed to parse sequence JSON")
}

pub async fn send_rebaser_tracker_message(
    nats: &NatsClient,
    workspace_id: &str,
    changeset_id: &str,
) -> Result<(), String> {
    let subject = format!(
        "rebaser.tasks.{}.{}.process",
        workspace_id, changeset_id
    );

    let js = jetstream::new(nats.clone());
    let headers = HeaderMap::new(); 
    let payload: Vec<u8> = Vec::new();

    match js.publish_with_headers(subject.clone(), headers, payload.into()).await {
        Ok(ack_future) => {
            match ack_future.await {
                Ok(ack) => {
                    println!("📨 Sent tracker message to {}, ack: {:?}", subject, ack);
                    Ok(())
                }
                Err(e) => {
                    println!("❌ Ack error sending tracker message to {}: {:?}", subject, e);
                    Err(format!("Ack error sending tracker message to {}: {:?}", subject, e))
                }
            }
        }
        Err(e) => {
            println!("❌ Publish failed for {}: {:?}", subject, e);
            Err(format!("Publish error for {}: {:?}", subject, e))
        }
    }
}

#[async_trait]
impl TestProfile for MeasureRebase {
    fn get(&self) -> Profile {
        Profile {
            service: "rebaser".into(),
            test: "measure_rebase".into(),
            parameters: Parameters {
                variant: "linear".into(),
                workspaceId: "01JVAP8SZGPT4K937KNXMAJXQN".into(),
                changeSetId: "01JVEP2JRS4KC0P5W9F66XSVHC".into(),
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

        println!("Running measure_rebase variant {} for Workspace: {} / Change Set: {}", &_parameters.variant.to_string(), &_parameters.workspaceId.to_string(), &_parameters.changeSetId.to_string());

        let js = jetstream::new(nats.clone());
        let mut success_count = 0;

        dbg!("Sending message to start the rebaser off on it's work (REBASER_TASKS");

        if let Err(e) = send_rebaser_tracker_message(
            nats,
            &_parameters.workspaceId.to_string(),
            &_parameters.changeSetId.to_string()
        ).await {
            println!("Failed to send tracker message: {:?}", e);
        }

        for json_msg in load_message_sequence(&_parameters.variant) {

            // I broke the rebaser when I didn't do this, naughty John
            // Need to figure out why, it got in such a tangle
            sleep(Duration::from_millis(50)).await;
            
            let mut headers = HeaderMap::new();

            for (k, v) in json_msg.headers.iter() {
                if k != "Nats-Stream-Source" {
                    if k != "Nats-Stream-Source" {
                        headers.insert(k.as_str(), HeaderValue::from(v.as_str()));
                    } else if k == "X-Reply-Inbox" {
                        headers.insert(k.as_str(), HeaderValue::from("_INBOX.INCOMING_RESPONSES"));
                    } else {
                        headers.insert(k.as_str(), HeaderValue::from(v.as_str()));
                    }
                }
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