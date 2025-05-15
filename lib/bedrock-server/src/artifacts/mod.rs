use bedrock_core::{ArtifactStoreConfig, PublishResult};
use std::time::Instant;
use telemetry::tracing::{error, info};
use serde_json::Value;
use std::io::Error;
use std::process::Command;
use tokio::task;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::io::Write;
use std::time::Duration;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;
use serde::{Serialize, Deserialize};
use std::fs::create_dir_all;
use si_data_nats::{NatsClient, jetstream};
use tokio::time::timeout;
use async_nats::jetstream::stream::{
    Config, StorageType, RetentionPolicy, DiscardPolicy, Source
};
use async_nats::jetstream::consumer::pull::Config as PullConsumerConfig;
use async_nats::jetstream::consumer::{DeliverPolicy, AckPolicy, ReplayPolicy};
use futures::TryStreamExt;
use std::path::Path;
use std::process::Stdio;

use async_trait::async_trait;
use aws_sdk_s3::Client;
use aws_sdk_s3::types::SdkError;

#[derive(Serialize, Deserialize)]
struct JsonMessage {
    subject: String,
    headers: HashMap<String, String>,
    payload_hex: String,
}

pub async fn capture_nats(
    nats_client: &NatsClient,
    nats_streams: &[String],
    recording_id: &str,
) -> Result<(), String> {
    let js = jetstream::new(nats_client.clone());
    let base_dir = PathBuf::from("./recordings/rebaser/datasources")
        .join(recording_id)
        .join("nats_sequences");
    create_dir_all(&base_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    for source_stream in nats_streams {
        let mirror_stream_name = format!("{}_AUDIT", source_stream);
        let output_path = base_dir.join(format!("{}.sequence", source_stream));

        println!("📥 Dumping stream: {}", mirror_stream_name);

        let mut stream = js
            .get_stream(&mirror_stream_name)
            .await
            .map_err(|e| format!("Failed to get stream {}: {}", mirror_stream_name, e))?;

        let mut messages = vec![];
        let mut seq = 1u64;

        loop {
            // Refresh stream info to get the latest last_sequence
            let stream_info = stream.info().await
                .map_err(|e| format!("Failed to fetch stream info: {}", e))?;
            let last_seq = stream_info.state.last_sequence;
        
            if seq > last_seq {
                println!("Reached end of stream at seq {}. Ending capture.", last_seq);
                break;
            }
        
            match timeout(Duration::from_secs(2), stream.get_raw_message(seq)).await {
                Ok(Ok(msg)) => {
                    let mut headers = HashMap::new();
                    for (k, vs) in msg.headers.iter() {
                        if let Some(v) = vs.get(0) {
                            if let Ok(vstr) = std::str::from_utf8(v.as_ref()) {
                                headers.insert(k.to_string(), vstr.to_string());
                            }
                        }
                    }
        
                    messages.push(JsonMessage {
                        subject: msg.subject.to_string(),
                        headers,
                        payload_hex: hex::encode(&msg.payload),
                    });
        
                    seq += 1;
                }
                Ok(Err(e)) => {
                    return Err(format!("Failed to get message {}: {}", seq, e));
                }
                Err(_) => {
                    println!("Timeout waiting for message at seq {}. Ending capture.", seq);
                    break;
                }
            }
        }

        let mut file = TokioFile::create(&output_path)
            .await
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        let serialized = serde_json::to_string_pretty(&messages)
            .map_err(|e| format!("Failed to serialize messages: {}", e))?;

        file.write_all(serialized.as_bytes())
            .await
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        println!("Dumped {} messages to {}", messages.len(), output_path.display());

        js.delete_stream(&mirror_stream_name)
            .await
            .map_err(|e| format!("Failed to delete stream {}: {}", mirror_stream_name, e))?;

        println!("🗑 Deleted stream {}", mirror_stream_name);
    }

    Ok(())
}

pub fn resolve_local_sql_files(recording_id: &str) -> Result<Vec<String>, String> {
    let base_dir = PathBuf::from("./recordings/rebaser/datasources")
        .join(recording_id)
        .join("database_restore_points/start");

    if !base_dir.exists() {
        return Err(format!("Directory does not exist: {}", base_dir.display()));
    }

    let entries = fs::read_dir(&base_dir)
        .map_err(|e| format!("Failed to read directory {}: {}", base_dir.display(), e))?;

    let mut sql_paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "sql") {
            sql_paths.push(
                path.canonicalize()
                    .map_err(|e| format!("Failed to resolve absolute path: {}", e))?
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }

    if sql_paths.is_empty() {
        Err(format!(
            "No SQL restore files found in {}",
            base_dir.display()
        ))
    } else {
        println!(
            "✅ Found {} SQL restore file(s) in {}",
            sql_paths.len(),
            base_dir.display()
        );
        Ok(sql_paths)
    }
}

pub async fn resolve_remote_sql_files(
    recording_id: &str,
    config: &ArtifactStoreConfig,
) -> Result<Vec<String>, String> {
    let bucket = config
        .metadata
        .get("bucketName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid 'bucketName' in artifact config".to_string())?;

    let prefix = format!("{}/database_restore_points/start/", recording_id);

    let client = Client::new(&aws_config::load_from_env().await);

    let resp = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| format!("Failed to list S3 objects: {}", e))?;

    let objects = resp.contents().unwrap_or_default();
    if objects.is_empty() {
        return Err(format!("No SQL files found in S3 under prefix {}", prefix));
    }

    let mut sql_paths = Vec::new();
    for obj in objects {
        if let Some(key) = obj.key() {
            if key.ends_with(".sql") {
                let get_resp = client
                    .get_object()
                    .bucket(bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to get object {}: {}", key, e))?;

                let local_path = format!("/tmp/{}", key.replace("/", "_"));
                let mut file = File::create(&local_path)
                    .await
                    .map_err(|e| format!("Failed to create temp file: {}", e))?;
                let data = get_resp.body.collect().await.map_err(|e| e.to_string())?;
                file.write_all(&data.into_bytes())
                    .await
                    .map_err(|e| format!("Failed to write temp file: {}", e))?;

                sql_paths.push(local_path);
            }
        }
    }

    println!(
        "✅ Downloaded {} SQL restore file(s) from S3 prefix {}",
        sql_paths.len(),
        prefix
    );

    Ok(sql_paths)
}

pub async fn resolve_test(
    recording_id: &String,
    artifact_config: ArtifactStoreConfig,
) -> Result<Vec<String>, String> {
    match resolve_local_sql_files(recording_id) {
        Ok(paths) => Ok(paths),
        Err(local_err) => {
            println!("Local resolution failed: {}. Trying S3...", local_err);
            resolve_remote_sql_files(recording_id, &artifact_config).await
        }
    }
}

pub async fn configure_nats(
    nats_client: &NatsClient,
    nats_streams: &[String],
    recording_id: &str,
) -> Result<(), String> {
    let js = jetstream::new(nats_client.clone());

    for source_stream in nats_streams {
        let mirror_stream_name = format!("{}_AUDIT", source_stream);

        if js.get_stream(&mirror_stream_name).await.is_ok() {
            println!("🗑 Deleting existing stream: {}", mirror_stream_name);
            js.delete_stream(&mirror_stream_name)
                .await
                .map_err(|e| format!("Failed to delete stream {}: {}", mirror_stream_name, e))?;
        }

        let stream_config = Config {
            name: mirror_stream_name.clone(),
            description: Some(format!(
                "Passive copy of {} stream for recording ID {}",
                source_stream, recording_id
            )),
            storage: StorageType::File,
            retention: RetentionPolicy::Limits,
            discard: DiscardPolicy::Old,
            allow_direct: true,
            sources: Some(vec![Source {
                name: source_stream.clone(),
                ..Default::default()
            }]),
            duplicate_window: Duration::from_secs(0),
            ..Default::default()
        };

        js.create_stream(stream_config.clone())
            .await
            .map_err(|e| format!("Failed to create stream {}: {}", mirror_stream_name, e))?;

        let consumer_config = PullConsumerConfig {
            durable_name: Some(format!("{}_SINK", mirror_stream_name)),
            deliver_policy: DeliverPolicy::All,
            ack_policy: AckPolicy::None,
            replay_policy: ReplayPolicy::Instant,
            max_ack_pending: 1024,
            ..Default::default()
        };

        js.create_consumer_on_stream(consumer_config, mirror_stream_name.clone())
            .await
            .map_err(|e| format!("Failed to create consumer for {}: {}", mirror_stream_name, e))?;
    }

    Ok(())
}

/*

use futures::TryStreamExt;
let client = Client::connect_with_options(
    "localhost:4222",
    None,
    ConnectOptions::default(),
).await?;
let jetstream = si_data_nats::jetstream::new(client);
let mut names = jetstream.stream_names();
while let Some(stream) = names.try_next().await? {
    println!("stream: {}", stream);
}

*/

pub async fn clear_nats(nats_client: &NatsClient) -> Result<(), String> {
    let js = si_data_nats::jetstream::new(nats_client.clone());

    let mut names = js.stream_names();
    while let Ok(Some(stream_name)) = TryStreamExt::try_next(&mut names).await {
        println!("stream: {}", stream_name);
        if stream_name.ends_with("_AUDIT") {
            println!("Deleting stream: {}", stream_name);
            js.delete_stream(&stream_name)
                .await
                .map_err(|e| format!("Failed to delete stream {}: {}", stream_name, e))?;
        } else {
            println!("Purging stream: {}", stream_name);
            let stream = js
                .get_stream(&stream_name)
                .await
                .map_err(|e| format!("Failed to get stream {}: {}", stream_name, e))?;
            stream
                .purge()
                .await
                .map_err(|e| format!("Failed to purge stream {}: {}", stream_name, e))?;
        }
    }

    Ok(())
}

const DATABASE_DUMP_SCRIPT: &str = include_str!("../../scripts/dump-database.sh");

pub async fn dump_databases(databases: &[String], recording_id: &str, variant: &str) -> Result<(), String> {
    let script_path = std::env::temp_dir().join("dump-database.sh");

    {
        let mut file = fs::File::create(&script_path)
            .map_err(|e| format!("Failed to create script file: {}", e))?;
        file.write_all(DATABASE_DUMP_SCRIPT.as_bytes())
            .map_err(|e| format!("Failed to write script: {}", e))?;

        let mut perms = file.metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    for db in databases {
        let db = db.clone();
        let recording_id = recording_id.to_string();
        let variant = variant.to_string();
        let script_path = script_path.clone();

        task::spawn_blocking(move || {
            Command::new(&script_path)
                .arg(&db)
                .arg(&recording_id)
                .arg(&variant)
                .status()
                .map_err(|e| format!("Failed to run script: {}", e))
                .and_then(|status| {
                    if status.success() {
                        Ok(())
                    } else {
                        Err(format!("Script failed for {} with exit code: {}", db, status))
                    }
                })
        })
        .await
        .map_err(|e| format!("Join error: {}", e))??;
    }

    Ok(())
}

const DATABASE_PREPARE_SCRIPT: &str = include_str!("../../scripts/prepare-database.sh");

pub async fn prepare_databases(
    sql_paths: Vec<String>,
) -> Result<(), String> {
    let script_path = std::env::temp_dir().join("prepare-database.sh");

    {
        let mut file = fs::File::create(&script_path)
            .map_err(|e| format!("Failed to create script file: {}", e))?;

        file.write_all(DATABASE_PREPARE_SCRIPT.as_bytes())
            .map_err(|e| format!("Failed to write script: {}", e))?;

        let mut perms = file.metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    // Spawn a task for each SQL file to be restored
    for sql_path in sql_paths {
        let path = Path::new(&sql_path);
        let file_name = path.file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| format!("Invalid file name in path: {}", sql_path))?;

        let database_name = if file_name == "globals.sql" {
            "postgres".to_string()
        } else if file_name.ends_with("public_schema.sql") {
            file_name.strip_suffix("_public_schema.sql")
                .ok_or_else(|| format!("Invalid public_schema filename: {}", file_name))?
                .to_string()
        } else {
            return Err(format!("Unknown SQL filename pattern: {}", file_name));
        };

        let script_path = script_path.clone();
        let sql_path = sql_path.clone();

        task::spawn_blocking(move || {
            Command::new(&script_path)
                .arg(&sql_path)
                .arg(&database_name)
                .stdout(Stdio::null()) // suppress stdout as it's super chatty
                .stderr(Stdio::inherit())
                .status()
                .map_err(|e| format!("Failed to run script: {}", e))
                .and_then(|status| {
                    if status.success() {
                        Ok(())
                    } else {
                        Err(format!("Script failed for {} with exit code: {}", sql_path, status))
                    }
                })
        })
        .await
        .map_err(|e| format!("Join error: {}", e))??;
    }

    Ok(())
}

pub async fn publish_artifact(
    artifact_id: &str,
    _metadata: Value,
    config: &ArtifactStoreConfig,
) -> PublishResult {
    let start_time = Instant::now();

    info!(
        "Publishing artifact: id={}, variant={}",
        artifact_id, config.variant
    );

    let result: Result<(), String> = match config.variant.as_str() {
        "s3" => {
            let bucket = match config.metadata.get("bucketName").and_then(Value::as_str) {
                Some(b) => b,
                None => {
                    return PublishResult {
                        success: false,
                        message: "Missing `bucketName` in config metadata".into(),
                        duration_ms: None,
                        output: None,
                    };
                }
            };

            let object_key = format!("bedrock/tests/{artifact_id}/");

            info!("Uploading to S3 bucket={} key={}", bucket, object_key);

            fake_upload_to_s3(bucket, &object_key)
                .await
                .map_err(|e| e.to_string())
        }
        other => Err(format!("Unsupported artifact store variant: {}", other)),
    };

    let duration = start_time.elapsed().as_millis() as u64;

    match result {
        Ok(_) => PublishResult {
            success: true,
            message: "Artifact published successfully".into(),
            duration_ms: Some(duration),
            output: None,
        },
        Err(e) => {
            error!("Publishing failed: {}", e);
            PublishResult {
                success: false,
                message: e,
                duration_ms: Some(duration),
                output: None,
            }
        }
    }
}

async fn fake_upload_to_s3(_bucket: &str, _key: &str) -> Result<(), Error> {
    Ok(())
}