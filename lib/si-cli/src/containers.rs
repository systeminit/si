use crate::SiCliError;
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::models::{ContainerSummary, ImageSummary};
use docker_api::opts::{
    ContainerFilter, ContainerListOpts, ImageListOpts, ImageRemoveOpts, LogsOpts, PullOpts,
    RegistryAuth,
};
use docker_api::Docker;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::cmp::min;
use std::string::ToString;

#[derive(Debug)]
pub struct DockerReleaseInfo {
    pub git_sha: String,
    pub created_at: String,
    pub image: String,
}

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

pub(crate) async fn get_container_details() -> CliResult<Vec<DockerReleaseInfo>> {
    let mut release_info: Vec<DockerReleaseInfo> = Vec::new();
    let containers = downloaded_systeminit_containers_list().await?;
    for container in containers {
        // Each of the containers we use will 100% have these labels so it's fine to unwrap them
        // it's not the ideal and we can find a better way to deal with the option but it works
        release_info.push(DockerReleaseInfo {
            git_sha: container
                .labels
                .get("org.opencontainers.image.revision")
                .unwrap()
                .to_string(),
            created_at: container
                .labels
                .get("org.opencontainers.image.created")
                .unwrap()
                .to_string(),
            image: container.labels.get("name").unwrap().to_string(),
        })
    }

    Ok(release_info)
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

            let auth = RegistryAuth::builder()
                .username("stack72")
                .password("dckr_pat_dHhJ3jhygqHx2gCCZqchywQEvDQ")
                .build();
            let pull_opts = PullOpts::builder()
                .image(missing_container)
                .tag("stable")
                .auth(auth)
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

pub(crate) async fn delete_container(
    docker: &Docker,
    container_summary: ContainerSummary,
    name: String,
) -> CliResult<()> {
    println!(
        "Deleting container: {} ({})",
        name,
        container_summary.id.as_ref().unwrap()
    );
    let container = docker
        .containers()
        .get(container_summary.id.as_ref().unwrap());
    container.delete().await?;
    Ok(())
}

pub(crate) async fn get_existing_container(
    docker: &Docker,
    name: String,
) -> CliResult<Option<ContainerSummary>> {
    let filter = ContainerFilter::Name(name.clone());
    let list_opts = ContainerListOpts::builder()
        .filter([filter])
        .all(true)
        .build();

    let mut containers = docker.containers().list(&list_opts).await?;
    Ok(containers.pop())
}

pub(crate) async fn cleanup_image(docker: &Docker, name: String) -> CliResult<()> {
    let image_name = format!("systeminit/{0}:stable", name);
    let opts = ImageRemoveOpts::builder()
        .force(true)
        .noprune(false)
        .build();

    if (docker.images().get(image_name.clone()).inspect().await).is_ok() {
        println!("Removing image: {0}", image_name.clone());
        docker
            .images()
            .get(image_name.clone())
            .remove(&opts)
            .await?;
    };

    Ok(())
}

pub(crate) async fn get_container_logs(
    docker: &Docker,
    name: String,
    log_lines: usize,
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
            let logs_opts = LogsOpts::builder()
                .n_lines(log_lines)
                .stdout(true)
                .stderr(true)
                .build();
            let container = docker.containers().get(existing_id);
            let logs_stream = container.logs(&logs_opts);
            let logs: Vec<_> = logs_stream
                .map(|chunk| match chunk {
                    Ok(chunk) => chunk.to_vec(),
                    Err(e) => {
                        eprintln!("Error: {e}");
                        vec![]
                    }
                })
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            println!("{}", String::from_utf8_lossy(&logs));
            return Ok(true);
        }
    }

    Ok(false)
}
