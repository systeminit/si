use clap::CommandFactory;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ulid::Ulid;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use module_index_client::{ModuleDetailsResponse, ModuleIndexClient};
use si_pkg::{PkgSpec, SiPkg};
use tokio::task::JoinSet;
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
    WriteExistingModulesSpec(WriteExistingModulesSpecArgs),
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
#[command(about = "Get all built-ins an write out a hashmap with their name and schema id")]
struct WriteExistingModulesSpecArgs {
    #[arg(long, short = 'o', required = true)]
    out: PathBuf,
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
        Some(Commands::UploadAllSpecs(args)) => upload_pkg_specs(client, args.target_dir).await?,
        Some(Commands::UploadSpec(args)) => upload_pkg_specs(client.clone(), args.target).await?,
        Some(Commands::WriteExistingModulesSpec(args)) => {
            write_existing_modules_spec(client, args.out).await?
        }
        Some(Commands::WriteAllSpecs(args)) => {
            write_all_specs(client, args.out.to_path_buf()).await?
        }
        Some(Commands::WriteSpec(args)) => {
            write_single_spec(client, args.spec_name, args.out.to_path_buf()).await?
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

enum ModuleState {
    HashesMatch,
    NeedsUpdated(String),
    New,
}
async fn write_existing_modules_spec(client: ModuleIndexClient, out: PathBuf) -> Result<()> {
    let modules = list_specs(client.clone()).await?;
    let mut entries: HashMap<String, String> = HashMap::new();

    for module in modules {
        if let Some(schema_id) = module.schema_id() {
            entries.insert(module.name, schema_id.to_string());
        }
    }
    let json_string = serde_json::to_string_pretty(&entries)?;
    fs::write(Path::new(&out), json_string)?;
    println!(
        "Wrote existing modules spec to {}",
        out.file_name()
            .expect("unable to get filename of file we just wrote")
            .to_string_lossy()
    );
    Ok(())
}

async fn write_single_spec(
    client: ModuleIndexClient,
    spec_name: String,
    out: PathBuf,
) -> Result<()> {
    let module = list_specs(client.clone())
        .await?
        .iter()
        .find(|m| m.name == spec_name)
        .cloned()
        .unwrap_or_else(|| panic!("Unable to find spec with name: {}", spec_name));
    write_spec(client, module.id, out).await
}

async fn write_all_specs(client: ModuleIndexClient, out: PathBuf) -> Result<()> {
    let specs = list_specs(client.clone()).await?;

    let pb = setup_progress_bar(specs.len() as u64);
    let mut joinset = tokio::task::JoinSet::new();

    for module in specs {
        let pb = pb.clone();
        let out = out.clone();
        let client = client.clone();

        joinset.spawn(async move {
            if let Err(e) = write_spec(client, module.id, out).await {
                println!("Failed to download {} due to {}", module.name, e);
            }
            pb.set_message(format!("Downloading: {}", module.name));
            pb.inc(1);
        });
    }

    pb.set_message("⏰ Waiting for all downloads to complete...");
    joinset.join_all().await;
    pb.finish_with_message("✨ All downloads complete!");
    Ok(())
}

async fn write_spec(client: ModuleIndexClient, module_id: String, out: PathBuf) -> Result<()> {
    let pkg = SiPkg::load_from_bytes(
        &client
            .download_module(Ulid::from_string(&module_id)?)
            .await?,
    )?;
    let spec = pkg.to_spec().await?;
    let spec_name = format!("{}.json", spec.name);
    fs::create_dir_all(&out)?;
    fs::write(
        Path::new(&out).join(spec_name),
        serde_json::to_string_pretty(&spec)?,
    )?;
    Ok(())
}

async fn upload_pkg_spec(client: ModuleIndexClient, pkg: SiPkg) -> Result<()> {
    let schema = pkg.schemas()?[0].clone();
    let metadata = pkg.metadata()?;

    client
        .upsert_builtin(
            metadata.name(),
            metadata.version(),
            Some(metadata.hash().to_string()),
            schema.unique_id().map(String::from),
            pkg.write_to_bytes()?,
            schema.variants()?[0].unique_id().map(String::from),
            Some(metadata.version().to_string()),
        )
        .await?;

    Ok(())
}

async fn upload_pkg_specs(client: ModuleIndexClient, target_dir: PathBuf) -> Result<()> {
    let specs: Vec<_> = if target_dir.is_file() {
        if let Some(parent) = target_dir.parent() {
            fs::read_dir(parent)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path() == target_dir)
                .collect()
        } else {
            vec![]
        }
    } else {
        fs::read_dir(&target_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "json")
            })
            .collect()
    };

    let mut joinset = JoinSet::new();

    let mut no_action_needed = 0;
    let mut new_modules = 0;
    let mut modules_with_updates = 0;

    let mut categorized_modules: Vec<(SiPkg, Option<String>)> = Vec::new();

    println!("Building modules list...");
    let existing_specs = &list_specs(client.clone()).await?;
    for spec in &specs {
        let pkg = json_to_pkg(spec.path())?;
        let schema = pkg.schemas()?[0].clone();
        let pkg_schema_id = schema.unique_id().unwrap();

        match remote_module_state(
            pkg_schema_id.to_string(),
            pkg.hash()?.to_string(),
            existing_specs,
        )
        .await?
        {
            ModuleState::HashesMatch => no_action_needed += 1,
            ModuleState::NeedsUpdated(module_id) => {
                modules_with_updates += 1;
                categorized_modules.push((pkg, Some(module_id)));
            }
            ModuleState::New => {
                new_modules += 1;
                categorized_modules.push((pkg, None));
            }
        }
    }

    println!(
        "🟰 {} modules have matching hashes and will be skipped",
        no_action_needed
    );
    println!(
        "🔼 {} modules exist and will be updated",
        modules_with_updates
    );
    println!("➕ {} new modules will be uploaded", new_modules);

    if categorized_modules.is_empty() {
        println!("No new modules or update, nothing to do!");
        std::process::exit(0);
    }

    println!("Would you like to continue? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
        return Ok(());
    }

    let pb = setup_progress_bar(categorized_modules.len() as u64);

    for (pkg, _existing_module_id) in categorized_modules {
        let client = client.clone();
        let pb = pb.clone();

        let metadata = pkg.metadata()?;
        joinset.spawn(async move {
            if let Err(e) = upload_pkg_spec(client, pkg).await {
                println!("Failed to upload {} due to {}", metadata.name(), e);
            }
            pb.set_message(format!("Uploading: {}", metadata.name()));
            pb.inc(1);
        });
    }

    pb.set_message("⏰ Waiting for all uploads to complete...");
    joinset.join_all().await;
    pb.finish_with_message("✨ All uploads complete!");
    Ok(())
}

fn json_to_pkg(spec: PathBuf) -> Result<SiPkg> {
    let buf = fs::read_to_string(&spec)?;
    let spec: PkgSpec = serde_json::from_str(&buf)?;
    Ok(SiPkg::load_from_spec(spec)?)
}

async fn list_specs(client: ModuleIndexClient) -> Result<Vec<ModuleDetailsResponse>> {
    Ok(client
        .list_builtins()
        .await?
        .modules
        .into_iter()
        .filter(|m| {
            m.owner_display_name
                .as_ref()
                .is_some_and(|n| n == CLOVER_DEFAULT_CREATOR)
        })
        .collect::<Vec<ModuleDetailsResponse>>())
}

async fn remote_module_state(
    schema_id: String,
    hash: String,
    modules: &Vec<ModuleDetailsResponse>,
) -> Result<ModuleState> {
    for module in modules {
        if let Some(module_schema_id) = &module.schema_id {
            if *module_schema_id == schema_id {
                if module.latest_hash == hash {
                    return Ok(ModuleState::HashesMatch);
                } else {
                    return Ok(ModuleState::NeedsUpdated(module.id.clone()));
                }
            }
        }
    }
    Ok(ModuleState::New)
}

fn setup_progress_bar(length: u64) -> Arc<ProgressBar> {
    let pb = Arc::new(ProgressBar::new(length));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})\n{msg}")
            .expect("could not build progress bar")
            .progress_chars("▸▹▹"),
    );
    pb
}
