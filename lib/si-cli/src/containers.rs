use crate::SiCliError;
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::models::ImageSummary;
use docker_api::opts::{ContainerFilter, ContainerListOpts, ImageListOpts, PullOpts};
use docker_api::Docker;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::cmp::min;
use std::string::ToString;

pub(crate) async fn downloaded_systeminit_containers_list() -> Result<Vec<ImageSummary>, SiCliError>
{
    let docker = Docker::unix("//var/run/docker.sock");
    let opts = ImageListOpts::builder().all(true).build();
    let mut containers = docker.images().list(&opts).await?;

    let containers: Vec<ImageSummary> = containers
        .drain(..)
        .filter(|c| {
            c.repo_tags
                .iter()
                .any(|t| t.starts_with("systeminit/") && t.ends_with(":stable"))
        })
        .collect();

    Ok(containers)
}

pub(crate) async fn missing_containers() -> Result<Vec<String>, SiCliError> {
    let mut missing_containers = Vec::new();
    let containers = downloaded_systeminit_containers_list().await?;

    for name in CONTAINER_NAMES.iter() {
        let required_container = format!("systeminit/{0}", name);
        if !containers.iter().any(|c| {
            c.repo_tags
                .iter()
                .all(|t| t.contains(required_container.as_str()))
        }) {
            missing_containers.push(required_container.to_string());
        }
    }

    Ok(missing_containers)
}

pub(crate) async fn download_missing_containers(missing_containers: Vec<String>) -> CliResult<()> {
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "{spinner:.red} [{elapsed_precise}] [{wide_bar:.yellow/blue}]",
    )
    .unwrap()
    .progress_chars("#>-");

    let total_size = 100123123;

    println!("Found {0} missing containers", missing_containers.len());

    let mut spawned = Vec::new();
    for missing_container in missing_containers {
        let pb = m.add(ProgressBar::new(total_size));
        pb.set_style(sty.clone());

        let mut message = "Downloading ".to_owned();
        message.push_str(missing_container.as_str());

        let h1 = tokio::spawn(async move {
            let docker = Docker::unix("//var/run/docker.sock");
            let mut downloaded = 0;

            let pull_opts = PullOpts::builder()
                .image(missing_container)
                .tag("stable")
                .build();
            let images = docker.images();
            let mut stream = images.pull(&pull_opts);
            while let Some(pull_result) = stream.next().await {
                match pull_result {
                    Ok(docker_api::models::ImageBuildChunk::PullStatus {
                        progress_detail, ..
                    }) => {
                        if let Some(progress_detail) = progress_detail {
                            let new = min(
                                downloaded + progress_detail.current.unwrap_or(0),
                                total_size,
                            );
                            downloaded = progress_detail.current.unwrap_or(0);
                            pb.set_position(new);
                        }
                    }
                    Ok(_) => {}
                    Err(e) => eprintln!("{e}"),
                }
            }
        });

        m.println(message).unwrap();

        spawned.push(h1);
    }

    for spawn in spawned {
        spawn.await.unwrap();
    }

    m.println("All containers successfully downloaded").unwrap();
    m.clear().unwrap();

    Ok(())
}

pub(crate) async fn has_existing_container(
    docker: &Docker,
    name: String,
    delete_container: bool,
) -> CliResult<bool> {
    let filter = ContainerFilter::Name(name.clone());
    let list_opts = ContainerListOpts::builder()
        .filter([filter])
        .all(true)
        .build();
    let containers = docker.containers().list(&list_opts).await?;
    if !containers.is_empty() {
        let existing_id = containers.first().unwrap().id.as_ref().unwrap();
        let state = containers.first().unwrap().state.as_ref().unwrap();

        if *state == "running" {
            return Ok(true);
        }

        if delete_container {
            println!("Found an existing {} container: {}", name, *existing_id);
            docker.containers().get(existing_id).delete().await?;
            return Ok(false);
        }
    }

    Ok(false)
}
