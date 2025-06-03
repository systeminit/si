use std::{
    collections::HashMap,
    fs::{
        self,
        create_dir_all,
    },
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{
        Path,
        PathBuf,
    },
    process::{
        Command,
        Stdio,
    },
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            AtomicUsize,
            Ordering,
        },
    },
    time::{
        Duration,
        Instant,
    },
};

use async_nats::jetstream::{
    consumer::{
        AckPolicy,
        DeliverPolicy,
        ReplayPolicy,
        pull::Config as PullConsumerConfig,
    },
    stream::{
        Config,
        DiscardPolicy,
        RetentionPolicy,
        Source,
        StorageType,
    },
};
use aws_credential_types::Credentials as AwsCredentials;
use aws_sdk_s3::{
    Client,
    config::{
        Builder as ConfigBuilder,
        Region as AwsRegion,
    },
    primitives::ByteStream,
};
use aws_smithy_runtime_api::client::behavior_version::BehaviorVersion;
use bedrock_core::{
    ArtifactStoreConfig,
    PublishResult,
};
use futures::{
    StreamExt,
    TryStreamExt,
};
use s3::{
    bucket::Bucket,
    creds::Credentials,
    region::Region,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use si_data_nats::{
    NatsClient,
    jetstream,
};
use telemetry::tracing::{
    error,
    info,
};
use tokio::{
    fs::{
        File as TokioFile,
        File,
    },
    io::{
        AsyncReadExt,
        AsyncWriteExt,
        BufReader,
    },
    task,
    time::{
        interval,
        timeout,
    },
};

#[derive(Serialize, Deserialize)]
struct JsonMessage {
    subject: String,
    headers: HashMap<String, String>,
    payload_hex: String,
}

fn progress_bar_line(percentage: f64, current: usize, total: usize) -> String {
    let width = 30;
    let filled = (percentage * width as f64).round() as usize;
    let empty = width - filled;
    let bar = format!(
        "[{}{}] artifact @ {:>3}% | artifact {}/{}",
        "#".repeat(filled),
        " ".repeat(empty),
        (percentage * 100.0).round() as u64,
        current,
        total
    );
    bar
}

pub async fn capture_nats(
    nats_client: &NatsClient,
    nats_streams: &[String],
    recording_id: &str,
) -> Result<(), String> {
    let js = jetstream::new(nats_client.clone());
    let base_dir = PathBuf::from("./recordings/datasources")
        .join(recording_id)
        .join("nats_sequences");
    create_dir_all(&base_dir).map_err(|e| format!("Failed to create output directory: {}", e))?;

    for source_stream in nats_streams {
        let mirror_stream_name = format!("{}_AUDIT", source_stream);
        let output_path = base_dir.join(format!("{}.sequence", source_stream));

        println!("Dumping stream: {}", mirror_stream_name);

        let mut stream = js
            .get_stream(&mirror_stream_name)
            .await
            .map_err(|e| format!("Failed to get stream {}: {}", mirror_stream_name, e))?;

        let mut messages = vec![];
        let mut seq = 1u64;

        loop {
            // Refresh stream info to get the latest last_sequence
            let stream_info = stream
                .info()
                .await
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
                        if let Some(v) = vs.first() {
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
                    println!(
                        "Timeout waiting for message at seq {}. Ending capture.",
                        seq
                    );
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

        println!(
            "Dumped {} messages to {}",
            messages.len(),
            output_path.display()
        );

        js.delete_stream(&mirror_stream_name)
            .await
            .map_err(|e| format!("Failed to delete stream {}: {}", mirror_stream_name, e))?;

        println!("🗑 Deleted stream {}", mirror_stream_name);
    }

    Ok(())
}

pub fn resolve_local_sql_files(recording_id: &str) -> Result<Vec<String>, String> {
    let base_dir = PathBuf::from("./recordings/datasources")
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
        if path.is_file() && path.extension().is_some_and(|e| e == "sql") {
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
            "Found {} SQL restore file(s) in {}",
            sql_paths.len(),
            base_dir.display()
        );
        Ok(sql_paths)
    }
}

pub async fn resolve_remote_artifact_files(
    recording_id: &str,
    config: &ArtifactStoreConfig,
) -> Result<Vec<String>, String> {
    let bucket_name = config
        .metadata
        .get("bucketName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid 'bucketName' in artifact config".to_string())?;

    let prefix = format!("bedrock/datasources/{}/", recording_id);
    info!("🔍 Using prefix '{}' in bucket '{}'", prefix, bucket_name);

    let credentials = Credentials::anonymous()
        .map_err(|e| format!("Failed to get anonymous credentials: {}", e))?;

    let region = Region::Custom {
        region: "us-east-1".to_string(),
        endpoint: "https://s3.amazonaws.com".to_string(),
    };

    let bucket = Bucket::new(bucket_name, region, credentials)
        .map_err(|e| format!("Failed to create bucket: {}", e))?
        .with_path_style();

    let results = bucket
        .list(prefix.clone(), None)
        .await
        .map_err(|e| format!("Failed to list objects: {}", e))?;

    let mut all_objects = vec![];
    for result in results {
        all_objects.extend(
            result
                .contents
                .into_iter()
                .filter(|obj| !obj.key.ends_with('/')),
        );
    }

    let total = all_objects.len();
    if total == 0 {
        return Err(format!(
            "No downloadable files found under prefix {}",
            prefix
        ));
    }

    info!("Found {} files under S3 prefix '{}'", total, prefix);

    let mut downloaded_paths = Vec::new();
    let mut found_sql = false;
    let mut found_sequence = false;

    for (index, obj) in all_objects.into_iter().enumerate() {
        let key = obj.key;

        let relative_s3_path = key
            .strip_prefix("bedrock/datasources/")
            .ok_or_else(|| format!("Unexpected object key format: {}", key))?;

        let relative_path = Path::new("recordings")
            .join("datasources")
            .join(relative_s3_path);

        info!("[{}/{}] Fetching {}", index + 1, total, key);

        if let Some(parent) = relative_path.parent() {
            create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
        }

        let mut file = File::create(&relative_path)
            .await
            .map_err(|e| format!("Failed to create file {:?}: {}", relative_path, e))?;

        // Start downloading with progress tracking
        let mut response = bucket
            .get_object_stream(&key)
            .await
            .map_err(|e| format!("Failed to fetch object {}: {}", key, e))?;

        let total_bytes = Arc::new(AtomicUsize::new(0));
        let size_bytes = obj.size as usize;
        let total_bytes_clone = Arc::clone(&total_bytes);
        let artifact_index = index + 1;
        let downloading = Arc::new(AtomicBool::new(true));
        let downloading_clone = Arc::clone(&downloading);

        let logger_handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));
            while downloading_clone.load(Ordering::Relaxed) {
                ticker.tick().await;
                let downloaded = total_bytes_clone.load(Ordering::Relaxed);
                let percent = (downloaded as f64 / size_bytes as f64).min(1.0);
                let bar = progress_bar_line(percent, artifact_index, total);
                info!("{}", bar);
            }
        });

        while let Some(chunk) = response.bytes().next().await {
            let bytes = chunk.map_err(|e| format!("Stream error: {}", e))?;
            total_bytes.fetch_add(bytes.len(), Ordering::Relaxed);
            file.write_all(&bytes)
                .await
                .map_err(|e| format!("Failed to write chunk to file {:?}: {}", relative_path, e))?;
        }

        downloading.store(false, Ordering::Relaxed);
        logger_handle.abort();

        let bar = progress_bar_line(1.0, index + 1, total);
        info!("{}", bar);

        if key.ends_with(".sql") {
            found_sql = true;
        } else if key.ends_with(".sequence") {
            found_sequence = true;
        }

        downloaded_paths.push(relative_path.to_string_lossy().to_string());

        let progress_bar = format!(
            "[{}>{}]",
            "=".repeat((index + 1) * 20 / total),
            " ".repeat(20 - (index + 1) * 20 / total)
        );

        info!("{} Finished: {}", progress_bar, relative_path.display());
    }

    if !found_sql && !found_sequence {
        return Err(format!(
            "No .sql or .sequence files found under prefix {}",
            prefix
        ));
    } else if !found_sql {
        return Err(format!("No .sql files found under prefix {}", prefix));
    } // No sequence files is totally valid for a DB restore point only, i.e. not recording.

    info!(
        "Downloaded {} file(s) to ./recordings/datasources/{}",
        downloaded_paths.len(),
        recording_id
    );

    Ok(downloaded_paths)
}

pub async fn resolve_test(
    recording_id: &String,
    artifact_config: ArtifactStoreConfig,
) -> Result<Vec<String>, String> {
    match resolve_local_sql_files(recording_id) {
        Ok(paths) => Ok(paths),
        Err(local_err) => {
            println!("Local resolution failed: {}. Trying S3...", local_err);

            let all_paths = resolve_remote_artifact_files(recording_id, &artifact_config).await?;
            let sql_paths: Vec<String> = all_paths
                .into_iter()
                .filter(|p| p.ends_with(".sql"))
                .collect();

            if sql_paths.is_empty() {
                Err(format!(
                    "No .sql files found remotely for recording {}",
                    recording_id
                ))
            } else {
                println!(
                    "✅ Found {} SQL file(s) from remote download",
                    sql_paths.len()
                );
                Ok(sql_paths)
            }
        }
    }
}

pub async fn collect_files(recording_id: &str) -> Result<Vec<PathBuf>, String> {
    fn collect_files_rec(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| format!("Read dir error: {}", e))? {
            let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
            let path = entry.path();
            if path.is_dir() {
                collect_files_rec(&path, files)?;
            } else if path.is_file() {
                files.push(path);
            }
        }
        Ok(())
    }

    let base_path = PathBuf::from("recordings/datasources").join(recording_id);
    if !base_path.exists() {
        return Err(format!("Path does not exist: {:?}", base_path));
    }

    let mut file_paths = Vec::new();
    collect_files_rec(&base_path, &mut file_paths)?;
    Ok(file_paths)
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
            .map_err(|e| {
                format!(
                    "Failed to create consumer for {}: {}",
                    mirror_stream_name, e
                )
            })?;
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

pub async fn dump_databases(
    databases: &[String],
    recording_id: &str,
    variant: &str,
) -> Result<(), String> {
    let script_path = std::env::temp_dir().join("dump-database.sh");

    {
        let mut file = fs::File::create(&script_path)
            .map_err(|e| format!("Failed to create script file: {}", e))?;
        file.write_all(DATABASE_DUMP_SCRIPT.as_bytes())
            .map_err(|e| format!("Failed to write script: {}", e))?;

        let mut perms = file
            .metadata()
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
                        Err(format!(
                            "Script failed for {} with exit code: {}",
                            db, status
                        ))
                    }
                })
        })
        .await
        .map_err(|e| format!("Join error: {}", e))??;
    }

    Ok(())
}

const DATABASE_PREPARE_SCRIPT: &str = include_str!("../../scripts/prepare-database.sh");

pub async fn prepare_databases(sql_paths: Vec<String>) -> Result<(), String> {
    let script_path = std::env::temp_dir().join("prepare-database.sh");

    {
        let mut file = fs::File::create(&script_path)
            .map_err(|e| format!("Failed to create script file: {}", e))?;

        file.write_all(DATABASE_PREPARE_SCRIPT.as_bytes())
            .map_err(|e| format!("Failed to write script: {}", e))?;

        let mut perms = file
            .metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    // Spawn a task for each SQL file to be restored
    for sql_path in sql_paths {
        let path = Path::new(&sql_path);
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| format!("Invalid file name in path: {}", sql_path))?;

        let database_name = if file_name == "globals.sql" {
            "postgres".to_string()
        } else if file_name.ends_with("public_schema.sql") {
            file_name
                .strip_suffix("_public_schema.sql")
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
                        Err(format!(
                            "Script failed for {} with exit code: {}",
                            sql_path, status
                        ))
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
    aws_credentials: AwsCredentials,
    config: &ArtifactStoreConfig,
) -> PublishResult {
    let start_time = Instant::now();

    let result: Result<(), String> = async {
        let bucket_name = config
            .metadata
            .get("bucketName")
            .and_then(Value::as_str)
            .ok_or_else(|| "Missing `bucketName` in config metadata".to_string())?;

        // Check if credentials are empty
        let access_key_empty = aws_credentials.access_key_id().trim().is_empty();
        let secret_key_empty = aws_credentials.secret_access_key().trim().is_empty();

        if access_key_empty || secret_key_empty {
            return Err(format!(
                "Credentials are required to publish to the artifact store: {}",
                artifact_id
            ));
        }

        let region = AwsRegion::new("us-east-1");
        let config = ConfigBuilder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(region)
            .credentials_provider(aws_credentials)
            .build();

        let client = Client::from_conf(config);
        let s3_prefix = format!("bedrock/datasources/{}/", artifact_id);

        let existing = client
            .list_objects_v2()
            .bucket(bucket_name)
            .prefix(&s3_prefix)
            .send()
            .await
            .map_err(|e| format!("Failed to list objects: {}", e))?;

        if existing.key_count().unwrap_or(0) > 0 {
            return Err(format!(
                "Test '{}' already exists. Please re-identify and retry.",
                artifact_id
            ));
        }

        let base_path = PathBuf::from("recordings/datasources").join(artifact_id);
        if !base_path.exists() {
            return Err(format!(
                "Local artifact path does not exist: {:?}",
                base_path
            ));
        }

        let file_paths: Vec<PathBuf> = collect_files(artifact_id).await?;

        for (index, path) in file_paths.iter().enumerate() {
            let key = path
                .strip_prefix("recordings/datasources")
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");

            let s3_key = format!("bedrock/datasources/{}", key);
            let total = file_paths.len();
            let artifact_index = index + 1;

            info!(
                "[{}/{}] Uploading -> s3://{}/{}",
                artifact_index, total, bucket_name, s3_key
            );

            let file = TokioFile::open(&path)
                .await
                .map_err(|e| format!("Failed to open file {:?}: {}", path, e))?;

            let metadata = file.metadata().await.map_err(|e| e.to_string())?;
            let size_bytes = metadata.len() as usize;

            let total_bytes = Arc::new(AtomicUsize::new(0));
            let downloading = Arc::new(AtomicBool::new(true));
            let total_bytes_clone = Arc::clone(&total_bytes);
            let downloading_clone = Arc::clone(&downloading);

            let logger_handle = tokio::spawn(async move {
                let mut ticker = interval(Duration::from_secs(5));
                while downloading_clone.load(Ordering::Relaxed) {
                    ticker.tick().await;
                    let downloaded = total_bytes_clone.load(Ordering::Relaxed);
                    let percent = if size_bytes > 0 {
                        (downloaded as f64 / size_bytes as f64).min(1.0)
                    } else {
                        1.0
                    };
                    let bar = progress_bar_line(percent, artifact_index, total);
                    info!("{}", bar);
                }
            });

            let mut buffer = Vec::with_capacity(size_bytes);
            let mut reader = BufReader::new(file);
            let mut chunk = [0u8; 8192];
            loop {
                let n = reader.read(&mut chunk).await.map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                total_bytes.fetch_add(n, Ordering::Relaxed);
                buffer.extend_from_slice(&chunk[..n]);
            }

            downloading.store(false, Ordering::Relaxed);
            logger_handle.abort();

            client
                .put_object()
                .bucket(bucket_name)
                .key(&s3_key)
                .body(ByteStream::from(buffer))
                .send()
                .await
                .map_err(|e| format!("Upload failed: {}", e))?;

            let bar = progress_bar_line(1.0, artifact_index, total);
            info!("{}", bar);

            let progress_bar = format!(
                "[{}>{}]",
                "=".repeat((artifact_index * 20) / total),
                " ".repeat(20 - (artifact_index * 20) / total)
            );

            info!("{} Finished: {}", progress_bar, path.display());
        }

        Ok(())
    }
    .await;

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
