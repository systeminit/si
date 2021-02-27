use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use strum_macros::{Display, EnumString};
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncRead, AsyncWrite, BufReader, BufWriter},
};

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Display, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum OutputLineStream {
    Stdout,
    Stderr,
    All,
}

#[derive(Error, Debug)]
pub enum EventLogFSError {
    #[error("cached file does not exist: {0}")]
    CachedNotFound(PathBuf),
    #[error("destination file exists, cannot copy over: {0}")]
    FileAlreadyExists(String),
    #[error("io error: {0}")]
    IO(#[from] tokio::io::Error),
}

pub type EventLogFSResult<T> = Result<T, EventLogFSError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventLogFS {
    cache_path: PathBuf,
    active_path: PathBuf,
    pending_persist_path: PathBuf,
}

async fn is_file(path: impl AsRef<Path>) -> bool {
    fs::metadata(path)
        .await
        .map(|m| m.is_file())
        .unwrap_or(false)
}

impl EventLogFS {
    pub async fn init(settings: &si_settings::EventLogFs) -> EventLogFSResult<Self> {
        let root = &settings.root;

        let fs = Self {
            cache_path: root.join("cache"),
            active_path: root.join("active"),
            pending_persist_path: root.join("pending_persist"),
        };

        fs::create_dir_all(&fs.cache_path).await?;
        fs::create_dir_all(&fs.active_path).await?;
        fs::create_dir_all(&fs.pending_persist_path).await?;

        Ok(fs)
    }

    pub async fn get_write_handle(
        &self,
        event_log_id: impl AsRef<str>,
        stream: &OutputLineStream,
    ) -> EventLogFSResult<Pin<Box<dyn AsyncWrite + Sync + Send>>> {
        let log_path = self.active_path.join(basename(event_log_id, stream));

        if !is_file(&log_path).await {
            let _ = fs::File::create(&log_path).await?;
        }
        let file = BufWriter::new(fs::OpenOptions::new().append(true).open(log_path).await?);

        Ok(Box::pin(file))
    }

    pub async fn finalize(
        &self,
        event_log_id: impl AsRef<str>,
        stream: &OutputLineStream,
    ) -> EventLogFSResult<()> {
        let event_log_id = event_log_id.as_ref();
        let src = self.active_path.join(basename(event_log_id, stream));
        let dst = self
            .pending_persist_path
            .join(basename(event_log_id, stream));

        if is_file(&dst).await {
            return Err(EventLogFSError::FileAlreadyExists(format!(
                "{}",
                dst.display()
            )));
        }

        fs::rename(&src, &dst).await?;

        {
            // TODO(fnichol): this temporary implementation takes the file from the "pending
            // persist" location where it would be processed (i.e. uploaded), and moves it
            // directly into the cache path. A future spawned service will handle this
            // uploading and moving, but until then, this is the faked version, so remove when
            // appropriate, s'il vous plait
            let temp_all_the_way_done = self.cache_path.join(basename(event_log_id, stream));
            fs::rename(&dst, &temp_all_the_way_done).await?;
        }

        Ok(())
    }

    pub async fn get_read_handle(
        &self,
        event_log_id: impl AsRef<str>,
        stream: &OutputLineStream,
    ) -> EventLogFSResult<Pin<Box<dyn AsyncRead>>> {
        let log_path = self.cache_path.join(basename(event_log_id, stream));

        if !is_file(&log_path).await {
            return Err(EventLogFSError::CachedNotFound(log_path));
        }

        let file = BufReader::new(fs::File::open(&log_path).await?);

        Ok(Box::pin(file))
    }
}

fn basename(event_log_id: impl AsRef<str>, stream: &OutputLineStream) -> String {
    format!("{}.{}.log", event_log_id.as_ref(), stream)
}
