use crate::containers::DockerClient;
use crate::key_management::{
    ensure_encryption_keys, ensure_jwt_public_signing_key, format_credentials_for_veritech,
    get_si_data_dir, get_user_email,
};
use crate::state::AppState;
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::opts::{ContainerCreateOpts, HostPort, PublishPort};

impl AppState {
    pub async fn start(&self, docker: &DockerClient) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "start-system"}),
        );
        invoke(self, docker, self.is_preview()).await?;
        Ok(())
    }
}

async fn invoke(app: &AppState, docker: &DockerClient, is_preview: bool) -> CliResult<()> {
    app.configure(false).await?;
    app.check(docker, false).await?;
    app.install(docker).await?;

    if is_preview {
        println!("Started the following containers:");
    }

    ensure_encryption_keys().await?;
    ensure_jwt_public_signing_key().await?;
    let si_data_dir = get_si_data_dir().await?;

    for name in CONTAINER_NAMES.iter() {
        let container = format!("systeminit/{0}", name);
        let container_name = format!("local-{0}-1", name);
        if container == "systeminit/otelcol" {
            let container_summary = docker
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                let non_running_container = docker.containers().get(existing.id.as_ref().unwrap());
                non_running_container.start().await?;
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
            let create_opts = ContainerCreateOpts::builder()
                .name(container_name.clone())
                .image(format!("{0}:stable", container.clone()))
                .expose(PublishPort::tcp(4317), HostPort::new(4317))
                .expose(PublishPort::tcp(55679), HostPort::new(55679))
                .links(["local-jaeger-1:jaeger"])
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if container == "systeminit/jaeger" {
            let container_summary = docker
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                let non_running_container = docker.containers().get(existing.id.as_ref().unwrap());
                non_running_container.start().await?;
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
            let create_opts = ContainerCreateOpts::builder()
                .name(container_name.clone())
                .image(format!("{0}:stable", container.clone()))
                .expose(PublishPort::tcp(5317), HostPort::new(5317))
                .expose(PublishPort::tcp(16686), HostPort::new(16686))
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if container == "systeminit/nats" {
            let container_summary = docker
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                let non_running_container = docker.containers().get(existing.id.as_ref().unwrap());
                non_running_container.start().await?;
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
            let create_opts = ContainerCreateOpts::builder()
                .name(container_name.clone())
                .image(format!("{0}:stable", container.clone()))
                .expose(PublishPort::tcp(4222), HostPort::new(4222))
                .command(vec!["--config", "nats-server.conf", "-DVV"])
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if container == "systeminit/postgres" {
            let container_summary = docker
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                let non_running_container = docker.containers().get(existing.id.as_ref().unwrap());
                non_running_container.start().await?;
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
            let create_opts = ContainerCreateOpts::builder()
                .name(container_name.clone())
                .image(format!("{0}:stable", container.clone()))
                .expose(PublishPort::tcp(5432), HostPort::new(5432))
                .env(vec![
                    "POSTGRES_PASSWORD=bugbear",
                    "PGPASSWORD=bugbear",
                    "POSTGRES_USER=si",
                    "POSTGRES_DB=si",
                ])
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if container == "systeminit/council" {
            let container_summary = docker
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                let non_running_container = docker.containers().get(existing.id.as_ref().unwrap());
                non_running_container.start().await?;
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
            let create_opts = ContainerCreateOpts::builder()
                .name(container_name.clone())
                .image(format!("{0}:stable", container.clone()))
                .links(vec!["local-nats-1:nats", "local-otelcol-1:otelcol"])
                .env(vec![
                    "SI_COUNCIL__NATS__URL=nats",
                    "OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol:4317",
                ])
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if container == "systeminit/veritech" {
            let container_summary = docker
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Deleting existing container {0}", container_name.clone());
                let non_running_container = docker.containers().get(existing.id.as_ref().unwrap());
                non_running_container.delete().await?;
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
            let mut env_vars = vec![
                "SI_VERITECH__NATS__URL=nats".to_string(),
                "OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol:4317".to_string(),
            ];
            env_vars.append(&mut veritech_credentials);
            let create_opts = ContainerCreateOpts::builder()
                .name(container_name.clone())
                .image(format!("{0}:stable", container.clone()))
                .links(vec!["local-nats-1:nats", "local-otelcol-1:otelcol"])
                .env(env_vars)
                .volumes([format!("{}:/run/cyclone", si_data_dir.display())])
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if container == "systeminit/pinga" {
            let container_summary = docker
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                let non_running_container = docker.containers().get(existing.id.as_ref().unwrap());
                non_running_container.start().await?;
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

            let create_opts = ContainerCreateOpts::builder()
                .name(container_name.clone())
                .image(format!("{0}:stable", container.clone()))
                .links(vec![
                    "local-nats-1:nats",
                    "local-postgres-1:postgres",
                    "local-otelcol-1:otelcol",
                ])
                .env(vec![
                    "SI_PINGA__NATS__URL=nats",
                    "SI_PINGA__PG__HOSTNAME=postgres",
                    "OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol:4317",
                ])
                .volumes([format!("{}:/run/pinga", si_data_dir.display())])
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if container == "systeminit/sdf" {
            let container_summary = docker
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                let non_running_container = docker.containers().get(existing.id.as_ref().unwrap());
                non_running_container.start().await?;
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
            let create_opts = ContainerCreateOpts::builder()
                .name(container_name.clone())
                .image(format!("{0}:stable", container.clone()))
                .links(vec![
                    "local-nats-1:nats",
                    "local-postgres-1:postgres",
                    "local-otelcol-1:otelcol",
                ])
                .env(vec![
                    "SI_SDF__NATS__URL=nats",
                    "SI_SDF__PG__HOSTNAME=postgres",
                    "OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol:4317",
                ])
                .network_mode("bridge")
                .expose(PublishPort::tcp(5156), HostPort::new(5156))
                .volumes([
                    format!(
                        "{}:/run/sdf/cyclone_encryption.key",
                        si_data_dir.join("cyclone_encryption.key").display()
                    ),
                    format!(
                        "{}:/run/sdf/jwt_signing_public_key.pem",
                        si_data_dir.join("jwt_signing_public_key.pem").display()
                    ),
                ])
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if container == "systeminit/web" {
            let container_summary = docker
                .get_existing_container(container_name.clone())
                .await?;
            if let Some(existing) = container_summary {
                // it means we have an existing container
                // If it's running, we have nothing to do here
                if existing.state.as_ref().unwrap() == "running" {
                    continue;
                }

                println!("Starting existing {0}", container_name.clone());
                let non_running_container = docker.containers().get(existing.id.as_ref().unwrap());
                non_running_container.start().await?;
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
            let create_opts = ContainerCreateOpts::builder()
                .name(container_name.clone())
                .image(format!("{0}:stable", container.clone()))
                .links(vec!["local-sdf-1:sdf"])
                .env(["SI_LOG=trace"])
                .network_mode("bridge")
                .expose(PublishPort::tcp(8080), HostPort::new(8080))
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
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
