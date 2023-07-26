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

static HOST: &str = "https://auth-api.systeminit.com";

pub async fn find(current_version: &str, host: Option<&str>) -> CliResult<Option<Release>> {
    let host = if let Some(host) = host { host } else { HOST };

    let req = reqwest::get(format!("{host}/github/releases")).await?;
    if req.status().as_u16() != 200 {
        println!("API returned an expected status code: {}", req.status());
        return Ok(None);
    }

    let releases: Vec<Release> = req.json().await?;
    if let Some(release) = releases.first() {
        if release.version != current_version {
            return Ok(Some(release.clone()));
        }
    }
    Ok(None)
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
    dbg!(&current_exe);
    tokio::fs::rename(tempfile.path(), current_exe).await?;

    println!("Binary updated!");

    Ok(())
}

pub async fn invoke(
    current_version: &str,
    host: Option<&str>,
    posthog_client: &PosthogClient,
    mode: String,
    skip_check: bool,
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
    if let Some(update) = &update {
        let version = &update.version;
        println!("Update found: from {current_version} to {version}\n",);
    }

    let ans = if let Some(update) = update {
        if skip_check {
            Ok((true, update))
        } else {
            println!("{}", "Updating the launcher will destroy your data!".red());
            Confirm::new("Are you sure you want to update this launcher?")
                .with_default(false)
                .prompt()
                .map(|ans| (ans, update))
        }
    } else {
        return Ok(());
    };

    match ans {
        Ok((true, update)) => {
            println!("\nThat's awesome! Let's do this");
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
        Ok((false, _)) => println!("See ya later ;)"),
        Err(err) => println!("Error: Try again later!: {err}"),
    }

    let _ = get_container_details().await?;

    Ok(())
}
