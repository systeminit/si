use std::{
    fs::Permissions,
    io::Error,
    os::unix::fs::PermissionsExt,
    path::{
        Path,
        PathBuf,
    },
    result,
};

use cyclone_core::process;
use tokio::{
    fs,
    process::{
        Child,
        Command,
    },
};
use tracing::info;

use crate::{
    disk::FirecrackerDisk,
    errors::FirecrackerJailError,
};

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
            .arg("--")
            .arg("--config-file")
            .arg("./firecracker.conf");
        let socket = PathBuf::from(&format!("/srv/jailer/firecracker/{id}/root/v.sock"));

        Ok(Self {
            jailer: cmd,
            child: None,
            socket,
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

        // TODO(nick,john,fletcher): delete or restore this once verideath investigation is done.
        // UnixStreamForwarder::new(FirecrackerDisk::jail_dir_from_id(id), id)
        //     .await?
        //     .start()
        //     .await?;

        Ok(())
    }

    pub async fn setup(pool_size: u32, create_scripts: bool) -> Result<()> {
        if create_scripts {
            info!("creating scripts...");
            Self::create_scripts().await?;
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
            .arg("-rk")
            .spawn()?
            .wait_with_output()
            .await?;

        if !output.status.success() {
            // FIXME(nick): came by and read this... it looks like we wanted this error enum to
            // encapsulate all kinds of setup errors into one. We should instead create a new
            // error enum with its own variants and make this a formal variant. Why? We may need
            // to provide more context and/or capture stdout here. Many script errors end with
            // empty stderr.
            return Err(FirecrackerJailError::Setup(Error::other(
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "Failed to decode stderr".to_string()),
            )));
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
