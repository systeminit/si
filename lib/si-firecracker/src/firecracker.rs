#[cfg(target_os = "linux")]
use crate::disk::FirecrackerDisk;
use crate::errors::FirecrackerJailError;
#[cfg(target_os = "linux")]
use crate::stream::UnixStreamForwarder;
use cyclone_core::process;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::result;
use tokio::fs;
use tokio::process::Child;
use tokio::process::Command;

type Result<T> = result::Result<T, FirecrackerJailError>;

const FIRECRACKER_PREPARE_PATH: &str = "/firecracker-data/prepare_jailer.sh";
const FIRECRACKER_SETUP_PATH: &str = "/firecracker-data/firecracker-setup.sh";
const FIRECRACKER_PREPARE_BYTES: &[u8] = include_bytes!("scripts/prepare_jailer.sh");
const FIRECRACKER_SETUP_BYTES: &[u8] = include_bytes!("scripts/firecracker-setup.sh");

const FIRECRACKER_SCRIPTS: &[(&str, &[u8])] = &[
    (FIRECRACKER_PREPARE_PATH, FIRECRACKER_PREPARE_BYTES),
    (FIRECRACKER_SETUP_PATH, FIRECRACKER_SETUP_BYTES),
];

#[derive(Debug)]
pub struct FirecrackerJail {
    jailer: Command,
    child: Option<Child>,
    socket: PathBuf,
}

impl FirecrackerJail {
    pub fn socket(&self) -> PathBuf {
        self.socket.to_owned()
    }

    pub async fn build(id: u32) -> Result<Self> {
        let mut cmd = Command::new("/usr/bin/jailer");
        cmd.arg("--cgroup-version")
            .arg("2")
            .arg("--id")
            .arg(id.to_string())
            .arg("--exec-file")
            .arg("/usr/bin/firecracker")
            .arg("--uid")
            .arg(format!("500{}", id))
            .arg("--gid")
            .arg("10000")
            .arg("--netns")
            .arg(format!("/var/run/netns/jailer-{}", id))
            .arg("--")
            .arg("--config-file")
            .arg("./firecracker.conf");
        let socket = PathBuf::from(&format!("/srv/jailer/firecracker/{}/root/v.sock", id));

        Ok(Self {
            jailer: cmd,
            child: None,
            socket,
        })
    }

    pub async fn clean(id: u32) -> Result<()> {
        let _ = id;
        #[cfg(target_os = "linux")]
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
            return Err(FirecrackerJailError::Output(
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "Failed to decode stderr".to_string()),
            ));
        }

        #[cfg(target_os = "linux")]
        UnixStreamForwarder::new(FirecrackerDisk::jail_dir_from_id(id), id)
            .await?
            .start()
            .await?;

        Ok(())
    }

    pub async fn setup(pool_size: u16) -> Result<()> {
        Self::create_scripts().await?;

        let output = Command::new("sudo")
            .arg(FIRECRACKER_SETUP_PATH)
            .arg("-j")
            .arg(pool_size.to_string())
            .arg("-rk")
            .spawn()?
            .wait_with_output()
            .await?;

        if !output.status.success() {
            return Err(FirecrackerJailError::Output(
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "Failed to decode stderr".to_string()),
            ));
        }

        Ok(())
    }

    pub async fn spawn(&mut self) -> Result<()> {
        self.child = Some(self.jailer.spawn().map_err(FirecrackerJailError::Spawn)?);
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

    async fn create_scripts() -> Result<()> {
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
