use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use axum::extract::FromRef;
use tokio::sync::mpsc;

#[derive(Clone, FromRef)]
pub struct AppState {
    lang_server_path: LangServerPath,
    telemetry_level: TelemetryLevel,
    lang_server_function_timeout: LangServerFunctionTimeout,
    lang_server_process_timeout: LangServerProcessTimeout,
}

impl AppState {
    pub fn new(
        lang_server_path: impl Into<PathBuf>,
        telemetry_level: Box<dyn telemetry::TelemetryLevel>,
        lang_server_function_timeout: Option<usize>,
        lang_server_process_timeout: Option<u64>,
    ) -> Self {
        Self {
            lang_server_path: LangServerPath(Arc::new(lang_server_path.into())),
            telemetry_level: TelemetryLevel(Arc::new(telemetry_level)),
            lang_server_function_timeout: LangServerFunctionTimeout(Arc::new(
                lang_server_function_timeout,
            )),
            lang_server_process_timeout: LangServerProcessTimeout(Arc::new(
                lang_server_process_timeout,
            )),
        }
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
