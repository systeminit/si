use clap::CommandFactory;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;
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
        Some(Commands::UploadSpec(args)) => {
            upload_pkg_spec(
                client.clone(),
                args.target,
                &list_specs(client.clone()).await?,
            )
            .await?
        }
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

    let pb = ProgressBar::new(1);
    setup_progress_bar(&pb);

    download_and_write_spec(client.clone(), module.id, out.clone(), &pb).await?;
    pb.finish_with_message("Download complete");

    Ok(())
}

async fn write_all_specs(client: ModuleIndexClient, out: PathBuf) -> Result<()> {
    let specs = list_specs(client.clone()).await?;

    let pb = ProgressBar::new(specs.len() as u64);
    setup_progress_bar(&pb);

    for module in specs {
        download_and_write_spec(client.clone(), module.id, out.clone(), &pb).await?;
    }

    pb.finish_with_message("✨ All downloads complete!");
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
    pb: &ProgressBar,
) -> Result<()> {
    let pkg = SiPkg::load_from_bytes(
        &client
            .download_module(Ulid::from_string(&module_id)?)
            .await?,
    )?;
    let spec = pkg.to_spec().await?;
    let spec_name = format!("{}.json", spec.name);
    fs::create_dir_all(&out)?;
    pb.set_message(format!("Downloading: {}", spec_name));
    fs::write(
        Path::new(&out).join(spec_name),
        serde_json::to_string_pretty(&spec)?,
    )?;
    pb.inc(1);
    Ok(())
}

async fn upload_pkg_spec(
    client: ModuleIndexClient,
    spec: PathBuf,
    existing_modules: &Vec<LatestModuleResponse>,
) -> Result<()> {
    let pkg = json_to_pkg(spec)?;
    let schema = pkg.schemas()?[0].clone();
    let pkg_schema_id = schema.unique_id().unwrap();

    // reject existing modules with this schema id
    for module in existing_modules {
        if let Some(schema_id) = &module.schema_id {
            if schema_id == pkg_schema_id {
                client
                    .reject_module(
                        Ulid::from_string(&module.id)?,
                        CLOVER_DEFAULT_CREATOR.to_string(),
                    )
                    .await?;
            }
        }
    }

    // upload the module
    let module_id = upload_module(client.clone(), pkg).await?;

    // promote the newly update module as a built-in
    client
        .promote_to_builtin(module_id, CLOVER_DEFAULT_CREATOR.to_string())
        .await?;

    Ok(())
}

async fn upload_all_pkg_specs(client: ModuleIndexClient, target_dir: PathBuf) -> Result<()> {
    let specs: Vec<_> = fs::read_dir(&target_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "json")
        })
        .collect();

    let pb = ProgressBar::new(specs.len() as u64);
    setup_progress_bar(&pb);

    let existing_modules = list_specs(client.clone()).await?;
    for (index, spec) in specs.iter().enumerate() {
        pb.set_message(format!("Uploading: {}", spec.file_name().to_string_lossy()));
        upload_pkg_spec(client.clone(), spec.path(), &existing_modules).await?;

        pb.inc(1);
        // Every 100th file, pause for S3 throttling
        if (index + 1) % 100 == 0 && index + 1 < specs.len() {
            pb.set_message("⏳ Rate limit cooling down...".to_string());
            sleep(Duration::from_secs(5)).await;
        }
    }

    pb.finish_with_message("✨ All uploads complete!");
    Ok(())
}

fn json_to_pkg(spec: PathBuf) -> Result<SiPkg> {
    let buf = fs::read_to_string(&spec)?;
    let spec: PkgSpec = serde_json::from_str(&buf)?;
    Ok(SiPkg::load_from_spec(spec)?)
}

async fn upload_module(client: ModuleIndexClient, pkg: SiPkg) -> Result<Ulid> {
    let schema = pkg.schemas()?[0].clone();
    let metadata = pkg.metadata()?;

    let module = client
        .upload_module(
            metadata.name(),
            metadata.version(),
            Some(metadata.hash().to_string()),
            schema.unique_id().map(String::from),
            pkg.write_to_bytes()?,
            schema.variants()?[0].unique_id().map(String::from),
            Some(metadata.version().to_string()),
        )
        .await?;

    Ok(Ulid::from_string(&module.id)?)
}

fn setup_progress_bar(pb: &ProgressBar) {
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})\n{msg}")
            .expect("could not build progress bar")
            .progress_chars("▸▹▹"),
    );
}
