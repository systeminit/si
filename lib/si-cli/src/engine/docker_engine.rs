use crate::engine::{ContainerEngine, ContainerReleaseInfo, SiContainerSummary, SiImageSummary};
use crate::{CliResult, SiCliError, CONTAINER_NAMES};
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use docker_api::opts::{
    ContainerCreateOpts, ContainerFilter, ContainerListOpts, ContainerStopOpts, HostPort,
    ImageListOpts, ImageRemoveOpts, LogsOpts, PublishPort, PullOpts,
};
use docker_api::Docker;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::cmp::min;
use std::path::PathBuf;

pub struct DockerEngine {
    docker: Docker,
}

impl DockerEngine {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(sock: Option<String>) -> CliResult<Box<dyn ContainerEngine>> {
        let docker_sock = if let Some(sock) = sock {
            sock
        } else {
            "".to_string()
        };

        let docker_socket_candidates = vec![
            #[allow(clippy::disallowed_methods)]
            // Used to determine a path relative to users's home
            std::path::Path::new(&std::env::var("HOME")?)
                .join(".docker")
                .join("run")
                .join("docker.sock"),
            std::path::Path::new("/var/run/docker.sock").to_path_buf(),
        ];

        let docker: Docker;
        if let "" = docker_sock.as_str() {
            let socket = docker_socket_candidates
                .iter()
                .find(|candidate| candidate.exists())
                .ok_or(eyre!(
            "failed to determine Docker socket location. Set a custom location using `--docker-sock` \
            or `SI_DOCKER_SOCK`; candidates={docker_socket_candidates:?}"
        ))?;
            docker = Docker::unix(socket)
        } else {
            println!("Checking for user supplied docker.sock");
            let path = std::path::Path::new(docker_sock.as_str()).to_path_buf();
            docker = Docker::unix(path);
        }

        Ok(Box::new(DockerEngine { docker }))
    }
}

#[async_trait]
impl ContainerEngine for DockerEngine {
    fn get_engine_identifier(&self) -> String {
        "docker".to_string()
    }

    async fn ping(&self) -> CliResult<()> {
        self.docker.ping().await?;
        Ok(())
    }

    async fn missing_containers(&self) -> Result<Vec<String>, SiCliError> {
        let mut missing_containers = Vec::new();
        let containers = self.downloaded_systeminit_containers_list().await?;

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

    async fn download_missing_containers(&self, missing_containers: Vec<String>) -> CliResult<()> {
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

            let docker = self.docker.clone();

            let h1 = tokio::spawn(async move {
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
                            progress_detail,
                            ..
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

    async fn get_container_details(&self) -> CliResult<Vec<ContainerReleaseInfo>> {
        let mut release_info: Vec<ContainerReleaseInfo> = Vec::new();
        let containers = self.downloaded_systeminit_containers_list().await?;
        for container in containers {
            // Each of the containers we use will 100% have these labels so it's fine to unwrap them
            // it's not the ideal and we can find a better way to deal with the option but it works
            release_info.push(ContainerReleaseInfo {
                git_sha: container
                    .labels
                    .get("org.opencontainers.image.revision")
                    .unwrap()
                    .to_string(),
                version: container
                    .labels
                    .get("org.opencontainers.image.version")
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

    async fn cleanup_image(&self, name: String) -> CliResult<()> {
        let image_name = format!("systeminit/{0}:stable", name);
        let opts = ImageRemoveOpts::builder()
            .force(true)
            .noprune(false)
            .build();

        if (self.docker.images().get(image_name.clone()).inspect().await).is_ok() {
            println!("Removing image: {0}", image_name.clone());
            self.docker
                .images()
                .get(image_name.clone())
                .remove(&opts)
                .await?;
        };

        Ok(())
    }

    async fn get_container_logs(&self, name: String, log_lines: usize) -> CliResult<bool> {
        let filter = ContainerFilter::Name(name.clone());
        let list_opts = ContainerListOpts::builder()
            .filter([filter])
            .all(true)
            .build();
        let containers = self.docker.containers().list(&list_opts).await?;
        if !containers.is_empty() {
            let existing_id = containers.first().unwrap().id.as_ref().unwrap();
            let state = containers.first().unwrap().state.as_ref().unwrap();

            if *state == "running" {
                let logs_opts = LogsOpts::builder()
                    .n_lines(log_lines)
                    .stdout(true)
                    .stderr(true)
                    .build();
                let container = self.docker.containers().get(existing_id);
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

    async fn get_existing_container(&self, name: String) -> CliResult<Option<SiContainerSummary>> {
        let filter = ContainerFilter::Name(name.clone());
        let list_opts = ContainerListOpts::builder()
            .filter([filter])
            .all(true)
            .build();

        let mut containers: Vec<SiContainerSummary> = self
            .docker
            .containers()
            .list(&list_opts)
            .await?
            .into_iter()
            .map(SiContainerSummary::from)
            .collect();
        Ok(containers.pop())
    }

    async fn delete_container(&self, id: String, name: String) -> CliResult<()> {
        println!("Deleting container: {} ({})", name, id.clone());
        let container = self.docker.containers().get(id);
        container.delete().await?;
        Ok(())
    }

    async fn downloaded_systeminit_containers_list(
        &self,
    ) -> Result<Vec<SiImageSummary>, SiCliError> {
        let opts = ImageListOpts::builder().all(true).build();
        let mut containers = self.docker.images().list(&opts).await?;

        let containers: Vec<SiImageSummary> = containers
            .drain(..)
            .filter(|c| {
                c.repo_tags
                    .iter()
                    .any(|t| t.starts_with("systeminit/") && t.ends_with(":stable"))
            })
            .map(SiImageSummary::from)
            .collect();

        Ok(containers)
    }

    async fn start_container(&self, id: String) -> CliResult<()> {
        self.docker.containers().get(id).start().await?;
        Ok(())
    }

    async fn stop_container(&self, id: String) -> CliResult<()> {
        let container = self.docker.containers().get(id);
        let inspectations = container.inspect().await?;

        if let Some(state) = inspectations.state {
            if let Some(true) = state.running {
                println!("Stopping container {}", inspectations.name.unwrap());
                container
                    .stop(&ContainerStopOpts::builder().build())
                    .await?;
            }
        }
        Ok(())
    }

    async fn create_network(&self) -> CliResult<()> {
        Ok(())
    }

    async fn delete_network(&self) -> CliResult<()> {
        Ok(())
    }

    async fn create_otelcol(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name.clone())
            .image(format!("{0}:stable", image))
            .links(["local-jaeger-1:jaeger"])
            .restart_policy("on-failure", 3)
            .build();

        let container = self.docker.containers().create(&create_opts).await?;
        container.start().await?;
        Ok(())
    }

    async fn create_jaeger(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name)
            .image(format!("{0}:stable", image))
            .expose(PublishPort::tcp(16686), HostPort::new(16686))
            .restart_policy("on-failure", 3)
            .build();

        let container = self.docker.containers().create(&create_opts).await?;
        container.start().await?;
        Ok(())
    }

    async fn create_nats(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name)
            .image(format!("{0}:stable", image))
            .command(vec!["--config", "nats-server.conf", "-DVV"])
            .restart_policy("on-failure", 3)
            .build();

        let container = self.docker.containers().create(&create_opts).await?;
        container.start().await?;
        Ok(())
    }

    async fn create_postgres(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name)
            .image(format!("{0}:stable", image))
            .env(vec![
                "POSTGRES_PASSWORD=bugbear",
                "PGPASSWORD=bugbear",
                "POSTGRES_USER=si",
                "POSTGRES_DB=si",
            ])
            .restart_policy("on-failure", 3)
            .build();

        let container = self.docker.containers().create(&create_opts).await?;
        container.start().await?;
        Ok(())
    }

    async fn create_council(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name)
            .image(format!("{0}:stable", image))
            .links(vec!["local-nats-1:nats", "local-otelcol-1:otelcol"])
            .env(vec![
                "SI_COUNCIL__NATS__URL=nats",
                "OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol:4317",
            ])
            .restart_policy("on-failure", 3)
            .build();

        let container = self.docker.containers().create(&create_opts).await?;
        container.start().await?;
        Ok(())
    }

    async fn create_veritech(
        &self,
        name: String,
        image: String,
        credentials: &mut Vec<String>,
        data_dir: PathBuf,
        with_debug_logs: bool,
    ) -> CliResult<()> {
        let mut env_vars = vec![
            "SI_VERITECH__NATS__URL=nats".to_string(),
            "OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol:4317".to_string(),
        ];
        if with_debug_logs {
            env_vars.push("SI_LOG=debug".to_string());
        }
        env_vars.append(credentials);
        let create_opts = ContainerCreateOpts::builder()
            .name(name)
            .image(format!("{0}:stable", image))
            .links(vec!["local-nats-1:nats", "local-otelcol-1:otelcol"])
            .env(env_vars)
            .volumes([format!("{}:/run/cyclone:z", data_dir.display())])
            .restart_policy("on-failure", 3)
            .build();

        let container = self.docker.containers().create(&create_opts).await?;
        container.start().await?;
        Ok(())
    }

    async fn create_pinga(&self, name: String, image: String, data_dir: PathBuf) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name)
            .image(format!("{0}:stable", image))
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
            .restart_policy("on-failure", 3)
            .volumes([format!("{}:/run/pinga:z", data_dir.display())])
            .build();

        let container = self.docker.containers().create(&create_opts).await?;
        container.start().await?;
        Ok(())
    }

    async fn create_sdf(
        &self, 
        name: String, 
        image: String, 
        host_ip: String,
        host_port: u32,
        data_dir: PathBuf
    ) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name)
            .image(format!("{0}:stable", image))
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
            .restart_policy("on-failure", 3)
            .expose(
                PublishPort::tcp(5156), 
                HostPort::with_ip(host_port, host_ip),
            )
            .volumes([
                format!(
                    "{}:/run/sdf/cyclone_encryption.key:z",
                    data_dir.join("cyclone_encryption.key").display()
                ),
                format!(
                    "{}:/run/sdf/jwt_signing_public_key.pem:z",
                    data_dir.join("jwt_signing_public_key.pem").display()
                ),
            ])
            .build();

        let container = self.docker.containers().create(&create_opts).await?;
        container.start().await?;
        Ok(())
    }

    async fn create_web(
        &self,
        name: String,
        image: String,
        host_ip: String,
        host_port: u32,
    ) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name)
            .image(format!("{0}:stable", image))
            .links(vec!["local-sdf-1:sdf"])
            .env(["SI_LOG=trace"])
            .network_mode("bridge")
            .restart_policy("on-failure", 3)
            .expose(
                PublishPort::tcp(8080),
                HostPort::with_ip(host_port, host_ip),
            )
            .build();

        let container = self.docker.containers().create(&create_opts).await?;
        container.start().await?;
        Ok(())
    }
}
