use crate::CliResult;
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

pub async fn find() -> CliResult<Option<Release>> {
    // Note: we currently can't deploy auth api to production, so we are only supporting it locally for now
    let host = "http://localhost:9001";
    let req = reqwest::get(format!("{host}/github/releases")).await?;
    let releases: Vec<Release> = req.json().await?;
    if let Some(release) = releases.first() {
        if release.version != CURRENT_VERSION {
            return Ok(Some(release.clone()));
        }
    }
    Ok(None)
}

const CURRENT_VERSION: &str = "0.9";

pub async fn invoke(
    posthog_client: &PosthogClient,
    mode: String,
    skip_check: bool,
) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "update-launcher", "mode": mode}),
    );

    let ans = if skip_check {
        Ok(true)
    } else {
        Confirm::new("Are you sure you want to update this launcher?")
            .with_default(false)
            .with_help_message("Please Note: No container data is backed up during update!")
            .prompt()
    };

    match ans {
        Ok(true) => {
            println!("That's awesome! Let's do this");
            #[cfg(target_os = "linux")]
            let our_os = "Linux";

            #[cfg(all(not(target_os = "linux"), target_vendor = "apple"))]
            let our_os = "Darwin";

            if let Some(update) = find().await? {
                println!(
                    "\nUpdate found: from {} to {}\n",
                    CURRENT_VERSION, update.version
                );
                // Note: we can't download from here because the repo is private, we should
                // automate the download and replace the current binary after we go public
                // (or start caching the binaries in auth api)
                for asset in &update.assets {
                    if asset.name.to_lowercase().contains(&our_os.to_lowercase()) {
                        println!("Download the new version here: {}\n", asset.url);
                    }
                }
            }
        }
        Ok(false) => println!("See ya later ;)"),
        Err(err) => println!("Error: Try again later!: {err}"),
    }

    Ok(())
}
