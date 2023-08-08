use crate::containers::{cleanup_image, get_container_details, has_existing_container};
use crate::key_management::get_user_email;
use crate::{CliResult, SiCliError};
use colored::Colorize;
use docker_api::Docker;
use flate2::read::GzDecoder;
use inquire::Confirm;
use serde::Deserialize;
use si_posthog::PosthogClient;
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
    pub name: String,
    pub git_sha: String,
    pub digest: String,
}

#[derive(Debug)]
pub struct Update {
    pub containers: Vec<LatestContainer>,
    pub si: Option<Release>,
}

static HOST: &str = "https://auth-api.systeminit.com";

pub async fn find(current_version: &str, host: Option<&str>) -> CliResult<Update> {
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
    if release.version != current_version {
        si = Some(release);
    }

    Ok(Update { containers, si })
}

pub async fn update_current_binary(url: &str) -> CliResult<()> {
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

pub async fn invoke(
    current_version: &str,
    host: Option<&str>,
    posthog_client: &PosthogClient,
    mode: String,
    skip_confirmation: bool,
    only_binary: bool,
) -> CliResult<()> {
    let email = get_user_email().await?;
    let _ = posthog_client.capture(
        "si-command",
        email,
        serde_json::json!({"name": "update-launcher", "mode": mode}),
    );

    #[cfg(target_os = "linux")]
    let our_os = "Linux";

    #[cfg(all(not(target_os = "linux"), target_vendor = "apple"))]
    let our_os = "Darwin";

    let update = find(current_version, host).await?;
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
                crate::cmd::stop::invoke(posthog_client, mode.clone(), false).await?;

                let docker = Docker::unix("//var/run/docker.sock");
                for container in &update.containers {
                    let container_name = format!("dev-{0}-1", container.repository);
                    has_existing_container(&docker, container_name, true).await?;
                    cleanup_image(&docker, container.repository.to_owned()).await?;
                }

                crate::cmd::start::invoke(posthog_client, mode.clone(), false).await?;
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
