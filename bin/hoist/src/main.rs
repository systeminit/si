use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use clap::{CommandFactory, Parser};
use color_eyre::{eyre::eyre, Result};
use commands::Commands;
use futures::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use module_index_client::{ModuleDetailsResponse, ModuleIndexClient};
use si_pkg::{PkgSpec, SiPkg};
use ulid::Ulid;
use url::Url;

mod commands;

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

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let endpoint = &args.endpoint;
    let token = &args.token;
    let client = ModuleIndexClient::new(Url::parse(endpoint)?, token);

    match args.command {
        Some(Commands::AnonymizeSpecs(args)) => anonymize_specs(args.target_dir, args.out).await?,
        Some(Commands::UploadAllSpecs(args)) => {
            upload_pkg_specs(&client, args.target_dir, args.max_concurrent).await?
        }
        Some(Commands::UploadSpec(args)) => {
            upload_pkg_specs(&client, args.target, args.max_concurrent).await?
        }
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
    NeedsUpdate,
    New,
}

async fn anonymize_specs(target_dir: PathBuf, out: PathBuf) -> Result<()> {
    fs::create_dir_all(&out)?;
    let specs = spec_from_dir_or_file(target_dir)?;
    for dir in specs {
        let spec = json_to_spec(dir.path())?;
        let spec_name = format!("{}.json", spec.name);

        fs::write(
            Path::new(&out).join(spec_name),
            serde_json::to_string_pretty(&spec.anonymize())?,
        )?;
    }

    Ok(())
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
    )
    .map_err(|e| eyre!(Box::new(e)))?;
    let spec = pkg.to_spec().await.map_err(|e| eyre!(Box::new(e)))?;
    let spec_name = format!("{}.json", spec.name);
    fs::create_dir_all(&out)?;
    fs::write(
        Path::new(&out).join(spec_name),
        serde_json::to_string_pretty(&spec)?,
    )?;
    Ok(())
}

async fn upload_pkg_spec(client: &ModuleIndexClient, pkg: &SiPkg) -> Result<()> {
    let schema = pkg.schemas().map_err(|e| eyre!(Box::new(e)))?[0].clone();
    let metadata = pkg.metadata().map_err(|e| eyre!(Box::new(e)))?;

    client
        .upsert_builtin(
            metadata.name(),
            metadata.version(),
            Some(metadata.hash().to_string()),
            schema.unique_id().map(String::from),
            pkg.write_to_bytes().map_err(|e| eyre!(Box::new(e)))?,
            schema.variants().map_err(|e| eyre!(Box::new(e)))?[0]
                .unique_id()
                .map(String::from),
            Some(metadata.version().to_string()),
        )
        .await?;

    Ok(())
}

async fn upload_pkg_specs(
    client: &ModuleIndexClient,
    target_dir: PathBuf,
    max_concurrent: usize,
) -> Result<()> {
    let specs: Vec<_> = spec_from_dir_or_file(target_dir)?;

    let mut no_action_needed = 0;
    let mut new_modules = vec![];
    let mut modules_with_updates = vec![];

    let mut categorized_modules = vec![];

    let existing_specs = &list_specs(client.clone()).await?;
    let pb = setup_progress_bar(specs.len() as u64);

    for spec in &specs {
        pb.inc(1);
        pb.set_message(format!(
            "Parsing module: {}",
            spec.file_name().to_string_lossy()
        ));

        let pkg = json_to_pkg(spec.path())?;
        let metadata = pkg.metadata().map_err(|e| eyre!(Box::new(e)))?;

        match remote_module_state(pkg.clone(), existing_specs).await? {
            ModuleState::HashesMatch => no_action_needed += 1,
            ModuleState::NeedsUpdate => {
                modules_with_updates.push(metadata.name().to_string());
                categorized_modules.push((pkg, metadata));
            }
            ModuleState::New => {
                new_modules.push(metadata.name().to_string());
                categorized_modules.push((pkg, metadata));
            }
        }
    }

    println!(
        "🟰 {} modules have matching hashes and will be skipped",
        no_action_needed
    );
    println!(
        "🔼 {} modules exist and will be updated",
        modules_with_updates.len()
    );
    println!("➕ {} new modules will be uploaded", new_modules.len());

    if categorized_modules.is_empty() {
        println!("No new modules or update, nothing to do!");
        std::process::exit(0);
    }

    loop {
        println!(
            "What would you like to do? [p]ush, list [n]ew assets, list [u]pdated assets, [c]ancel"
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "p" => break,
            "n" => {
                for module in &new_modules {
                    println!("{}", module);
                }
            }
            "u" => {
                for module in &modules_with_updates {
                    println!("{}", module);
                }
            }

            _ => return Ok(()),
        }
    }

    // Set up progress bar
    let total = categorized_modules.len() as u64;
    let pb = setup_progress_bar(total);
    pb.set_message("⏰ Beginning uploads ...");

    // Generates the "X failed" message for various set_message() calls to use
    let failed = AtomicU64::new(0);
    let failed_message = || {
        let failed = failed.load(Ordering::Relaxed);
        if failed > 0 {
            format!(" ❌ {} failed.  ", failed)
        } else {
            "".to_string()
        }
    };

    futures::stream::iter(categorized_modules)
        .for_each_concurrent(max_concurrent, |(pkg, metadata)| {
            let pb = pb.clone();
            let failed = &failed;
            pb.set_message(format!(
                "{}⏰ Uploading: {}",
                failed_message(),
                metadata.name(),
            ));
            pb.inc(1);
            async move {
                if let Err(e) = upload_pkg_spec(client, &pkg).await {
                    println!("Failed to upload {} due to {}", metadata.name(), e);
                    failed.fetch_add(1, Ordering::Relaxed);
                    pb.set_message(failed_message());
                }
            }
        })
        .await;

    pb.finish_with_message(format!(
        "✨ {} uploads complete!{}",
        total - failed.load(Ordering::Relaxed),
        failed_message(),
    ));
    // If this message is not here, the console does not show the final message for some reason
    println!("Done");
    Ok(())
}

fn json_to_pkg(spec: PathBuf) -> Result<SiPkg> {
    SiPkg::load_from_spec(json_to_spec(spec)?).map_err(|e| eyre!(Box::new(e)))
}

fn json_to_spec(spec: PathBuf) -> Result<PkgSpec> {
    let buf = fs::read_to_string(&spec)?;
    let spec: PkgSpec = serde_json::from_str(&buf)?;
    Ok(spec)
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
    pkg: SiPkg,
    modules: &Vec<ModuleDetailsResponse>,
) -> Result<ModuleState> {
    let schema = pkg.schemas().map_err(|e| eyre!(Box::new(e)))?[0].clone();

    // FIXME(victor, scott) Converting pkg to bytes changes the hash, and since we calculate hashes
    // on the module index, we need to make this conversion here too to get the same hashes
    let pkg = SiPkg::load_from_bytes(&pkg.write_to_bytes().map_err(|e| eyre!(Box::new(e)))?)
        .map_err(|e| eyre!(Box::new(e)))?;

    let structural_hash = SiPkg::load_from_spec(
        pkg.to_spec()
            .await
            .map_err(|e| eyre!(Box::new(e)))?
            .anonymize(),
    )
    .map_err(|e| eyre!(Box::new(e)))?
    .metadata()
    .map_err(|e| eyre!(Box::new(e)))?
    .hash()
    .to_string();
    let schema_id = schema.unique_id().unwrap();

    for module in modules {
        match (&module.schema_id, &module.structural_hash) {
            (Some(module_schema_id), Some(this_hash)) if *module_schema_id == schema_id => {
                return if *this_hash == structural_hash {
                    Ok(ModuleState::HashesMatch)
                } else {
                    Ok(ModuleState::NeedsUpdate)
                };
            }
            _ => {}
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

fn spec_from_dir_or_file(target_dir: PathBuf) -> Result<Vec<DirEntry>> {
    Ok(if target_dir.is_file() {
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
    })
}
