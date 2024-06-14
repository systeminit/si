use crate::errors::FirecrackerJailError;
use cyclone_core::process;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::result;
use tokio::process::Child;
use tokio::process::Command;

type Result<T> = result::Result<T, FirecrackerJailError>;

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
        let command = String::from("/firecracker-data/stop.sh");
        let output = Command::new(command)
            .arg(id.to_string())
            .output()
            .await
            .map_err(|e| FirecrackerJailError::Clean(e.to_string()))?;

        if !output.status.success() {
            return Err(FirecrackerJailError::Clean(
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "Failed to decode stderr".to_string()),
            ));
        }
        Ok(())
    }

    pub async fn prepare(id: u32) -> Result<()> {
        let command = String::from("/firecracker-data/prepare_jailer.sh");
        let output = Command::new(command)
            .arg(id.to_string())
            .output()
            .await
            .map_err(|e| FirecrackerJailError::Prepare(e.to_string()))?;

        if !output.status.success() {
            return Err(FirecrackerJailError::Prepare(
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "Failed to decode stderr".to_string()),
            ));
        }
        Ok(())
    }

    pub async fn setup(pool_size: u16) -> Result<()> {
        let script_bytes = include_bytes!("firecracker-setup.sh");
        let command = Path::new("/firecracker-data/firecracker-setup.sh");

        // we need to ensure the file is in the correct location with the correct permissions
        std::fs::create_dir_all(
            command
                .parent()
                .expect("This should never happen. Did you remove the path from the string above?"),
        )
        .map_err(|e| FirecrackerJailError::Setup(e.to_string()))?;

        std::fs::write(command, script_bytes)
            .map_err(|e| FirecrackerJailError::Setup(e.to_string()))?;

        std::fs::set_permissions(command, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| FirecrackerJailError::Setup(e.to_string()))?;

        // Spawn the shell process
        let output = Command::new("sudo")
            .arg(command)
            .arg("-j")
            .arg(pool_size.to_string())
            .arg("-rk")
            .spawn()
            .map_err(|e| FirecrackerJailError::Setup(e.to_string()))?
            .wait_with_output()
            .await
            .map_err(|e| FirecrackerJailError::Setup(e.to_string()))?;

        if !output.status.success() {
            return Err(FirecrackerJailError::Setup(
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "Failed to decode stderr".to_string()),
            ));
        }

        Ok(())
    }

    pub async fn spawn(&mut self) -> Result<()> {
        self.child = Some(
            self.jailer
                .spawn()
                .map_err(|e| FirecrackerJailError::Spawn(e.to_string()))?,
        );
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
}
