use crate::cmd::{check, install};
use crate::containers::get_non_running_containers;
use crate::CliResult;
use docker_api::opts::{
    ContainerCreateOpts, ContainerFilter, ContainerListOpts, HostPort, PublishPort,
};
use docker_api::Docker;
use si_posthog::PosthogClient;

pub async fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "start-system", "mode": mode}),
    );

    check::invoke(posthog_client, mode.clone()).await?;

    install::invoke(posthog_client, mode.clone()).await?;

    let non_running_containers = get_non_running_containers().await?;

    let docker = Docker::unix("//var/run/docker.sock");

    for non_running_container in &non_running_containers {
        if non_running_container.as_str() == "systeminit/otelcol" {
            let filter = ContainerFilter::Name("dev-otelcol-1".to_string());
            let list_opts = ContainerListOpts::builder()
                .filter([filter])
                .all(true)
                .build();
            let otel_containers = docker.containers().list(&list_opts).await?;
            if !otel_containers.is_empty() {
                let existing_id = otel_containers.first().unwrap().id.as_ref().unwrap();
                println!("Found an existing otel container: {}", *existing_id);
                docker.containers().get(existing_id).delete().await?;
            }

            println!("Starting systeminit/otelcol:stable as dev-otelcol-1");
            let otel_container_ops = ContainerCreateOpts::builder()
                .name("dev-otelcol-1")
                .image("systeminit/otelcol:stable")
                .expose(PublishPort::tcp(4317), HostPort::new(4317))
                .expose(PublishPort::tcp(55679), HostPort::new(55679))
                .build();

            let otel_container = docker.containers().create(&otel_container_ops).await?;
            otel_container.start().await?;
        }
    }

    println!("All system components running... System Initiative is alive!");

    Ok(())
}
