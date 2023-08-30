use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, SiCliError};
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;
use std::time::Duration;
use std::{env, thread};

impl AppState {
    pub async fn launch(&self, launch_metrics: bool) -> CliResult<()> {
        invoke(launch_metrics, self.web_host(), self.web_port(), self.sdf_host(), self.sdf_port()).await?;
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "launch-ui"}),
        );
        Ok(())
    }
}

async fn invoke(launch_metrics: bool, web_host: String, web_port: u32, sdf_host: String, sdf_port: u32) -> CliResult<()> {
    let path = if launch_metrics {
        "http://localhost:16686".to_string()
    } else {
        format!("http://{0}:{1}", web_host, web_port)
    };

    if path == format!("http://{0}:{1}", web_host, web_port) {
        check_web(web_host, web_port).await?;
        check_sdf(sdf_host, sdf_port).await?;
    }

    let output = if cfg!(target_os = "macos") {
        Command::new("open").arg(path.clone()).output()
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open").arg(path.clone()).output()
    } else {
        // This should NEVER get called but I added it in here just incase
        return Err(SiCliError::UnsupportedOperatingSystem(
            env::consts::OS.to_string(),
        ));
    };

    if let Err(_err) = output {
        return Err(SiCliError::FailToLaunch(path));
    }

    println!("Successfully opened URL: {}", path);
    Ok(())
}

async fn check_web(web_host: String, web_port: u32) -> CliResult<()> {
    let path = format!("http://{0}:{1}", web_host, web_port);
    let resp = reqwest::get(path).await;
    if let Err(_e) = resp {
        return Err(SiCliError::WebPortal());
    }

    Ok(())
}

async fn check_sdf(sdf_host: String, sdf_port: u32) -> CliResult<()> {
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let sdf_path = format!("http://{0}:{1}/api/", sdf_host, sdf_port);

    let mut is_ready = false;
    let sdf_path_clone = sdf_path.clone(); // Clone sdf_path for use inside the async block

    let h = tokio::spawn(async move {
        let count = 200;
        let pb = ProgressBar::new(count);
        pb.set_style(spinner_style.clone());
        while !is_ready {
            loop {
                match reqwest::get(&sdf_path_clone).await {
                    Ok(x) => {
                        if x.status().as_u16() == 200 {
                            is_ready = true;
                            break;
                        }
                    }
                    Err(_e) => {
                        pb.set_message("{e}");
                        pb.set_message("Almost ready, waiting for migrations to complete...");
                        pb.inc(1);
                        thread::sleep(Duration::new(10, 0));
                    }
                }
            }
        }
    });

    h.await?;

    Ok(())
}

