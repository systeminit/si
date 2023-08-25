use crate::engine::{ContainerEngine, ContainerReleaseInfo, SiContainerSummary, SiImageSummary};
use crate::{CliResult, SiCliError};
use async_trait::async_trait;
use podman_api::Podman;
use std::path::PathBuf;

pub struct PodmanEngine {
    podman: Podman,
}

impl PodmanEngine {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(_sock: Option<String>) -> CliResult<Box<dyn ContainerEngine>> {
        let podman = Podman::unix("//run/podman.sock");
        Ok(Box::new(PodmanEngine { podman }))
    }
}

#[allow(clippy::diverging_sub_expression)] // TODO(fnichol): remove when `todo!()`s are gone
#[async_trait]
impl ContainerEngine for PodmanEngine {
    fn get_engine_identifier(&self) -> String {
        "podman".to_string()
    }

    async fn ping(&self) -> CliResult<()> {
        let ping_info = self.podman.ping().await?;
        dbg!(&ping_info);
        Ok(())
    }

    async fn missing_containers(&self) -> Result<Vec<String>, SiCliError> {
        todo!()
    }

    async fn download_missing_containers(&self, _missing_containers: Vec<String>) -> CliResult<()> {
        todo!()
    }

    async fn get_container_details(&self) -> CliResult<Vec<ContainerReleaseInfo>> {
        todo!()
    }

    async fn cleanup_image(&self, _name: String) -> CliResult<()> {
        todo!()
    }

    async fn get_container_logs(&self, _name: String, _log_lines: usize) -> CliResult<bool> {
        todo!()
    }

    async fn get_existing_container(&self, _name: String) -> CliResult<Option<SiContainerSummary>> {
        todo!()
    }

    async fn delete_container(&self, _id: String, _name: String) -> CliResult<()> {
        todo!()
    }

    async fn downloaded_systeminit_containers_list(
        &self,
    ) -> Result<Vec<SiImageSummary>, SiCliError> {
        todo!()
    }

    async fn start_container(&self, _id: String) -> CliResult<()> {
        todo!()
    }

    async fn stop_container(&self, _id: String) -> CliResult<()> {
        todo!()
    }

    async fn create_otelcol(&self, _name: String, _image: String) -> CliResult<()> {
        todo!()
    }

    async fn create_jaeger(&self, _name: String, _image: String) -> CliResult<()> {
        todo!()
    }

    async fn create_nats(&self, _name: String, _image: String) -> CliResult<()> {
        todo!()
    }

    async fn create_postgres(&self, _name: String, _image: String) -> CliResult<()> {
        todo!()
    }

    async fn create_council(&self, _name: String, _image: String) -> CliResult<()> {
        todo!()
    }

    async fn create_veritech(
        &self,
        _name: String,
        _image: String,
        _credentials: &mut Vec<String>,
        _data_dir: PathBuf,
        _with_debug_logs: bool,
    ) -> CliResult<()> {
        todo!()
    }

    async fn create_pinga(
        &self,
        _name: String,
        _image: String,
        _data_dir: PathBuf,
    ) -> CliResult<()> {
        todo!()
    }

    async fn create_sdf(&self, _name: String, _image: String, _data_dir: PathBuf) -> CliResult<()> {
        todo!()
    }

    async fn create_web(
        &self,
        _name: String,
        _image: String,
        _host_port: u32,
        _host_ip: String,
    ) -> CliResult<()> {
        todo!()
    }
}
