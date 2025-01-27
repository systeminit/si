use clap::CommandFactory;
use std::path::{Path, PathBuf};
use tokio::fs;
use ulid::Ulid;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use module_index_client::{LatestModuleResponse, ModuleIndexClient};
use si_pkg::{PkgSpec, SiPkg};
use url::Url;

const CLOVER_DEFAULT_CREATOR: &str = "Clover";

#[derive(Parser, Debug)]
#[command(name = "hoist", version = "0.1")]
#[command(about = "Gets and puts cloud control assets from the module index")]
struct Args {
    #[arg(long, short = 'e')]
    endpoint: String,
    #[arg(long, short = 't', env = "SI_BEARER_TOKEN", hide_env_values(true))]
    token: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
#[remain::sorted]
enum Commands {
    UploadAllSpecs(UploadAllSpecsArgs),
    UploadSpec(UploadSpecArgs),
    WriteAllSpecs(WriteAllSpecsArgs),
    WriteSpec(WriteSpecArgs),
}

#[derive(clap::Args, Debug)]
#[command(about = "Upload all specs in {target_dir} to the module index")]
struct UploadAllSpecsArgs {
    #[arg(long, short = 't', required = true)]
    target_dir: PathBuf,
}
#[derive(clap::Args, Debug)]
#[command(about = "Upload the spec {target} to the module index")]
struct UploadSpecArgs {
    #[arg(long, short = 't', required = true)]
    target: PathBuf,
}

#[derive(clap::Args, Debug)]
#[command(about = "Get {spec_name} from the module index and write it to {out}")]
struct WriteSpecArgs {
    #[arg(long, short = 's', required = true)]
    spec_name: String,
    #[arg(long, short = 'o', required = true)]
    out: PathBuf,
}

#[derive(clap::Args, Debug)]
#[command(about = "Get all specs from the module index and write them to {out}")]
struct WriteAllSpecsArgs {
    #[arg(long, short = 'o', required = true)]
    out: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let endpoint = &args.endpoint;
    let token = &args.token;
    let client = ModuleIndexClient::new(Url::parse(endpoint)?, token);

    match args.command {
        Some(Commands::UploadAllSpecs(args)) => {
            upload_all_pkg_specs(client, args.target_dir).await?
        }
        Some(Commands::UploadSpec(args)) => upload_pkg_spec(client, args.target).await?,
        Some(Commands::WriteAllSpecs(args)) => {
            write_all_specs(client, args.out.to_path_buf()).await?
        }
        Some(Commands::WriteSpec(args)) => {
            write_spec(client, args.spec_name, args.out.to_path_buf()).await?
        }
        None => {
            if let Err(err) = Args::command().print_help() {
                eprintln!("Error displaying help: {}", err);
                std::process::exit(1);
            }
            std::process::exit(0);
        }
    }

    Ok(())
}

async fn write_spec(client: ModuleIndexClient, spec_name: String, out: PathBuf) -> Result<()> {
    let module = list_specs(client.clone())
        .await?
        .iter()
        .find(|m| m.name == spec_name)
        .cloned()
        .unwrap_or_else(|| panic!("Unable to find spec with name: {}", spec_name));

    download_and_write_spec(client.clone(), module.id, out.clone()).await?;

    Ok(())
}

async fn write_all_specs(client: ModuleIndexClient, out: PathBuf) -> Result<()> {
    for module in list_specs(client.clone()).await? {
        download_and_write_spec(client.clone(), module.id, out.clone()).await?;
    }

    Ok(())
}

async fn list_specs(client: ModuleIndexClient) -> Result<Vec<LatestModuleResponse>> {
    Ok(client
        .list_latest_modules()
        .await?
        .modules
        .into_iter()
        .filter(|m| {
            m.owner_display_name
                .as_ref()
                .is_some_and(|n| n == CLOVER_DEFAULT_CREATOR)
        })
        .collect::<Vec<LatestModuleResponse>>())
}

async fn download_and_write_spec(
    client: ModuleIndexClient,
    module_id: String,
    out: PathBuf,
) -> Result<()> {
    let pkg = SiPkg::load_from_bytes(
        &client
            .download_module(Ulid::from_string(&module_id)?)
            .await?,
    )?;
    let spec = pkg.to_spec().await?;
    let spec_name = format!("{}.json", spec.name);
    fs::create_dir_all(&out).await?;
    println!("Writing {spec_name} to disk");
    fs::write(
        Path::new(&out).join(spec_name),
        serde_json::to_string_pretty(&spec)?,
    )
    .await?;
    Ok(())
}

async fn upload_pkg_spec(client: ModuleIndexClient, spec: PathBuf) -> Result<()> {
    read_json_and_upload(client, spec).await?;

    Ok(())
}

async fn upload_all_pkg_specs(client: ModuleIndexClient, target_dir: PathBuf) -> Result<()> {
    let mut specs = fs::read_dir(&target_dir).await?;
    while let Some(spec) = specs.next_entry().await? {
        let path = spec.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            read_json_and_upload(client.clone(), path).await?;
        }
    }

    Ok(())
}

async fn read_json_and_upload(client: ModuleIndexClient, spec: PathBuf) -> Result<()> {
    let buf = fs::read_to_string(&spec).await?;
    let spec: PkgSpec = serde_json::from_str(&buf)?;
    let pkg = SiPkg::load_from_spec(spec)?;
    let metadata = pkg.metadata()?;

    println!("Uploading {}", metadata.name());
    client
        .upload_module(
            metadata.name(),
            metadata.version(),
            Some(metadata.hash().to_string()),
            None,
            pkg.write_to_bytes()?,
            None,
            Some(metadata.version().to_string()),
        )
        .await?;

    Ok(())
}
