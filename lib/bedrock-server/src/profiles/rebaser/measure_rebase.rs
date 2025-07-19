use std::path::PathBuf;

use bedrock_core::{
    ExecutionParameters,
    Parameters,
    Profile,
    TestProfile,
    TestResult,
};
use serde_json::json;
use si_data_nats::NatsClient;
pub struct MeasureRebase;
use std::{
    collections::HashMap,
    fs,
    io::Cursor,
};

use async_trait::async_trait;
use ciborium::de::from_reader;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use si_data_nats::{
    HeaderMap,
    HeaderValue,
    jetstream,
};
use tokio::time::Duration;
use ulid::Ulid;

#[derive(Debug, Deserialize)]
struct JsonMessage {
    subject: String,
    headers: HashMap<String, String>,
    payload_hex: String,
}

fn load_message_sequence(recording_id: &str) -> Vec<JsonMessage> {
    let base_dir = PathBuf::from("./recordings/datasources")
        .join(recording_id)
        .join("nats_sequences");

    if !base_dir.exists() {
        panic!("NATS sequence directory not found: {}", base_dir.display());
    }

    let entries = fs::read_dir(&base_dir)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", base_dir.display(), e));

    let mut layerdb_messages = Vec::new();
    let mut other_messages = Vec::new();

    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("Failed to read entry: {e}"));
        let path = entry.path();

        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension == "sequence" {
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                println!("Loading NATS sequence from: {}", path.display());

                let content = fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

                let parsed: Vec<JsonMessage> = serde_json::from_str(&content).unwrap_or_else(|e| {
                    panic!("Failed to parse JSON from {}: {}", path.display(), e)
                });

                if file_name.starts_with("LAYERDB_EVENTS") {
                    layerdb_messages.extend(parsed);
                } else {
                    other_messages.extend(parsed);
                }
            }
        }
    }

    if layerdb_messages.is_empty() && other_messages.is_empty() {
        panic!(
            "No NATS sequence .sequence file found in {}",
            base_dir.display()
        );
    }

    // PREPEND layerdb messages to other messages
    // TODO(johnrwatson): this is poor show, I should have a sequence in the params which
    // tells the system which order to play them back in, or I could do it by timestamp
    // that's also an option. Forgive me, I'm only human.
    layerdb_messages.extend(other_messages);
    layerdb_messages
}

pub async fn reissue_rebaser_tracker_message(
    nats: &NatsClient,
    workspace_id: &str,
    changeset_id: &str,
) -> Result<(), String> {
    let subject = format!("rebaser.tasks.{workspace_id}.{changeset_id}.process");

    let js = jetstream::new(nats.clone());
    let headers = HeaderMap::new();
    let payload: Vec<u8> = Vec::new();

    // Publish the new message and return mapped error
    match js
        .publish_with_headers(subject.clone(), headers, payload.into())
        .await
    {
        Ok(ack_future) => {
            if let Err(e) = ack_future.await {
                Err(format!("Ack error: {:?}", e.kind()))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!("Publish error: {:?}", e.kind())),
    }
}

#[async_trait]
impl TestProfile for MeasureRebase {
    fn get(&self) -> Profile {
        // TODO(johnrwatson): Future work here, as below:
        // These need to be a dynamic lookup from both the remote artifact store and local
        Profile {
            recording_id: "example".to_string(),
            parameters: Parameters {
                workspace_id: "your-workspace-id".into(),
                change_set_id: "your-change-set-id".into(),
            },
            execution_parameters: ExecutionParameters {
                iterations: 5,
                timeout: 60,
            },
        }
    }

    async fn run(
        &self,
        recording_id: &str,
        _parameters: &Parameters,
        _exec: &ExecutionParameters,
        nats: &NatsClient,
    ) -> TestResult {
        println!(
            "Running recording_id {} / Workspace ID: {} / Change Set ID: {}",
            &recording_id, &_parameters.workspace_id, &_parameters.change_set_id
        );

        let js = jetstream::new(nats.clone());
        let messages = load_message_sequence(recording_id);
        let mut success_count = 0;

        for (i, json_msg) in messages.into_iter().enumerate() {
            let reply_subject = format!("_INBOX.INCOMING_RESPONSES.{i}");
            let mut headers = HeaderMap::new();
            let new_ulid = Ulid::new().to_string();

            for (k, v) in json_msg.headers.iter() {
                if k != "Nats-Stream-Source" {
                    if k == "X-Reply-Inbox" {
                        headers.insert(k.clone(), HeaderValue::from(reply_subject.as_str()));
                    } else if k == "Nats-Msg-Id" {
                        headers.insert(k.clone(), HeaderValue::from(new_ulid.as_str()));
                    } else {
                        headers.insert(k.clone(), HeaderValue::from(v.as_str()));
                    }
                }
            }

            let payload = match hex::decode(&json_msg.payload_hex) {
                Ok(bytes) => {
                    let mut cursor = Cursor::new(&bytes);
                    match from_reader::<JsonValue, _>(&mut cursor) {
                        Ok(val) => println!(
                            "Decoded CBOR JSON:\n{}",
                            serde_json::to_string_pretty(&val).unwrap_or_default()
                        ),
                        Err(_) => {
                            println!(
                                "Failed to parse CBOR payload, logging raw hex:\n{}",
                                json_msg.payload_hex
                            );
                        }
                    }
                    bytes
                }
                Err(e) => {
                    println!(
                        "Payload decode error for {}: {:?}\nRaw payload (hex): {}",
                        json_msg.subject, e, json_msg.payload_hex
                    );
                    json_msg.payload_hex.as_bytes().to_vec()
                }
            };

            // Subscribe to the unique reply inbox
            let mut sub = nats
                .subscribe(reply_subject.clone())
                .await
                .expect("Failed to subscribe to reply inbox");

            // Send the message
            match js
                .publish_with_headers(json_msg.subject.clone(), headers, payload.into())
                .await
            {
                Ok(ack_future) => match ack_future.await {
                    Ok(ack) => {
                        println!("Sent to {}, ack: {:?}", json_msg.subject, ack);
                        success_count += 1;

                        // Skip waiting if this is an layerdb event message
                        if json_msg.subject.starts_with("si.layerdb.events") {
                            continue;
                        } else {
                            // Always issue the tracker message for each rebase request, swallow error silently
                            let _ = reissue_rebaser_tracker_message(
                                nats,
                                &_parameters.workspace_id,
                                &_parameters.change_set_id,
                            )
                            .await;
                        }

                        // Await single response
                        let timeout = tokio::time::sleep(Duration::from_secs(15));
                        tokio::pin!(timeout);

                        let response = tokio::select! {
                            msg = sub.next() => msg,
                            _ = &mut timeout => {
                                println!("Timeout waiting for response on {reply_subject}");
                                return TestResult {
                                    success: false,
                                    message: format!("Timeout waiting for response from service on subject {reply_subject}"),
                                    duration_ms: None,
                                    output: Some(json!({
                                        "failed_message_index": i,
                                        "error": format!("Timeout at waiting for message {}: {}", i, reply_subject)
                                    })),
                                };
                            }
                        };

                        match response {
                            Some(msg) => {
                                println!(
                                    "Got response on {} ({} bytes)",
                                    msg.subject(),
                                    msg.payload().len()
                                );
                                let mut cursor = Cursor::new(msg.payload());
                                match from_reader::<JsonValue, _>(&mut cursor) {
                                    Ok(json_val) => {
                                        println!(
                                            "Response JSON:\n{}",
                                            serde_json::to_string_pretty(&json_val)
                                                .unwrap_or_default()
                                        );

                                        // Check for error: json["v1"]["status"]["error"]["message"]
                                        if let Some(error_msg) = json_val
                                            .get("v1")
                                            .and_then(|v1| v1.get("status"))
                                            .and_then(|status| status.get("error"))
                                            .and_then(|err| err.get("message"))
                                            .and_then(|msg| msg.as_str())
                                        {
                                            println!("Early exit: error in response: {error_msg}");
                                            return TestResult {
                                                success: false,
                                                message: format!(
                                                    "Error at message {i}: {error_msg}"
                                                ),
                                                duration_ms: None,
                                                output: Some(json!({
                                                    "failed_message_index": i,
                                                    "error": error_msg
                                                })),
                                            };
                                        }
                                    }
                                    Err(e) => {
                                        println!("Failed to parse response CBOR: {e:?}");
                                    }
                                }
                            }
                            None => println!("No response received on {reply_subject}"),
                        }
                    }
                    Err(e) => println!("Ack error for {}: {:?}", json_msg.subject, e),
                },
                Err(e) => println!("Publish failed for {}: {:?}", json_msg.subject, e),
            }
        }

        TestResult {
            success: true,
            message: format!("Sent and received {success_count} message-response pairs"),
            duration_ms: Some(42),
            output: Some(json!({ "message_response_pairs": success_count })),
        }
    }
}
