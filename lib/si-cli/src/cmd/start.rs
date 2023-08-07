use crate::cmd::{check, configure, install};
use crate::containers::has_existing_container;
use crate::key_management::{
    ensure_encryption_keys, ensure_jwt_public_signing_key, get_si_data_dir,
    get_veritech_credentials,
};
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::opts::{ContainerCreateOpts, HostPort, PublishPort};
use docker_api::Docker;
use si_posthog::PosthogClient;

pub async fn invoke(
    posthog_client: &PosthogClient,
    mode: String,
    is_preview: bool,
) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "start-system", "mode": mode}),
    );

    check::invoke(posthog_client, mode.clone(), false, is_preview).await?;
    install::invoke(posthog_client, mode.clone(), is_preview).await?;
    configure::invoke(posthog_client, mode.clone(), is_preview, false).await?;

    let docker = Docker::unix("//var/run/docker.sock");

    if is_preview {
        println!("Started the following containers:");
    }

    ensure_encryption_keys().await?;
    ensure_jwt_public_signing_key().await?;
    let si_data_dir = get_si_data_dir().await?;

    for name in CONTAINER_NAMES.iter() {
        let container = format!("systeminit/{0}", name);
        let container_name = format!("dev-{0}-1", name);
        if container == "systeminit/otelcol" {
            let running_container =
                has_existing_container(&docker, container_name.clone(), true).await?;

            if !running_container {
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
                    .links(["dev-jaeger-1:jaeger"])
                    .build();

                let container = docker.containers().create(&create_opts).await?;
                container.start().await?;
            }
        }
        if container == "systeminit/jaeger" {
            let running_container =
                has_existing_container(&docker, "dev-jaeger-1".to_string(), true).await?;

            if !running_container {
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
        }
        if container == "systeminit/nats" {
            let running_container =
                has_existing_container(&docker, "dev-nats-1".to_string(), true).await?;

            if !running_container {
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
        }
        if container == "systeminit/postgres" {
            let running_container =
                has_existing_container(&docker, "dev-postgres-1".to_string(), true).await?;

            if !running_container {
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
        }
        if container == "systeminit/council" {
            let running_container =
                has_existing_container(&docker, "dev-council-1".to_string(), true).await?;

            if !running_container {
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
                    .links(vec!["dev-nats-1:nats", "dev-otelcol-1:otelcol"])
                    .env(vec![
                        "SI_COUNCIL__NATS__URL=nats",
                        "OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol:4317",
                    ])
                    .build();

                let container = docker.containers().create(&create_opts).await?;
                container.start().await?;
            }
        }
        if container == "systeminit/veritech" {
            let running_container =
                has_existing_container(&docker, "dev-veritech-1".to_string(), true).await?;

            if !running_container {
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
                let mut veritech_credentials = get_veritech_credentials().await?;
                let mut env_vars = vec![
                    "SI_VERITECH__NATS__URL=nats".to_string(),
                    "OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol:4317".to_string(),
                ];
                env_vars.append(&mut veritech_credentials);
                let create_opts = ContainerCreateOpts::builder()
                    .name(container_name.clone())
                    .image(format!("{0}:stable", container.clone()))
                    .links(vec!["dev-nats-1:nats", "dev-otelcol-1:otelcol"])
                    .env(env_vars)
                    .volumes([format!("{}:/run/cyclone", si_data_dir.display())])
                    .build();

                let container = docker.containers().create(&create_opts).await?;
                container.start().await?;
            }
        }
        if container == "systeminit/pinga" {
            let running_container =
                has_existing_container(&docker, "dev-pinga-1".to_string(), true).await?;

            if !running_container {
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
                        "dev-nats-1:nats",
                        "dev-postgres-1:postgres",
                        "dev-otelcol-1:otelcol",
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
        }
        if container == "systeminit/sdf" {
            let running_container =
                has_existing_container(&docker, "dev-sdf-1".to_string(), true).await?;

            if !running_container {
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
                        "dev-nats-1:nats",
                        "dev-postgres-1:postgres",
                        "dev-otelcol-1:otelcol",
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
        }
        if container == "systeminit/web" {
            let running_container =
                has_existing_container(&docker, "dev-web-1".to_string(), true).await?;

            if !running_container {
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
                    .links(vec!["dev-sdf-1:sdf"])
                    .env(["SI_LOG=trace"])
                    .network_mode("bridge")
                    .expose(PublishPort::tcp(8080), HostPort::new(8080))
                    .build();

                let container = docker.containers().create(&create_opts).await?;
                container.start().await?;
            }
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
