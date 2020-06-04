use std::env;
use std::path::PathBuf;
use tokio::process::Command;

const KUBECTL_COMMAND: &str = "kubectl";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not find command on PATH: {0}")]
    CommandNotFound(&'static str),
    #[error("environment variable error: {0}")]
    EnvVarError(#[from] env::VarError),
}

pub struct KubectlCommand {
    namespace: String,
}

impl KubectlCommand {
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
        }
    }

    pub fn apply(&mut self) -> Result<Command> {
        let mut cmd = self.cmd()?;
        cmd.arg("apply").arg("-f").arg("-");

        Ok(cmd)
    }

    pub fn get_deployments(&mut self) -> Result<Command> {
        let mut cmd = self.cmd()?;
        cmd.arg("get").arg("deployments.apps");

        Ok(cmd)
    }

    pub fn get_deployment_last_applied(&mut self, name: impl AsRef<str>) -> Result<Command> {
        let mut cmd = self.cmd()?;
        cmd.arg("apply")
            .arg("view-last-applied")
            .arg(format!("deployment.apps/{}", name.as_ref()))
            .arg("--output=yaml");

        Ok(cmd)
    }

    fn cmd(&self) -> Result<Command> {
        let mut cmd = Command::new(kubectl_binary()?);
        cmd.arg(format!("--namespace={}", &self.namespace));

        Ok(cmd)
    }
}

fn kubectl_binary() -> Result<PathBuf> {
    env::split_paths(&env::var("PATH")?)
        .map(|path| path.join(KUBECTL_COMMAND))
        .find(|candidate| candidate.is_file())
        .ok_or(Error::CommandNotFound(KUBECTL_COMMAND))
}
