use std::{
    fs::Permissions,
    io::Error,
    os::unix::fs::PermissionsExt,
    path::{
        Path,
        PathBuf,
    },
    process::Stdio,
    result,
};

use cyclone_core::process;
use tokio::{
    fs,
    io::{
        AsyncBufReadExt,
        AsyncWriteExt,
        BufReader,
    },
    net::UnixStream,
    process::{
        Child,
        Command,
    },
    time::{
        Duration,
        timeout,
    },
};
use tracing::{
    debug,
    info,
};

use crate::{
    disk::FirecrackerDisk,
    errors::FirecrackerJailError,
};

type Result<T> = result::Result<T, FirecrackerJailError>;

/// Spawn a background task to forward process output to debug logs
fn spawn_log_forwarder(
    stream: Option<impl tokio::io::AsyncRead + Unpin + Send + 'static>,
    id: u32,
    stream_name: &'static str,
) {
    if let Some(stream) = stream {
        tokio::spawn(async move {
            let reader = BufReader::new(stream);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                debug!(jail_id = id, stream = stream_name, "{}", line);
            }
        });
    }
}

const FIRECRACKER_PREPARE_PATH: &str = "/firecracker-data/prepare_jailer.sh";
const FIRECRACKER_SETUP_PATH: &str = "/firecracker-data/firecracker-setup.sh";
const FIRECRACKER_SNAPSHOT_PREPARE_PATH: &str = "/firecracker-data/prepare_snapshot_restore.sh";
const FIRECRACKER_CREATE_SNAPSHOT_PATH: &str = "/firecracker-data/create_golden_snapshot.sh";

const FIRECRACKER_PREPARE_BYTES: &[u8] = include_bytes!("scripts/prepare_jailer.sh");
const FIRECRACKER_SETUP_BYTES: &[u8] = include_bytes!("scripts/firecracker-setup.sh");
const FIRECRACKER_SNAPSHOT_PREPARE_BYTES: &[u8] =
    include_bytes!("scripts/prepare_snapshot_restore.sh");
const FIRECRACKER_CREATE_SNAPSHOT_BYTES: &[u8] =
    include_bytes!("scripts/create_golden_snapshot.sh");

const FIRECRACKER_SCRIPTS: &[(&str, &[u8])] = &[
    (FIRECRACKER_PREPARE_PATH, FIRECRACKER_PREPARE_BYTES),
    (FIRECRACKER_SETUP_PATH, FIRECRACKER_SETUP_BYTES),
    (
        FIRECRACKER_SNAPSHOT_PREPARE_PATH,
        FIRECRACKER_SNAPSHOT_PREPARE_BYTES,
    ),
    (
        FIRECRACKER_CREATE_SNAPSHOT_PATH,
        FIRECRACKER_CREATE_SNAPSHOT_BYTES,
    ),
];

/// Path to the golden snapshot directory
const GOLDEN_SNAPSHOT_DIR: &str = "/firecracker-data/golden-snapshot";

/// Spawn strategy for FirecrackerJail
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SpawnStrategy {
    /// Cold boot: Start VM from scratch using firecracker.conf
    #[default]
    ColdBoot,
    /// Snapshot restore: Load from golden snapshot (much faster)
    SnapshotRestore,
}

#[derive(Debug)]
pub struct FirecrackerJail {
    id: u32,
    jailer: Command,
    child: Option<Child>,
    socket: PathBuf,
    api_socket: PathBuf,
}

impl FirecrackerJail {
    pub fn socket(&self) -> PathBuf {
        self.socket.to_owned()
    }

    /// Build a jail configured for cold boot (traditional startup)
    pub async fn build(id: u32) -> Result<Self> {
        Self::build_with_strategy(id, SpawnStrategy::ColdBoot).await
    }

    /// Build a jail with specified spawn strategy
    pub async fn build_with_strategy(id: u32, strategy: SpawnStrategy) -> Result<Self> {
        let mut cmd = Command::new("/usr/bin/jailer");
        cmd.arg("--cgroup-version")
            .arg("2")
            .arg("--parent-cgroup")
            .arg("veritech/firecracker")
            .arg("--cgroup")
            .arg("cpuset.cpus=16-63")
            .arg("--cgroup")
            .arg("cpu.max=1000000,1000000")
            .arg("--id")
            .arg(id.to_string())
            .arg("--exec-file")
            .arg("/usr/bin/firecracker")
            .arg("--uid")
            .arg(format!("500{id}"))
            .arg("--gid")
            .arg("10000")
            .arg("--netns")
            .arg(format!("/var/run/netns/jailer-{id}"))
            .arg("--");

        // For cold boot, use config file. For snapshot restore, just start with API socket
        match strategy {
            SpawnStrategy::ColdBoot => {
                cmd.arg("--config-file").arg("./firecracker.conf");
            }
            SpawnStrategy::SnapshotRestore => {
                cmd.arg("--api-sock").arg("/run/firecracker.socket");
            }
        }

        // Pipe stdout/stderr so we can redirect to debug logs
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        // For snapshot restore, the vsock path is stored in the snapshot.
        // The golden snapshot (created by create_golden_snapshot.sh)
        // uses /tmp/fc-vsock-test.sock which maps to
        // /srv/jailer/firecracker/{id}/root/tmp/fc-vsock-test.sock after chroot.
        // For cold boot, the config file specifies ./v.sock
        let socket = match strategy {
            SpawnStrategy::ColdBoot => {
                PathBuf::from(&format!("/srv/jailer/firecracker/{id}/root/v.sock"))
            }
            SpawnStrategy::SnapshotRestore => PathBuf::from(&format!(
                "/srv/jailer/firecracker/{id}/root/tmp/fc-vsock-test.sock"
            )),
        };
        let api_socket = PathBuf::from(&format!(
            "/srv/jailer/firecracker/{id}/root/run/firecracker.socket"
        ));

        Ok(Self {
            id,
            jailer: cmd,
            child: None,
            socket,
            api_socket,
        })
    }

    pub async fn clean(id: u32) -> Result<()> {
        FirecrackerDisk::clean(id)?;
        Ok(())
    }

    pub async fn prepare(id: u32) -> Result<()> {
        let output = Command::new(FIRECRACKER_PREPARE_PATH)
            .arg(id.to_string())
            .output()
            .await
            .map_err(FirecrackerJailError::Prepare)?;

        if !output.status.success() {
            return Err(FirecrackerJailError::Prepare(Error::other(
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "Failed to decode stderr".to_string()),
            )));
        }

        Ok(())
    }

    /// Prepare a jail for snapshot restore (faster than cold boot prepare)
    pub async fn prepare_for_snapshot(id: u32) -> Result<()> {
        let output = Command::new(FIRECRACKER_SNAPSHOT_PREPARE_PATH)
            .arg(id.to_string())
            .output()
            .await
            .map_err(FirecrackerJailError::Prepare)?;

        if !output.status.success() {
            return Err(FirecrackerJailError::Prepare(Error::other(
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "Failed to decode stderr".to_string()),
            )));
        }

        Ok(())
    }

    /// Check if a golden snapshot exists and is ready for use
    pub fn golden_snapshot_exists() -> bool {
        let vmstate = PathBuf::from(GOLDEN_SNAPSHOT_DIR).join("vmstate");
        let memory = PathBuf::from(GOLDEN_SNAPSHOT_DIR).join("memory");
        vmstate.exists() && memory.exists()
    }

    /// Spawn the VM and load from snapshot via API
    /// This should be called after prepare_for_snapshot() and build_with_strategy(SnapshotRestore)
    pub async fn spawn_from_snapshot(&mut self) -> Result<()> {
        // Start the jailer/firecracker process
        let mut child = self.jailer.spawn().map_err(FirecrackerJailError::Spawn)?;

        // Forward stdout/stderr to debug logs
        spawn_log_forwarder(child.stdout.take(), self.id, "stdout");
        spawn_log_forwarder(child.stderr.take(), self.id, "stderr");

        self.child = Some(child);

        // Wait for API socket to be ready
        let socket_ready = timeout(Duration::from_secs(5), async {
            loop {
                if self.api_socket.exists() {
                    // Try to connect to verify it's actually listening
                    if UnixStream::connect(&self.api_socket).await.is_ok() {
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await;

        if socket_ready.is_err() {
            return Err(FirecrackerJailError::Spawn(Error::other(
                "Timeout waiting for firecracker API socket",
            )));
        }

        // Send snapshot load request
        self.send_snapshot_load_request().await?;

        Ok(())
    }

    /// Send the snapshot load request to firecracker API
    async fn send_snapshot_load_request(&self) -> Result<()> {
        let mut stream = UnixStream::connect(&self.api_socket).await.map_err(|e| {
            FirecrackerJailError::Spawn(Error::other(format!(
                "Failed to connect to firecracker API socket: {e}"
            )))
        })?;

        // The snapshot stores absolute paths, which are recreated inside the jail
        // Paths are relative to jail root after chroot
        let request_body = r#"{
            "snapshot_path": "/vmstate",
            "mem_backend": {
                "backend_type": "File",
                "backend_path": "/memory"
            },
            "enable_diff_snapshots": false,
            "resume_vm": true
        }"#;

        let request = format!(
            "PUT /snapshot/load HTTP/1.1\r\n\
             Host: localhost\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             \r\n\
             {}",
            request_body.len(),
            request_body
        );

        stream.write_all(request.as_bytes()).await.map_err(|e| {
            FirecrackerJailError::Spawn(Error::other(format!(
                "Failed to send snapshot load request: {e}"
            )))
        })?;

        // Read response (we don't strictly need to parse it, just check for errors)
        let mut response = vec![0u8; 4096];
        let n = timeout(Duration::from_secs(10), stream.readable())
            .await
            .map_err(|_| {
                FirecrackerJailError::Spawn(Error::other(
                    "Timeout waiting for snapshot load response",
                ))
            })?;
        n.map_err(|e| {
            FirecrackerJailError::Spawn(Error::other(format!(
                "Failed to read snapshot load response: {e}"
            )))
        })?;

        // Try to read the response
        match stream.try_read(&mut response) {
            Ok(n) if n > 0 => {
                let response_str = String::from_utf8_lossy(&response[..n]);
                // Check for success (HTTP 204 No Content)
                if response_str.contains("204") || response_str.contains("200") {
                    debug!("Snapshot load successful");
                    Ok(())
                } else if response_str.contains("400") || response_str.contains("fault_message") {
                    Err(FirecrackerJailError::Spawn(Error::other(format!(
                        "Snapshot load failed: {response_str}"
                    ))))
                } else {
                    // Assume success if we got a response without clear error
                    debug!(response = %response_str, "Snapshot load response");
                    Ok(())
                }
            }
            Ok(_) => {
                // Empty response, assume success (204 No Content)
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data ready yet, but socket is connected - assume success
                Ok(())
            }
            Err(e) => Err(FirecrackerJailError::Spawn(Error::other(format!(
                "Failed to read snapshot load response: {e}"
            )))),
        }
    }

    pub async fn setup(pool_size: u32, create_scripts: bool) -> Result<()> {
        if create_scripts {
            info!("creating scripts...");
            Self::deploy_scripts().await?;
        } else {
            info!("skipping creation of scripts and checking that they exist...");

            // This is normally not a good idea. Just try to use the file and don't perform
            // point-in-time file existence checks. HOWEVER, this is a weird case where we are
            // explicitly not creating our own scripts, so performing a safety gut check (the foil
            // to "create these scripts") will help disambiguate the error vs. running the command
            // to execute the script and then not having a clear error for what's going on.
            let mut missing_scripts = Vec::new();
            for (path, _) in FIRECRACKER_SCRIPTS {
                if !std::fs::exists(path)? {
                    missing_scripts.push(path.to_string());
                }
            }
            if !missing_scripts.is_empty() {
                return Err(FirecrackerJailError::SetupScriptsDoNotExist(
                    missing_scripts,
                ));
            }
        }

        // we want to work with a clean slate, but we don't necessarily care about failures here
        for id in 0..pool_size + 1 {
            Self::clean(id).await?;
        }

        let output = Command::new("sudo")
            .arg(FIRECRACKER_SETUP_PATH)
            .arg("-j")
            .arg(pool_size.to_string())
            .spawn()?
            .wait_with_output()
            .await?;

        if !output.status.success() {
            return Err(FirecrackerJailError::Setup(Error::other(
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "Failed to decode stderr".to_string()),
            )));
        }

        Ok(())
    }

    pub async fn spawn(&mut self) -> Result<()> {
        let mut child = self.jailer.spawn().map_err(FirecrackerJailError::Spawn)?;

        // Forward stdout/stderr to debug logs
        spawn_log_forwarder(child.stdout.take(), self.id, "stdout");
        spawn_log_forwarder(child.stderr.take(), self.id, "stderr");

        self.child = Some(child);
        Ok(())
    }

    pub async fn terminate(&mut self) -> Result<()> {
        match self.child.as_mut() {
            Some(c) => {
                process::child_shutdown(c, Some(process::Signal::SIGTERM), None).await?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    /// Deploy embedded scripts to /firecracker-data/ without running full setup
    pub async fn deploy_scripts() -> Result<()> {
        for (path, bytes) in FIRECRACKER_SCRIPTS {
            Self::create_script(Path::new(*path), bytes).await?;
        }
        Ok(())
    }

    async fn create_script(path: &Path, bytes: &[u8]) -> Result<()> {
        if let Some(parent_dir) = path.parent() {
            fs::create_dir_all(parent_dir).await?
        }
        fs::write(&path, bytes).await?;
        fs::set_permissions(&path, Permissions::from_mode(0o755)).await?;
        Ok(())
    }
}
