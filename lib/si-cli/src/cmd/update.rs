use crate::containers::get_container_details;
use crate::{CliResult, SiCliError};
use colored::Colorize;
use inquire::Confirm;
use serde::Deserialize;
use si_posthog::PosthogClient;

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
            if current.git_sha == latest.git_sha {
                containers.push(latest);
                continue 'outer;
            }
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

    let req = reqwest::get(url).await?;
    if req.status().as_u16() != 200 {
        println!(
            "Unable to update: API returned an expected status code {}",
            req.status()
        );
        return Err(SiCliError::UnableToDownloadUpdate(req.status().as_u16()));
    }

    // Note: named temp files may be leaked if destructors don't run
    // ideally we would use tempfile::tempfile(), but it doesn't have a path so it
    // can't be renamed atomically to replace the current binary without corrupting it
    //
    // We could have a folder in tmp with updates so we can delete it every time we start one,
    // but this leak is not a significant issue, so no need to do it _right now_
    let tempfile = tempfile::NamedTempFile::new()?;

    println!("Downloading new binary");
    let bytes = req.bytes().await?;
    tokio::fs::write(tempfile.path(), bytes).await?;

    println!("Overwriting current binary");
    tokio::fs::rename(tempfile.path(), current_exe).await?;

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
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
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
            println!("\nThat's awesome! Let's do this");

            if !only_binary {
                dbg!(update.containers);
            }

            if let Some(update) = &update.si {
                // Note: we can't download from here because the repo is private, we should
                // automate the download + replace of the current binary after we go public
                // (or start caching the binaries in auth api)
                for asset in &update.assets {
                    if asset.name.to_lowercase().contains(&our_os.to_lowercase()) {
                        println!("Download the new version here: {}\n", asset.url);
                        // update_current_binary(&asset.url).await?;
                    }
                }
            }
        }
        Ok(false) => println!("See ya later ;)"),
        Err(err) => println!("Error: Try again later!: {err}"),
    }

    Ok(())
}
