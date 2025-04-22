use std::{
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use axum::extract::FromRef;
use telemetry::tracing::debug;
use tokio::{
    process::{
        Child,
        Command,
    },
    sync::{
        Mutex,
        mpsc,
    },
};

use crate::execution::ExecutionError;
type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Clone, FromRef)]
pub struct AppState {
    child: LangServerChild,
    lang_server_process_timeout: LangServerProcessTimeout,
    telemetry_level: TelemetryLevel,
}

impl AppState {
    pub async fn new(
        lang_server_path: impl Into<PathBuf> + std::convert::AsRef<std::ffi::OsStr>,
        telemetry_level: Box<dyn telemetry::TelemetryLevel>,
        lang_server_function_timeout: Option<usize>,
        lang_server_process_timeout: Option<u64>,
    ) -> Result<Self> {
        let mut cmd = Command::new(&lang_server_path);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(timeout) = lang_server_function_timeout {
            cmd.arg("--timeout").arg(timeout.to_string());
        }
        if telemetry_level.is_debug_or_lower().await {
            cmd.env("SI_LANG_JS_LOG", "*");
        }

        debug!(cmd = ?cmd, "spawning child process");
        let child = cmd
            .spawn()
            .map_err(|err| ExecutionError::ChildSpawn(err, lang_server_path.into()))?;

        Ok(Self {
            child: LangServerChild(Arc::new(Mutex::new(child))),
            lang_server_process_timeout: LangServerProcessTimeout(Arc::new(
                lang_server_process_timeout,
            )),
            telemetry_level: TelemetryLevel(Arc::new(telemetry_level)),
        })
    }
}

#[derive(Clone, Debug, FromRef)]
pub struct LangServerPath(Arc<PathBuf>);

impl LangServerPath {
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

#[derive(Clone, FromRef)]
pub struct TelemetryLevel(Arc<Box<dyn telemetry::TelemetryLevel>>);

impl Deref for TelemetryLevel {
    type Target = Box<dyn telemetry::TelemetryLevel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, FromRef)]
pub struct LangServerFunctionTimeout(Arc<Option<usize>>);

impl LangServerFunctionTimeout {
    pub fn inner(&self) -> Option<usize> {
        Arc::clone(&self.0).as_ref().to_owned()
    }
}

#[derive(Clone, Debug, FromRef)]
pub struct LangServerChild(Arc<Mutex<Child>>);

impl LangServerChild {
    pub fn inner(&self) -> Arc<Mutex<Child>> {
        Arc::clone(&self.0)
    }
}

#[derive(Clone, Debug, FromRef)]
pub struct LangServerProcessTimeout(Arc<Option<u64>>);

impl LangServerProcessTimeout {
    pub fn inner(&self) -> Option<u64> {
        Arc::clone(&self.0).as_ref().to_owned()
    }
}

pub struct WatchKeepalive {
    tx: mpsc::Sender<()>,
    timeout: Duration,
}

impl WatchKeepalive {
    pub fn new(tx: mpsc::Sender<()>, timeout: Duration) -> Self {
        Self { tx, timeout }
    }

    pub fn clone_tx(&self) -> mpsc::Sender<()> {
        self.tx.clone()
    }

    /// Gets a reference to the watch keepalive tx's timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}
