use std::str::FromStr;

use clap::Parser;
use color_eyre::Result;
use nix::{fcntl::OFlag, sys::stat::Mode, unistd::ForkResult};
use si_filesystem::{mount, WorkspaceId};
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
#[command(name = "si-fs", version = "0.1")]
#[command(about = "Mounts a fuse filesystem that represents a System Initiative workspace")]
struct Args {
    #[arg(long, short = 'w')]
    workspace_id: String,
    #[arg(long, short = 'e')]
    endpoint: String,
    #[arg(long, short = 't', env = "SI_BEARER_TOKEN", hide_env_values(true))]
    token: String,
    #[arg(long, short = 'f')]
    foreground: bool,
    #[arg(value_name = "MOUNTPOINT")]
    mount_point: String,
}

fn redirect_to_dev_null() -> Result<()> {
    let dev_null_fd = nix::fcntl::open("/dev/null", OFlag::O_RDWR, Mode::empty())?;

    nix::unistd::dup2(dev_null_fd, nix::libc::STDOUT_FILENO)?;
    nix::unistd::dup2(dev_null_fd, nix::libc::STDIN_FILENO)?;
    nix::unistd::dup2(dev_null_fd, nix::libc::STDERR_FILENO)?;

    nix::unistd::close(dev_null_fd)?;

    Ok(())
}

fn daemonize() -> Result<()> {
    match unsafe { nix::unistd::fork() }? {
        ForkResult::Parent { .. } => {
            std::process::exit(0);
        }
        ForkResult::Child => {
            nix::unistd::setsid()?;

            match unsafe { nix::unistd::fork() }? {
                ForkResult::Parent { .. } => {
                    std::process::exit(0);
                }
                ForkResult::Child => {
                    redirect_to_dev_null()?;

                    Ok(())
                }
            }
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let workspace_id = WorkspaceId::from_str(&args.workspace_id)?;

    if !args.foreground {
        daemonize()?;
    }

    let rt = Runtime::new()?;

    mount(
        args.token.clone(),
        args.endpoint.clone(),
        workspace_id,
        &args.mount_point,
        rt.handle().clone(),
        None,
    );

    Ok(())
}
