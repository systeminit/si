use crate::containers::{
    cleanup_image, delete_container, get_container_details, get_existing_container,
};
use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, SiCliError};
use colored::Colorize;
use docker_api::Docker;
use flate2::read::GzDecoder;
use inquire::Confirm;
use serde::Deserialize;
use std::fs;
use std::io::Cursor;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: i64,
    pub content_type: String,
    pub size: i64,
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: i64,
    pub version: String,
    pub name: String,
    pub description: String,
    pub assets: Vec<Asset>,
    pub published_at: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LatestContainer {
    pub namespace: String,
    pub repository: String,
    pub git_sha: String,
    pub digest: String,
}

#[derive(Debug)]
pub struct Update {
    pub containers: Vec<LatestContainer>,
    pub si: Option<Release>,
}

static HOST: &str = "https://auth-api.systeminit.com";

async fn update_current_binary(url: &str) -> CliResult<()> {
    let current_exe = std::env::current_exe()?;

    let exe_path = if current_exe.is_symlink() {
        fs::read_link(current_exe)?
    } else {
        current_exe
    };

    // TODO: remove this line when we open source
    let url = url.replace(
        "/systeminit/si/releases/download/",
        "/stack72/test-download/releases/download/",
    );

    let req = reqwest::get(url).await?;
    if req.status().as_u16() != 200 {
        println!(
            "Unable to update: API returned an expected status code {}",
            req.status()
        );
        return Err(SiCliError::UnableToDownloadUpdate(req.status().as_u16()));
    }

    // Note: temp folders will be leaked if destructors don't run
    let tempdir = tempfile::TempDir::new()?;

    println!("Downloading new binary");
    let bytes = req.bytes().await?;
    let bytes = GzDecoder::new(Cursor::new(bytes));

    let path = tempdir.path().to_owned();
    tokio::task::spawn_blocking(move || {
        let mut archive = tar::Archive::new(bytes);
        archive.unpack(path)?;
        Ok::<(), SiCliError>(())
    })
    .await??;

    println!("Overwriting current binary");
    tokio::fs::rename(tempdir.path().join("si"), exe_path).await?;

    println!("Binary updated!");

    Ok(())
}

impl AppState {
    pub async fn update(
        &self,
        current_version: &str,
        host: Option<&str>,
        skip_confirmation: bool,
        only_binary: bool,
    ) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "update-launcher"}),
        );
        invoke(self, current_version, host, skip_confirmation, only_binary).await?;
        Ok(())
    }

    pub async fn find(&self, current_version: &str, host: Option<&str>) -> CliResult<Update> {
        let host = if let Some(host) = host { host } else { HOST };

        let req = reqwest::get(format!("{host}/github/containers/latest")).await?;
        if req.status().as_u16() != 200 {
            return Err(SiCliError::UnableToFetchContainersUpdate(
                req.status().as_u16(),
            ));
        }

        let current_containers = get_container_details().await?;

        let mut containers = Vec::new();
        let latest_containers: Vec<LatestContainer> = req.json().await?;
        'outer: for latest in latest_containers {
            for current in &current_containers {
                if current.image != format!("{}/{}", latest.namespace, latest.repository) {
                    continue;
                }

                if current.git_sha != latest.git_sha {
                    containers.push(latest);
                }
                continue 'outer;
            }

            // If we don't have the container locally we should fetch it
            containers.push(latest);
        }

        let req = reqwest::get(format!("{host}/github/releases/latest")).await?;
        if req.status().as_u16() != 200 {
            return Err(SiCliError::UnableToFetchSiUpdate(req.status().as_u16()));
        }

        let mut si = None;
        let release: Release = req.json().await?;

        // The binary tags are now in the format bin/si/binary/version
        // this future proofs us to ensure that we don't have different binaries
        // of the same name
        if release.version != format!("bin/si/binary/{}", current_version) {
            si = Some(release);
        }

        Ok(Update { containers, si })
    }
}

async fn invoke(
    app: &AppState,
    current_version: &str,
    host: Option<&str>,
    skip_confirmation: bool,
    only_binary: bool,
) -> CliResult<()> {
    #[cfg(target_os = "linux")]
    let our_os = "Linux";

    #[cfg(all(not(target_os = "linux"), target_vendor = "apple"))]
    let our_os = "Darwin";

    let update = app.find(current_version, host).await?;
    if !only_binary {
        for image in &update.containers {
            println!(
                "Container update found for {}/{}",
                image.namespace, image.repository
            );
        }
    }

    if let Some(update) = &update.si {
        let version = &update.version;
        println!("Launcher update found: from {current_version} to {version}",);
    }

    let ans = if update.si.is_some() || (!only_binary && !update.containers.is_empty()) {
        if skip_confirmation {
            Ok(true)
        } else {
            let mut prompt = "Are you sure you want to update".to_owned();
            if update.si.is_some() {
                prompt.push_str(" the binary");
                if !update.containers.is_empty() {
                    prompt.push_str(" and");
                }
            }

            if !only_binary && !update.containers.is_empty() {
                println!(
                    "\n{}",
                    "Updating the containers will destroy your data!".red()
                );
                prompt.push_str(" the containers listed above");
            }

            prompt.push('?');

            Confirm::new(&prompt).with_default(false).prompt()
        }
    } else {
        println!("No updates found!");
        return Ok(());
    };

    match ans {
        Ok(true) => {
            if !only_binary && !update.containers.is_empty() {
                app.stop().await?;

                let docker = Docker::unix("//var/run/docker.sock");
                for container in &update.containers {
                    let container_name = format!("local-{0}-1", container.repository);
                    let container_summary =
                        get_existing_container(&docker, container_name.clone()).await?;
                    if let Some(container_summary) = container_summary {
                        delete_container(&docker, container_summary, container_name.clone())
                            .await?;
                        cleanup_image(&docker, container_name.to_string()).await?;
                    }
                }

                app.start().await?;
            }

            if let Some(update) = &update.si {
                // Note: we can't download from here because the repo is private, we should
                // automate the download + replace of the current binary after we go public
                // (or start caching the binaries in auth api)
                #[cfg(target_arch = "x86_64")]
                let arch = "x86_64";

                #[cfg(target_arch = "aarch64")]
                let arch = "aarch64";

                for asset in &update.assets {
                    if asset.name.to_lowercase().contains(arch)
                        && asset.name.to_lowercase().contains(&our_os.to_lowercase())
                    {
                        update_current_binary(&asset.url).await?;
                    }
                }
            }
        }
        Ok(false) => println!("See ya later ;)"),
        Err(err) => println!("Error: Try again later!: {err}"),
    }

    Ok(())
}
