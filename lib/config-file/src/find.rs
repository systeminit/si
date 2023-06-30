use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
};

use directories::ProjectDirs;
use tracing::{debug, trace};

use crate::{ConfigFileError, FileFormat, ToFileFormats};

pub fn find(
    app_name: impl AsRef<str>,
    file_formats: impl ToFileFormats,
    env_var: &Option<impl AsRef<OsStr>>,
) -> Result<Option<(PathBuf, FileFormat)>, ConfigFileError> {
    let app_name = app_name.as_ref();
    let (candidate, file_format) = match find_first_file_candidate(app_name, file_formats, env_var)?
    {
        Some(candidate) => candidate,
        None => return Ok(None),
    };
    let target = candidate
        .canonicalize()
        .map_err(|err| ConfigFileError::Canonicalize(err, candidate))?;

    Ok(Some((target, file_format)))
}

fn find_first_file_candidate(
    app_name: impl AsRef<str>,
    file_formats: impl ToFileFormats,
    env_var: &Option<impl AsRef<OsStr>>,
) -> Result<Option<(PathBuf, FileFormat)>, ConfigFileError> {
    let app_name = app_name.as_ref();
    let goals: Vec<_> = file_formats
        .to_file_formats()?
        .map(|f| (goal_file_name(app_name, f), f))
        .collect();

    // Return candidate if environment variable is asked for with a value
    if let Some(ref env_var) = env_var {
        trace!(
            "checking environment variable; var={}",
            env_var.as_ref().to_string_lossy().as_ref()
        );
        #[allow(clippy::disallowed_methods)] // This method explicitly checks env vars as its
        // strategy
        if let Ok(value) = env::var(env_var) {
            let env_candidate = Path::new(&value);
            if env_candidate.is_file() {
                trace!(
                    "candidate found in environment and is a file; var={}, candidate={}",
                    env_var.as_ref().to_string_lossy().as_ref(),
                    env_candidate.display()
                );
                let extension = env_candidate
                    .extension()
                    .ok_or_else(|| {
                        ConfigFileError::UnknownFileFormat(
                            env_candidate.to_string_lossy().to_string(),
                        )
                    })?
                    .to_string_lossy();
                let file_format = FileFormat::from_str(extension.as_ref())?;

                let found = PathBuf::from(value);
                trace!(path = %found.display(), "found candidate from environment");
                return Ok(Some((found, file_format)));
            }

            // File doesn't exist, but we asked for it explicitly, so we fail
            return Err(ConfigFileError::EnvNotFound(
                app_name.to_string(),
                env_var.as_ref().to_string_lossy().to_string(),
                value,
            ));
        }
    }

    // Return candidate if found in the current directory
    for (goal, file_format) in &goals {
        let current_dir_candidate = env::current_dir()
            .map_err(ConfigFileError::CurrentDirectory)?
            .join(goal);
        trace!(path = %current_dir_candidate.display(), "checking candidate in current directory");
        if current_dir_candidate.is_file() {
            debug!(path = %current_dir_candidate.display(), "found candidate in current directory");
            return Ok(Some((current_dir_candidate, *file_format)));
        }
    }

    // Return candidate if found in the user's local config directory
    for (goal, file_format) in &goals {
        let user_candidate = ProjectDirs::from("", "", app_name)
            .ok_or(ConfigFileError::HomeDirectory)?
            .config_dir()
            .join(goal);
        trace!(path = %user_candidate.display(), "checking candidate in user directory");
        if user_candidate.is_file() {
            debug!(path = %user_candidate.display(), "found candidate in user directory");
            return Ok(Some((user_candidate, *file_format)));
        }
    }

    // Return candidate if found in root location
    for (goal, file_format) in &goals {
        for prefix in ["/usr/local/etc", "/etc"] {
            let mut root_candidate = Path::new(prefix).join(app_name).join(goal);
            #[allow(clippy::disallowed_methods)] // This is a supported env var which we must check
            if let Ok(fs_root) = env::var("FS_ROOT") {
                root_candidate = Path::new(&fs_root).join(root_candidate);
            }
            trace!(path = %root_candidate.display(), "checking candidate in root");
            if root_candidate.is_file() {
                debug!(path = %root_candidate.display(), "found candidate in root");
                return Ok(Some((root_candidate, *file_format)));
            }
        }
    }

    // We tried, but couldn't find a successful candidate
    debug!("no config file candidates found");
    Ok(None)
}

fn goal_file_name(app_name: impl AsRef<str>, file_format: FileFormat) -> String {
    format!("{}.{}", app_name.as_ref(), file_format.as_str())
}
