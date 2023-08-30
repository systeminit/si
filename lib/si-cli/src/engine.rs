use crate::{CliResult, SiCliError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod docker_engine;
pub mod podman_engine;

#[async_trait]
pub trait ContainerEngine {
    fn get_engine_identifier(&self) -> String;
    async fn ping(&self) -> CliResult<()>;
    async fn missing_containers(&self) -> Result<Vec<String>, SiCliError>;
    async fn download_missing_containers(&self, missing_containers: Vec<String>) -> CliResult<()>;
    async fn get_container_details(&self) -> CliResult<Vec<ContainerReleaseInfo>>;
    async fn cleanup_image(&self, name: String) -> CliResult<()>;
    async fn get_container_logs(&self, name: String, log_lines: usize) -> CliResult<bool>;
    async fn get_existing_container(&self, name: String) -> CliResult<Option<SiContainerSummary>>;
    async fn delete_container(&self, id: String, name: String) -> CliResult<()>;
    async fn downloaded_systeminit_containers_list(
        &self,
    ) -> Result<Vec<SiImageSummary>, SiCliError>;
    async fn create_network(&self) -> CliResult<()>;
    async fn delete_network(&self) -> CliResult<()>;
    async fn start_container(&self, id: String) -> CliResult<()>;
    async fn stop_container(&self, id: String) -> CliResult<()>;
    async fn create_otelcol(&self, name: String, image: String) -> CliResult<()>;
    async fn create_jaeger(&self, name: String, image: String) -> CliResult<()>;
    async fn create_nats(&self, name: String, image: String) -> CliResult<()>;
    async fn create_postgres(&self, name: String, image: String) -> CliResult<()>;
    async fn create_council(&self, name: String, image: String) -> CliResult<()>;
    async fn create_veritech(
        &self,
        name: String,
        image: String,
        credentials: &mut Vec<String>,
        data_dir: PathBuf,
        with_debug_logs: bool,
    ) -> CliResult<()>;
    async fn create_pinga(&self, name: String, image: String, data_dir: PathBuf) -> CliResult<()>;
    async fn create_sdf(
        &self, 
        name: String, 
        image: String,
        host_ip: String,
        host_port: u32,
        data_dir: PathBuf,
    ) -> CliResult<()>;
    async fn create_web(
        &self,
        name: String,
        image: String,
        host_ip: String,
        host_port: u32,
    ) -> CliResult<()>;
}

#[derive(Debug)]
pub struct ContainerReleaseInfo {
    pub git_sha: String,
    pub created_at: String,
    pub image: String,
    pub version: String,
}

pub struct SiContainerSummary {
    pub created: Option<i64>,
    pub id: Option<String>,
    pub image: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    pub status: Option<String>,
    pub state: Option<String>,
}

pub struct SiImageSummary {
    pub containers: isize,
    pub created: isize,
    pub id: String,
    pub labels: HashMap<String, String>,
    pub repo_tags: Vec<String>,
}

impl From<docker_api::models::ImageSummary> for SiImageSummary {
    fn from(image: docker_api::models::ImageSummary) -> SiImageSummary {
        SiImageSummary {
            containers: image.containers,
            created: image.created,
            id: image.id,
            labels: image.labels,
            repo_tags: image.repo_tags,
        }
    }
}

impl From<docker_api::models::ContainerSummary> for SiContainerSummary {
    fn from(container: docker_api::models::ContainerSummary) -> SiContainerSummary {
        SiContainerSummary {
            created: container.created,
            id: container.id,
            image: container.image,
            labels: container.labels,
            status: container.status,
            state: container.state,
        }
    }
}

impl From<podman_api::models::LibpodImageSummary> for SiImageSummary {
    fn from(image: podman_api::models::LibpodImageSummary) -> SiImageSummary {
        let containers = match image.containers {
            Some(count) => count as isize,
            None => 0,
        };

        let created = match image.created {
            Some(seconds) => seconds as isize,
            None => 0,
        };

        let id = match image.id {
            Some(id) => id,
            None => "".to_owned(),
        };

        let labels = match image.labels {
            Some(labels) => labels,
            None => HashMap::new(),
        };

        let repo_tags = match image.repo_tags {
            Some(repo_tags) => repo_tags,
            None => Vec::new(),
        };

        SiImageSummary {
            containers,
            created,
            id,
            labels,
            repo_tags,
        }
    }
}

impl From<podman_api::models::ListContainer> for SiContainerSummary {
    fn from(container: podman_api::models::ListContainer) -> SiContainerSummary {
        SiContainerSummary {
            created: container.created.map(|created| created.timestamp()),
            id: container.id,
            image: container.image,
            labels: container.labels,
            status: container.status,
            state: container.state,
        }
    }
}
