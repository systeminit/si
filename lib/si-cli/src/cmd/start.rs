use crate::key_management::{
    ensure_encryption_keys, ensure_jwt_public_signing_key, format_credentials_for_veritech,
    get_si_data_dir, get_user_email,
};
use crate::state::AppState;
use crate::{CliResult, CONTAINER_NAMES};

impl AppState {
    pub async fn start(&self) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "start-system"}),
        );
        invoke(self, self.is_preview()).await?;
        Ok(())
    }
}

async fn invoke(app: &AppState, is_preview: bool) -> CliResult<()> {
    app.configure(false).await?;
    app.check(false).await?;
    app.install().await?;

    if is_preview {
        println!("Started the following containers:");
    }

    ensure_encryption_keys().await?;
    ensure_jwt_public_signing_key().await?;
    let si_data_dir = get_si_data_dir().await?;

    app.container_engine().create_network().await?;

    for name in CONTAINER_NAMES.iter() {
        let container = format!("systeminit/{0}", name);
        let container_name = format!("local-{0}-1", name);
        if container == "systeminit/otelcol" {
            let container_summary = app
                .container_engine()
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                app.container_engine()
                    .start_container(existing.id.as_ref().unwrap().to_string())
                    .await?;
                continue;
            }

            if is_preview {
                println!(
                    "{0}:stable as {1}",
                    container.clone(),
                    container_name.clone()
                );
                continue;
            }
            println!(
                "Starting {0}:stable as {1}",
                container.clone(),
                container_name.clone()
            );

            app.container_engine()
                .create_otelcol(container_name.clone(), container.clone())
                .await?;
        }
        if container == "systeminit/jaeger" {
            let container_summary = app
                .container_engine()
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                app.container_engine()
                    .start_container(existing.id.as_ref().unwrap().to_string())
                    .await?;
                continue;
            }

            if is_preview {
                println!(
                    "{0}:stable as {1}",
                    container.clone(),
                    container_name.clone()
                );
                continue;
            }
            println!(
                "Starting {0}:stable as {1}",
                container.clone(),
                container_name.clone()
            );

            app.container_engine()
                .create_jaeger(container_name.clone(), container.clone())
                .await?;
        }
        if container == "systeminit/nats" {
            let container_summary = app
                .container_engine()
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                app.container_engine()
                    .start_container(existing.id.as_ref().unwrap().to_string())
                    .await?;
                continue;
            }

            if is_preview {
                println!(
                    "{0}:stable as {1}",
                    container.clone(),
                    container_name.clone()
                );
                continue;
            }
            println!(
                "Starting {0}:stable as {1}",
                container.clone(),
                container_name.clone()
            );

            app.container_engine()
                .create_nats(container_name.clone(), container.clone())
                .await?;
        }
        if container == "systeminit/postgres" {
            let container_summary = app
                .container_engine()
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                app.container_engine()
                    .start_container(existing.id.as_ref().unwrap().to_string())
                    .await?;
                continue;
            }

            if is_preview {
                println!(
                    "{0}:stable as {1}",
                    container.clone(),
                    container_name.clone()
                );
                continue;
            }
            println!(
                "Starting {0}:stable as {1}",
                container.clone(),
                container_name.clone()
            );

            app.container_engine()
                .create_postgres(container_name.clone(), container.clone())
                .await?;
        }
        if container == "systeminit/council" {
            let container_summary = app
                .container_engine()
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                app.container_engine()
                    .start_container(existing.id.as_ref().unwrap().to_string())
                    .await?;
                continue;
            }

            if is_preview {
                println!(
                    "{0}:stable as {1}",
                    container.clone(),
                    container_name.clone()
                );
                continue;
            }
            println!(
                "Starting {0}:stable as {1}",
                container.clone(),
                container_name.clone()
            );

            app.container_engine()
                .create_council(container_name.clone(), container.clone())
                .await?;
        }
        if container == "systeminit/veritech" {
            let container_summary = app
                .container_engine()
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                app.container_engine()
                    .delete_container(
                        existing.id.as_ref().unwrap().to_string(),
                        container_name.clone(),
                    )
                    .await?;
            }

            if is_preview {
                println!(
                    "{0}:stable as {1}",
                    container.clone(),
                    container_name.clone()
                );
                continue;
            }
            println!(
                "Starting {0}:stable as {1}",
                container.clone(),
                container_name.clone()
            );
            let mut veritech_credentials = format_credentials_for_veritech().await?;

            app.container_engine()
                .create_veritech(
                    container_name.clone(),
                    container.clone(),
                    veritech_credentials.as_mut(),
                    si_data_dir.clone(),
                    app.with_function_debug_logs(),
                )
                .await?;
        }
        if container == "systeminit/pinga" {
            let container_summary = app
                .container_engine()
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                app.container_engine()
                    .start_container(existing.id.as_ref().unwrap().to_string())
                    .await?;
                continue;
            }

            if is_preview {
                println!(
                    "{0}:stable as {1}",
                    container.clone(),
                    container_name.clone()
                );
                continue;
            }
            println!(
                "Starting {0}:stable as {1}",
                container.clone(),
                container_name.clone()
            );

            app.container_engine()
                .create_pinga(
                    container_name.clone(),
                    container.clone(),
                    si_data_dir.clone(),
                )
                .await?;
        }
        if container == "systeminit/sdf" {
            let container_summary = app
                .container_engine()
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                let mut needs_recreated = false;
                if let Some(ports) = existing.ports {
                    if !ports.is_empty() {
                        let port = ports.first().unwrap().clone();
                        let public_port = port.public_port.unwrap_or(0);
                        let ip = port.ip.clone().unwrap_or("".to_string());

                        if public_port as u32 != app.sdf_port() || ip != app.sdf_host() {
                            needs_recreated = true;
                        }
                    }
                } else {
                    // No ports suggest that the container isn't in a started state
                    needs_recreated = true
                }

                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" && !needs_recreated {
                    continue;
                }

                if needs_recreated {
                    println!(
                        "Container Port Mappings have changed for {0} so recreating",
                        container_name.clone()
                    );

                    if existing.state.as_ref().unwrap() == "running" {
                        app.container_engine()
                            .stop_container(existing.id.as_ref().unwrap().to_string())
                            .await?;
                    }

                    app.container_engine()
                        .delete_container(
                            existing.id.as_ref().unwrap().to_string(),
                            container_name.clone(),
                        )
                        .await?;
                } else {
                    println!("Starting existing {0}", container_name.clone());
                    app.container_engine()
                        .start_container(existing.id.as_ref().unwrap().to_string())
                        .await?;
                    continue;
                }
            }

            if is_preview {
                println!(
                    "{0}:stable as {1}",
                    container.clone(),
                    container_name.clone()
                );
                continue;
            }
            println!(
                "Starting {0}:stable as {1}",
                container.clone(),
                container_name.clone()
            );

            app.container_engine()
                .create_sdf(
                    container_name.clone(),
                    container.clone(),
                    app.sdf_host(),
                    app.sdf_port(),
                    si_data_dir.clone(),
                )
                .await?;
        }
        if container == "systeminit/web" {
            let container_summary = app
                .container_engine()
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                let mut needs_recreated = false;
                if let Some(ports) = existing.ports {
                    if !ports.is_empty() {
                        let port = ports.first().unwrap().clone();
                        let public_port = port.public_port.unwrap_or(0);
                        let ip = port.ip.clone().unwrap_or("".to_string());

                        if public_port as u32 != app.web_port() || ip != app.web_host() {
                            needs_recreated = true;
                        }
                    }
                } else {
                    // No ports suggest that the container isn't in a started state
                    needs_recreated = true
                }

                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" && !needs_recreated {
                    continue;
                }

                if needs_recreated {
                    println!(
                        "Container Port Mappings have changed for {0} so recreating",
                        container_name.clone()
                    );

                    if existing.state.as_ref().unwrap() == "running" {
                        app.container_engine()
                            .stop_container(existing.id.as_ref().unwrap().to_string())
                            .await?;
                    }

                    app.container_engine()
                        .delete_container(
                            existing.id.as_ref().unwrap().to_string(),
                            container_name.clone(),
                        )
                        .await?;
                } else {
                    println!("Starting existing {0}", container_name.clone());
                    app.container_engine()
                        .start_container(existing.id.as_ref().unwrap().to_string())
                        .await?;
                    continue;
                }
            }

            if is_preview {
                println!(
                    "{0}:stable as {1}",
                    container.clone(),
                    container_name.clone()
                );
                continue;
            }
            println!(
                "Starting {0}:stable as {1}",
                container.clone(),
                container_name.clone()
            );

            app.container_engine()
                .create_web(
                    container_name.clone(),
                    container.clone(),
                    app.web_host(),
                    app.web_port(),
                )
                .await?;
        }
    }

    if !is_preview {
        println!("All system components running... System Initiative is alive!");
        println!(
            "\nYou can now use the `si launch` command to open the System Initiative web portal"
        )
    }

    Ok(())
}
