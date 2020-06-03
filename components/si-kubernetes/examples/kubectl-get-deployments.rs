use si_kubernetes::kubectl::KubectlCommand;
use std::{env, io, process};

const NAMESPACE_VAR: &str = "NAMESPACE";
const NAMESPACE_DEFAULT: &str = "si";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    process::exit(
        KubectlCommand::new(namespace())
            .get_deployments()?
            .status()
            .await?
            .code()
            .ok_or(io_other_err("terminated by signal"))?,
    )
}

fn namespace() -> String {
    env::var(NAMESPACE_VAR).unwrap_or_else(|_| NAMESPACE_DEFAULT.to_string())
}

fn io_other_err(cause: impl AsRef<str>) -> io::Error {
    io::Error::new(io::ErrorKind::Other, cause.as_ref())
}
