use crate::cmd::{check, install};
use crate::containers::{delete_existing_container_id, get_non_running_containers};
use crate::CliResult;
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

    let non_running_containers = get_non_running_containers().await?;
    let docker = Docker::unix("//var/run/docker.sock");

    for non_running_container in &non_running_containers {
        if non_running_container.as_str() == "systeminit/otelcol" {
            delete_existing_container_id(&docker, "dev-otelcol-1".to_string()).await?;

            println!("Starting systeminit/otelcol:stable as dev-otelcol-1");
            let create_opts = ContainerCreateOpts::builder()
                .name("dev-otelcol-1")
                .image("systeminit/otelcol:stable")
                .expose(PublishPort::tcp(4317), HostPort::new(4317))
                .expose(PublishPort::tcp(55679), HostPort::new(55679))
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if non_running_container.as_str() == "systeminit/jaeger" {
            delete_existing_container_id(&docker, "dev-jaeger-1".to_string()).await?;

            println!("Starting systeminit/jaeger:stable as dev-jaeger-1");
            let create_opts = ContainerCreateOpts::builder()
                .name("dev-jaeger-1")
                .image("systeminit/jaeger:stable")
                .expose(PublishPort::tcp(5317), HostPort::new(5317))
                .expose(PublishPort::tcp(16686), HostPort::new(16686))
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if non_running_container.as_str() == "systeminit/nats" {
            delete_existing_container_id(&docker, "dev-nats-1".to_string()).await?;

            println!("Starting systeminit/nats:stable as dev-nats-1");
            let create_opts = ContainerCreateOpts::builder()
                .name("dev-nats-1")
                .image("systeminit/nats:stable")
                .expose(PublishPort::tcp(4222), HostPort::new(4222))
                .command(vec!["--config", "nats-server.conf", "-DVV"])
                .build();

            let container = docker.containers().create(&create_opts).await?;
            container.start().await?;
        }
        if non_running_container.as_str() == "systeminit/postgres" {
            delete_existing_container_id(&docker, "dev-postgres-1".to_string()).await?;

            println!("Starting systeminit/postgres:stable as dev-nats-1");
            let create_opts = ContainerCreateOpts::builder()
                .name("dev-postgres-1")
                .image("systeminit/postgres:stable")
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

    println!("All system components running... System Initiative is alive!");

    Ok(())
}
