use crate::engine::{ContainerEngine, ContainerReleaseInfo, SiContainerSummary, SiImageSummary};
use crate::{CliResult, SiCliError, CONTAINER_NAMES};
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use directories::UserDirs;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use podman_api::models::{ContainerMount, Namespace, PerNetworkOptions, PortMapping};
use podman_api::opts::{
    ContainerCreateOpts, ContainerDeleteOpts, ContainerListFilter, ContainerListOpts,
    ContainerLogsOpts, ContainerStopOpts, ImageListOpts, NetworkCreateOpts, PullOpts,
};
use podman_api::Podman;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

pub struct PodmanEngine {
    podman: Podman,
    network: String,
}

impl PodmanEngine {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(sock: Option<String>) -> CliResult<Box<dyn ContainerEngine>> {
        let podman_sock = if let Some(sock) = sock {
            sock
        } else {
            "".to_string()
        };

        let mut podman_socket_candidates = Vec::new();

        if cfg!(target_os = "macos") {
            if let Some(user_dirs) = UserDirs::new() {
                podman_socket_candidates.push(
                    user_dirs
                        .home_dir()
                        .join(".local")
                        .join("share")
                        .join("containers")
                        .join("podman")
                        .join("machine")
                        .join("qemu")
                        .join("podman.sock"),
                );
            }
        } else if cfg!(target_os = "linux") {
            podman_socket_candidates.push(
                #[allow(clippy::disallowed_methods)]
                std::path::Path::new(&std::env::var("XDG_RUNTIME_DIR")?)
                    .join("podman")
                    .join("podman.sock"),
            );
        } else {
            // This should NEVER get called but I added it in here just incase
            return Err(SiCliError::UnsupportedOperatingSystem(
                env::consts::OS.to_string(),
            ));
        };

        podman_socket_candidates.push(std::path::Path::new("/var/run/podman.sock").to_path_buf());

        let podman = if podman_sock.is_empty() {
            let socket = podman_socket_candidates
                .iter()
                .find(|candidate| candidate.exists())
                .ok_or(eyre!(
                    "failed to determine podman socket location. Set a custom location using \
                    `--podman-sock` or `SI_PODMAN_SOCK`; candidates={podman_socket_candidates:?}"
                ))?;

            Podman::unix(socket)
        } else {
            println!("Checking for user supplied podman.sock");
            let path = std::path::Path::new(podman_sock.as_str()).to_path_buf();

            Podman::unix(path)
        };

        Ok(Box::new(PodmanEngine {
            podman,
            network: "si".to_owned(),
        }))
    }
}

#[allow(clippy::diverging_sub_expression)] // TODO(fnichol): remove when `todo!()`s are gone
#[async_trait]
impl ContainerEngine for PodmanEngine {
    fn get_engine_identifier(&self) -> String {
        "podman".to_string()
    }

    async fn ping(&self) -> CliResult<()> {
        self.podman.ping().await?;
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
                    .all(|t| t.contains(&required_container.to_string()))
            }) {
                missing_containers.push(required_container.to_string());
            }
        }

        Ok(missing_containers)
    }

    async fn download_missing_containers(&self, missing_containers: Vec<String>) -> CliResult<()> {
        let m = MultiProgress::new();
        let sty = ProgressStyle::with_template(
            "{spinner:.red} [{elapsed_precise}] [{wide_msg:.yellow/blue}]",
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

            let podman = self.podman.clone();

            let h1 = tokio::spawn(async move {
                let pull_opts = PullOpts::builder()
                    // TODO: Can the docker.io/ prefix be omitted?
                    .reference(format!("docker.io/{}:stable", missing_container))
                    .build();
                let images = podman.images();
                let mut stream = images.pull(&pull_opts);
                while let Some(pull_report) = stream.next().await {
                    match pull_report {
                        Ok(pull_report) => {
                            if let Some(stream) = pull_report.stream {
                                pb.set_message(stream.trim().to_owned());
                            }
                        }
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

        if (self.podman.images().get(image_name.clone()).inspect().await).is_ok() {
            println!("Removing image: {0}", image_name.clone());
            self.podman
                .images()
                .get(image_name.clone())
                .remove()
                .await?;
        };

        Ok(())
    }

    async fn get_container_logs(&self, name: String, log_lines: usize) -> CliResult<bool> {
        let list_opts = ContainerListOpts::builder()
            .all(true)
            .filter([ContainerListFilter::Name(name.clone())])
            .build();
        let containers = self.podman.containers().list(&list_opts).await?;
        if !containers.is_empty() {
            let existing_id = containers.first().unwrap().id.as_ref().unwrap();
            let state = containers.first().unwrap().state.as_ref().unwrap();

            if *state == "running" {
                let logs_opts = ContainerLogsOpts::builder()
                    .tail(log_lines.to_string())
                    .stdout(true)
                    .stderr(true)
                    .build();
                let container = self.podman.containers().get(existing_id);
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
        let list_opts = ContainerListOpts::builder()
            .all(true)
            .filter([ContainerListFilter::Name(name.clone())])
            .build();

        let mut containers: Vec<SiContainerSummary> = self
            .podman
            .containers()
            .list(&list_opts)
            .await?
            .into_iter()
            .map(SiContainerSummary::from)
            .collect();

        Ok(containers.pop())
    }

    async fn delete_container(&self, id: String, name: String) -> CliResult<()> {
        println!("Deleting container: {} ({})", name, id);
        let container = self.podman.containers().get(id);
        container
            .delete(&ContainerDeleteOpts::builder().force(true).build())
            .await?;
        Ok(())
    }

    async fn downloaded_systeminit_containers_list(
        &self,
    ) -> Result<Vec<SiImageSummary>, SiCliError> {
        let opts = ImageListOpts::builder().all(true).build();
        let mut containers = self.podman.images().list(&opts).await?;

        let containers: Vec<SiImageSummary> = containers
            .drain(..)
            .filter(|c| {
                c.repo_tags.iter().any(|t| {
                    t.starts_with(&["systeminit/".to_owned()])
                        && t.ends_with(&[":stable".to_owned()])
                })
            })
            .map(SiImageSummary::from)
            .collect();

        Ok(containers)
    }

    async fn start_container(&self, id: String) -> CliResult<()> {
        self.podman.containers().get(id).start(None).await?;
        Ok(())
    }

    async fn stop_container(&self, id: String) -> CliResult<()> {
        self.podman
            .containers()
            .get(id)
            .stop(&ContainerStopOpts::builder().build())
            .await?;
        Ok(())
    }

    async fn create_network(&self) -> CliResult<()> {
        match self
            .podman
            .networks()
            .get(self.network.clone())
            .exists()
            .await
        {
            Ok(exists) => {
                if exists {
                    return Ok(());
                }
            }
            Err(e) => return Err(SiCliError::Podman(e)),
        }

        let opts = NetworkCreateOpts::builder()
            .name(self.network.clone())
            .dns_enabled(true)
            .build();
        let network = self.podman.networks().create(&opts).await?;
        println!("Created network: {0}", network.name.unwrap());
        Ok(())
    }

    async fn delete_network(&self) -> CliResult<()> {
        match self
            .podman
            .networks()
            .get(self.network.clone())
            .exists()
            .await
        {
            Ok(exists) => {
                if !exists {
                    return Ok(());
                }
            }
            Err(e) => return Err(SiCliError::Podman(e)),
        }

        println!("Removing network: {0}", self.network.clone());
        self.podman
            .networks()
            .get(self.network.clone())
            .delete()
            .await?;

        Ok(())
    }

    async fn create_otelcol(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name.clone())
            .image(format!("{0}:stable", image.clone()))
            .net_namespace(Namespace {
                nsmode: Some("bridge".to_owned()),
                value: None,
            })
            .networks(HashMap::from([(
                self.network.to_owned(),
                PerNetworkOptions {
                    aliases: Some(vec!["otelcol".to_owned()]),
                    interface_name: None,
                    static_ips: None,
                    static_mac: None,
                },
            )]))
            .build();

        let container = self.podman.containers().create(&create_opts).await?;
        self.podman
            .containers()
            .get(container.id)
            .start(None)
            .await?;
        Ok(())
    }

    async fn create_jaeger(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name.clone())
            .image(format!("{0}:stable", image.clone()))
            .net_namespace(Namespace {
                nsmode: Some("bridge".to_owned()),
                value: None,
            })
            .networks(HashMap::from([(
                self.network.to_owned(),
                PerNetworkOptions {
                    aliases: Some(vec!["jaeger".to_owned()]),
                    interface_name: None,
                    static_ips: None,
                    static_mac: None,
                },
            )]))
            .portmappings(vec![PortMapping {
                container_port: Some(16686),
                host_port: Some(16686),
                host_ip: None,
                protocol: None,
                range: None,
            }])
            .build();

        let container = self.podman.containers().create(&create_opts).await?;
        self.podman
            .containers()
            .get(container.id)
            .start(None)
            .await?;
        Ok(())
    }

    async fn create_nats(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name.clone())
            .image(format!("{0}:stable", image.clone()))
            .net_namespace(Namespace {
                nsmode: Some("bridge".to_owned()),
                value: None,
            })
            .networks(HashMap::from([(
                self.network.to_owned(),
                PerNetworkOptions {
                    aliases: Some(vec!["nats".to_owned()]),
                    interface_name: None,
                    static_ips: None,
                    static_mac: None,
                },
            )]))
            .command(vec!["--config", "nats-server.conf", "-DVV"])
            .build();

        let container = self.podman.containers().create(&create_opts).await?;
        self.podman
            .containers()
            .get(container.id)
            .start(None)
            .await?;
        Ok(())
    }

    async fn create_postgres(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name.clone())
            .image(format!("{0}:stable", image.clone()))
            .net_namespace(Namespace {
                nsmode: Some("bridge".to_owned()),
                value: None,
            })
            .networks(HashMap::from([(
                self.network.to_owned(),
                PerNetworkOptions {
                    aliases: Some(vec!["postgres".to_owned()]),
                    interface_name: None,
                    static_ips: None,
                    static_mac: None,
                },
            )]))
            .env(HashMap::from([
                ("POSTGRES_PASSWORD", "bugbear"),
                ("PGPASSWORD", "bugbear"),
                ("POSTGRES_USER", "si"),
                ("POSTGRES_DB", "si"),
            ]))
            .build();

        let container = self.podman.containers().create(&create_opts).await?;
        self.podman
            .containers()
            .get(container.id)
            .start(None)
            .await?;
        Ok(())
    }

    async fn create_council(&self, name: String, image: String) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name.clone())
            .image(format!("{0}:stable", image.clone()))
            .net_namespace(Namespace {
                nsmode: Some("bridge".to_owned()),
                value: None,
            })
            .networks(HashMap::from([(
                self.network.to_owned(),
                PerNetworkOptions {
                    aliases: Some(vec!["council".to_owned()]),
                    interface_name: None,
                    static_ips: None,
                    static_mac: None,
                },
            )]))
            .env(HashMap::from([
                ("SI_COUNCIL__NATS__URL", "nats"),
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://otelcol:4317"),
            ]))
            .build();

        let container = self.podman.containers().create(&create_opts).await?;
        self.podman
            .containers()
            .get(container.id)
            .start(None)
            .await?;
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
        let mut env_vars = HashMap::from([
            ("SI_VERITECH__NATS__URL", "nats"),
            ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://otelcol:4317"),
        ]);

        if with_debug_logs {
            env_vars.insert("SI_LOG", "debug");
        }

        for env_val in credentials.iter() {
            let fields: Vec<&str> = env_val.split('=').collect();
            env_vars.insert(fields[0], fields[1]);
        }

        let create_opts = ContainerCreateOpts::builder()
            .name(name.clone())
            .image(format!("{0}:stable", image.clone()))
            .net_namespace(Namespace {
                nsmode: Some("bridge".to_owned()),
                value: None,
            })
            .networks(HashMap::from([(
                self.network.to_owned(),
                PerNetworkOptions {
                    aliases: Some(vec!["veritech".to_owned()]),
                    interface_name: None,
                    static_ips: None,
                    static_mac: None,
                },
            )]))
            .env(env_vars)
            .mounts(vec![ContainerMount {
                destination: Some("/run/cyclone".to_owned()),
                source: Some(data_dir.display().to_string()),
                options: Some(get_container_mount_opts()),
                _type: Some("bind".to_owned()),
                uid_mappings: None,
                gid_mappings: None,
            }])
            .build();

        let container = self.podman.containers().create(&create_opts).await?;
        self.podman
            .containers()
            .get(container.id)
            .start(None)
            .await?;
        Ok(())
    }

    async fn create_pinga(&self, name: String, image: String, data_dir: PathBuf) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name.clone())
            .image(format!("{0}:stable", image.clone()))
            .net_namespace(Namespace {
                nsmode: Some("bridge".to_owned()),
                value: None,
            })
            .networks(HashMap::from([(
                self.network.to_owned(),
                PerNetworkOptions {
                    aliases: Some(vec!["pinga".to_owned()]),
                    interface_name: None,
                    static_ips: None,
                    static_mac: None,
                },
            )]))
            .env(HashMap::from([
                (
                    "SI_PINGA__CRYPTO__ENCRYPTION_KEY_FILE",
                    "/run/pinga/cyclone_encryption.key",
                ),
                ("SI_PINGA__NATS__URL", "nats"),
                (
                    "SI_PINGA__PG__CERTIFICATE_PATH",
                    "/run/pinga/dev.postgres.root.crt",
                ),
                ("SI_PINGA__PG__HOSTNAME", "postgres"),
                (
                    "SI_PINGA__SYMMETRIC_CRYPTO_SERVICE__ACTIVE_KEY",
                    "/run/pinga/donkey.key",
                ),
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://otelcol:4317"),
            ]))
            .mounts(vec![ContainerMount {
                destination: Some("/run/pinga".to_owned()),
                source: Some(data_dir.display().to_string()),
                options: Some(get_container_mount_opts()),
                _type: Some("bind".to_owned()),
                uid_mappings: None,
                gid_mappings: None,
            }])
            .build();

        let container = self.podman.containers().create(&create_opts).await?;
        self.podman
            .containers()
            .get(container.id)
            .start(None)
            .await?;
        Ok(())
    }

    async fn create_sdf(
        &self,
        name: String,
        image: String,
        host_ip: String,
        host_port: u32,
        data_dir: PathBuf,
    ) -> CliResult<()> {
        let create_opts = ContainerCreateOpts::builder()
            .name(name.clone())
            .image(format!("{0}:stable", image.clone()))
            .net_namespace(Namespace {
                nsmode: Some("bridge".to_owned()),
                value: None,
            })
            .networks(HashMap::from([(
                self.network.to_owned(),
                PerNetworkOptions {
                    aliases: Some(vec!["sdf".to_owned()]),
                    interface_name: None,
                    static_ips: None,
                    static_mac: None,
                },
            )]))
            .env(HashMap::from([
                (
                    "SI_SDF__CRYPTO__ENCRYPTION_KEY_FILE",
                    "/run/sdf/cyclone_encryption.key",
                ),
                (
                    "SI_SDF__JWT_SIGNING_PUBLIC_KEY__KEY_FILE",
                    "/sdf/jwt_signing_public_key.pem",
                ),
                ("SI_SDF__NATS__URL", "nats"),
                (
                    "SI_SDF__PG__CERTIFICATE_PATH",
                    "/run/sdf/dev.postgres.root.crt",
                ),
                ("SI_SDF__PG__HOSTNAME", "postgres"),
                (
                    "SI_SDF__SYMMETRIC_CRYPTO_SERVICE__ACTIVE_KEY",
                    "/run/sdf/donkey.key",
                ),
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://otelcol:4317"),
            ]))
            .portmappings(vec![PortMapping {
                container_port: Some(5156),
                host_port: Some(host_port.try_into().unwrap()),
                host_ip: Some(host_ip),
                protocol: None,
                range: None,
            }])
            .mounts(vec![
                ContainerMount {
                    destination: Some("/run/sdf/cyclone_encryption.key".to_owned()),
                    source: Some(
                        data_dir
                            .join("cyclone_encryption.key")
                            .display()
                            .to_string(),
                    ),
                    options: Some(get_container_mount_opts()),
                    _type: Some("bind".to_owned()),
                    uid_mappings: None,
                    gid_mappings: None,
                },
                ContainerMount {
                    destination: Some("/run/sdf/donkey.key".to_owned()),
                    source: Some(data_dir.join("donkey.key").display().to_string()),
                    options: Some(get_container_mount_opts()),
                    _type: Some("bind".to_owned()),
                    uid_mappings: None,
                    gid_mappings: None,
                },
                ContainerMount {
                    destination: Some("/run/sdf/dev.postgres.root.crt".to_owned()),
                    source: Some(data_dir.join("dev.postgres.root.crt").display().to_string()),
                    options: Some(get_container_mount_opts()),
                    _type: Some("bind".to_owned()),
                    uid_mappings: None,
                    gid_mappings: None,
                },
                ContainerMount {
                    destination: Some("/run/sdf/jwt_signing_public_key.pem".to_owned()),
                    source: Some(
                        data_dir
                            .join("jwt_signing_public_key.pem")
                            .display()
                            .to_string(),
                    ),
                    options: Some(get_container_mount_opts()),
                    _type: Some("bind".to_owned()),
                    uid_mappings: None,
                    gid_mappings: None,
                },
            ])
            .build();

        let container = self.podman.containers().create(&create_opts).await?;
        self.podman
            .containers()
            .get(container.id)
            .start(None)
            .await?;
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
            .name(name.clone())
            .image(format!("{0}:stable", image.clone()))
            .net_namespace(Namespace {
                nsmode: Some("bridge".to_owned()),
                value: None,
            })
            .networks(HashMap::from([(
                self.network.to_owned(),
                PerNetworkOptions {
                    aliases: Some(vec!["web".to_owned()]),
                    interface_name: None,
                    static_ips: None,
                    static_mac: None,
                },
            )]))
            .env(HashMap::from([("SI_LOG", "trace")]))
            .portmappings(vec![PortMapping {
                container_port: Some(8080),
                host_port: Some(host_port.try_into().unwrap()),
                host_ip: Some(host_ip),
                protocol: None,
                range: None,
            }])
            .build();

        let container = self.podman.containers().create(&create_opts).await?;
        self.podman
            .containers()
            .get(container.id)
            .start(None)
            .await?;
        Ok(())
    }
}

fn get_container_mount_opts() -> Vec<String> {
    let mut mount_options = vec!["ro".to_string()];
    if cfg!(target_os = "linux") {
        mount_options.push("z".to_string())
    }

    mount_options
}
