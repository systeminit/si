use crate::cmd::{check, install};
use crate::containers::has_existing_container;
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::opts::{ContainerCreateOpts, HostPort, PublishPort};
use docker_api::Docker;
use si_posthog::PosthogClient;

pub async fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "start-system", "mode": mode}),
    );

    check::invoke(posthog_client, mode.clone(), false).await?;
    install::invoke(posthog_client, mode.clone()).await?;

    let docker = Docker::unix("//var/run/docker.sock");

    for name in CONTAINER_NAMES.iter() {
        let container = format!("systeminit/{0}", name);
        let container_name = format!("dev-{0}-1", name);
        if container == "systeminit/otelcol" {
            let running_container =
                has_existing_container(&docker, container_name.clone(), true).await?;

            if !running_container {
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
                    .build();

                let container = docker.containers().create(&create_opts).await?;
                container.start().await?;
            }
        }
        if container == "systeminit/jaeger" {
            let running_container =
                has_existing_container(&docker, "dev-jaeger-1".to_string(), true).await?;

            if !running_container {
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
    }

    println!("All system components running... System Initiative is alive!");

    Ok(())
}
