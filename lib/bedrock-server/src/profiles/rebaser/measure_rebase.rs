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
use futures::StreamExt;
use async_trait::async_trait;
use ciborium::de::from_reader;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use si_data_nats::{
    HeaderMap,
    HeaderValue,
    jetstream,
};
use tokio::time::{Duration, sleep};
use ulid::Ulid;

#[derive(Debug, Deserialize)]
struct JsonMessage {
    subject: String,
    headers: HashMap<String, String>,
    payload_hex: String,
}

pub fn load_message_sequence(recording_id: &str) -> Vec<JsonMessage> {
    // Construct path to: ./recordings/rebaser/datasources/{recording_id}/nats_sequences/
    let base_dir = PathBuf::from("./recordings/rebaser/datasources")
        .join(recording_id)
        .join("nats_sequences");

    if !base_dir.exists() {
        panic!("NATS sequence directory not found: {}", base_dir.display());
    }

    let entries = fs::read_dir(&base_dir)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", base_dir.display(), e));

    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("Failed to read entry: {}", e));
        let path = entry.path();

        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension == "json" || extension == "sequence" {
                println!("🔍 Loading NATS sequence from: {}", path.display());

                let content = fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

                return serde_json::from_str(&content)
                    .unwrap_or_else(|e| panic!("Failed to parse JSON from {}: {}", path.display(), e));
            }
        }
    }

    panic!("No NATS sequence .json or .sequence file found in {}", base_dir.display());
}

pub async fn reissue_rebaser_tracker_message(
    nats: &NatsClient,
    workspace_id: &str,
    changeset_id: &str,
) -> Result<(), String> {
    let subject = format!("rebaser.tasks.{}.{}.process", workspace_id, changeset_id);
    let stream_name = "REBASER_TASKS";

    let js = jetstream::new(nats.clone());
    let headers = HeaderMap::new();
    let payload: Vec<u8> = Vec::new();

    // Step 1: Load the stream
    match js.get_stream(stream_name).await {
        Ok(stream) => {
            // Step 2: Try to get the last message on the subject
            match stream.get_last_raw_message_by_subject(&subject).await {
                Ok(msg) => {
                    let seq = msg.sequence;
                    println!("Found existing message with seq={} on {}, deleting...", seq, subject);
                    if let Err(e) = stream.delete_message(seq).await {
                        println!("Failed to delete message seq={} from {}: {:?}", seq, stream_name, e);
                    }
                }
                Err(e) => {
                    println!("ℹNo existing message found on {}: {:?}", subject, e);
                    // OK to continue
                }
            }
        }
        Err(e) => {
            println!("Could not load JetStream stream {}: {:?}", stream_name, e);
            return Err(format!("Stream access error: {}", e));
        }
    }

    // Step 3: Publish the new message
    match js.publish_with_headers(subject.clone(), headers, payload.into()).await {
        Ok(ack_future) => match ack_future.await {
            Ok(ack) => {
                println!("Published tracker message to {}, ack: {:?}", subject, ack);
                Ok(())
            }
            Err(e) => {
                println!("Ack error: {:?}", e);
                Err(format!("Ack error sending tracker message to {}: {:?}", subject, e))
            }
        },
        Err(e) => {
            println!("Publish failed: {:?}", e);
            Err(format!("Publish error for {}: {:?}", subject, e))
        }
    }
}

#[async_trait]
impl TestProfile for MeasureRebase {
    fn get(&self) -> Profile {
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
        recording_id: &String,
        _parameters: &Parameters,
        _exec: &ExecutionParameters,
        nats: &NatsClient,
    ) -> TestResult {
        println!(
            "Running recording_id {} / Workspace ID: {} / Change Set ID: {}",
            &recording_id,            
            &_parameters.workspace_id,
            &_parameters.change_set_id
        );

        let js = jetstream::new(nats.clone());
        let messages = load_message_sequence(&recording_id);
        let mut success_count = 0;

        for (i, json_msg) in messages.into_iter().enumerate() {
            sleep(Duration::from_millis(250)).await;

            // Always issue the tracker message for each rebase request + swallo
            // if it already exists
            if let Err(e) = reissue_rebaser_tracker_message(
                nats,
                &_parameters.workspace_id,
                &_parameters.change_set_id,
            )
            .await
            {
                println!("Failed to send tracker message, swallowing {:?}", e);
            }

            let reply_subject = format!("_INBOX.INCOMING_RESPONSES.{}", i);
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
                        Ok(val) => println!("Decoded CBOR JSON:\n{}", serde_json::to_string_pretty(&val).unwrap_or_default()),
                        Err(e) => println!("Failed to parse CBOR payload: {:?}", e),
                    }
                    bytes
                }
                Err(e) => {
                    println!("Payload decode error for {}: {:?}", json_msg.subject, e);
                    continue;
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

                        // Await single response
                        let timeout = tokio::time::sleep(Duration::from_secs(15));
                        tokio::pin!(timeout);

                        let response = tokio::select! {
                            msg = sub.next() => msg,
                            _ = &mut timeout => {
                                println!("Timeout waiting for response on {}", reply_subject);
                                return TestResult {
                                    success: false,
                                    message: format!("Timeout waiting for response from service"),
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
                                println!("Got response on {} ({} bytes)", msg.subject(), msg.payload().len());
                                let mut cursor = Cursor::new(msg.payload());
                                match from_reader::<JsonValue, _>(&mut cursor) {
                                    Ok(json_val) => {
                                        println!("Response JSON:\n{}", serde_json::to_string_pretty(&json_val).unwrap_or_default());
                        
                                        // Check for error: json["v1"]["status"]["error"]["message"]
                                        if let Some(error_msg) = json_val
                                            .get("v1")
                                            .and_then(|v1| v1.get("status"))
                                            .and_then(|status| status.get("error"))
                                            .and_then(|err| err.get("message"))
                                            .and_then(|msg| msg.as_str())
                                        {
                                            println!("Early exit: error in response: {}", error_msg);
                                            return TestResult {
                                                success: false,
                                                message: format!("Error at message {}: {}", i, error_msg),
                                                duration_ms: None,
                                                output: Some(json!({
                                                    "failed_message_index": i,
                                                    "error": error_msg
                                                })),
                                            };
                                        }
                                    }
                                    Err(e) => {
                                        println!("Failed to parse response CBOR: {:?}", e);
                                    }
                                }
                            }
                            None => println!("No response received on {}", reply_subject),
                        }
                    }
                    Err(e) => println!("Ack error for {}: {:?}", json_msg.subject, e),
                },
                Err(e) => println!("Publish failed for {}: {:?}", json_msg.subject, e),
            }
        }

        TestResult {
            success: true,
            message: format!("Sent and received {} message-response pairs", success_count),
            duration_ms: Some(42),
            output: Some(json!({ "message_response_pairs": success_count })),
        }
    }
}
