use std::{
    collections::HashMap,
    fs::{
        self,
        DirEntry,
    },
    path::{
        Path,
        PathBuf,
    },
    sync::{
        Arc,
        atomic::{
            AtomicU64,
            AtomicUsize,
            Ordering,
        },
    },
    time::Duration,
};

use clap::{
    CommandFactory,
    Parser,
};
use color_eyre::Result;
use commands::Commands;
use diff::{
    patch_list_to_changelog,
    rewrite_spec_for_diff,
};
use futures::stream::StreamExt;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use json_patch::diff;
use module_index_client::{
    ModuleDetailsResponse,
    ModuleIndexClient,
};
use rand::random;
use si_pkg::{
    PkgSpec,
    SiPkg,
};
use ulid::Ulid;
use url::Url;

use crate::diff::patch_list_to_summary;

mod commands;
mod diff;

const CLOVER_DEFAULT_CREATOR: &str = "Clover";

#[derive(Parser, Debug)]
#[command(name = "hoist", version = "0.1")]
#[command(about = "Gets and puts cloud control assets from the module index")]
struct Args {
    #[arg(
        long,
        short = 'e',
        default_value = "https://localhost:5157",
        env = "SI_MODULE_INDEX_URL"
    )]
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
    let client = ModuleIndexClient::new(Url::parse(endpoint)?, token)?;
    println!("Working with Module Index at: {endpoint}");

    match args.command {
        Some(Commands::AnonymizeSpecs(args)) => anonymize_specs(args.target_dir, args.out).await?,
        Some(Commands::UploadAllSpecs(args)) => {
            upload_pkg_specs(
                &client,
                args.target_dir,
                args.max_concurrent,
                args.skip_confirmation,
                args.non_interactive,
            )
            .await?
        }
        Some(Commands::UploadSpec(args)) => {
            upload_pkg_specs(
                &client,
                args.target,
                args.max_concurrent,
                args.skip_confirmation,
                args.non_interactive,
            )
            .await?
        }
        Some(Commands::ValidateSpecs(args)) => validate_specs(args.target_dir).await?,
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
                eprintln!("Error displaying help: {err}");
                std::process::exit(1);
            }
            std::process::exit(0);
        }
        Some(Commands::CompareSpecs(args)) => {
            compare_specs(args.source_path, args.target_path).await?
        }
        Some(Commands::GetDiffSummary(args)) => {
            diff_summaries_with_module_index(&client, args.target_dir).await?
        }
        Some(Commands::GetDiffForAsset(args)) => {
            detailed_diff_with_module_index(&client, args.target_path).await?
        }
    }

    Ok(())
}

async fn detailed_diff_with_module_index(
    client: &ModuleIndexClient,
    target_path: PathBuf,
) -> Result<()> {
    let new_spec = json_to_spec(target_path)?;

    let existing_specs = &list_specs(client.clone()).await?;

    // TODO deal with func changes

    let pkg = SiPkg::load_from_spec(new_spec.clone())?;
    let metadata = pkg.metadata()?;
    println!("### Diffed *{}* with the module index:", metadata.name());

    let remote_module_id = match remote_module_state(pkg, existing_specs).await? {
        ModuleState::HashesMatch => {
            println!("Asset matches the module index hash");
            return Ok(());
        }
        ModuleState::NeedsUpdate(remote_module_id) => remote_module_id,
        ModuleState::New => {
            println!("Asset is brand new");
            return Ok(());
        }
    };

    let remote_ulid = Ulid::from_string(&remote_module_id)?;

    let module_bytes = client.download_module(remote_ulid).await?;

    let current_spec = SiPkg::load_from_bytes(&module_bytes)?
        .to_spec()
        .await?
        .anonymize();

    let new_spec = new_spec.anonymize();

    let new_spec_json = rewrite_spec_for_diff(serde_json::to_value(new_spec)?);
    let current_spec_json = rewrite_spec_for_diff(serde_json::to_value(current_spec)?);

    let patch = diff(&current_spec_json, &new_spec_json);

    let logs = patch_list_to_changelog(patch.clone());
    println!("{}", logs.join("\n"));

    Ok(())
}

enum ModuleState {
    HashesMatch,
    NeedsUpdate(String), // Contains Remote Module Id
    New,
}

async fn validate_specs(target_dir: PathBuf) -> Result<()> {
    let specs = spec_from_dir_or_file(target_dir)?;
    let mut success_count = 0;
    let mut error_count = 0;

    println!("Validating {} spec(s)...", specs.len());

    for spec_entry in specs {
        let spec_path = spec_entry.path();
        let spec_name = spec_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        match json_to_pkg(spec_path.clone()) {
            Ok(pkg) => match pkg.metadata() {
                Ok(metadata) => {
                    println!(
                        "✅ {}: {} ({})",
                        spec_name,
                        metadata.name(),
                        metadata.version()
                    );
                    success_count += 1;
                }
                Err(e) => {
                    println!("❌ {spec_name}: Failed to get metadata - {e}");
                    error_count += 1;
                }
            },
            Err(e) => {
                println!("❌ {spec_name}: Failed to load spec - {e}");
                error_count += 1;
            }
        }
    }

    println!("\nValidation complete: {success_count} successful, {error_count} failed");

    if error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
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
        .unwrap_or_else(|| panic!("Unable to find spec with name: {spec_name}"));
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

async fn compare_specs(source: PathBuf, target: PathBuf) -> Result<()> {
    let source_spec = json_to_spec(source)?;
    let target_spec = json_to_spec(target)?;

    let source_spec = source_spec.anonymize();
    let target_spec = target_spec.anonymize();

    let source_hash = SiPkg::load_from_spec(source_spec.clone())?
        .metadata()?
        .hash();
    let target_hash = SiPkg::load_from_spec(target_spec.clone())?
        .metadata()?
        .hash();

    if source_hash == target_hash {
        println!("Specs match!");
        return Ok(());
    }

    let source_value = rewrite_spec_for_diff(serde_json::to_value(source_spec)?);
    let target_value = rewrite_spec_for_diff(serde_json::to_value(target_spec)?);

    let patch = diff(&source_value, &target_value);

    println!("found {} changes", patch.len());
    println!("{}", patch_list_to_changelog(patch).join("\n"));

    Ok(())
}

async fn diff_summaries_with_module_index(
    client: &ModuleIndexClient,
    target_dir: PathBuf,
) -> Result<()> {
    let specs: Vec<_> = spec_from_dir_or_file(target_dir)?;

    let existing_specs = &list_specs(client.clone()).await?;

    // TODO deal with func changes

    let hash_mismatches = Arc::new(AtomicUsize::new(0));
    let changes_with_summary = Arc::new(AtomicUsize::new(0));
    let total_added = Arc::new(AtomicUsize::new(0));

    let max_concurrent = 100;
    futures::stream::iter(specs)
        .for_each_concurrent(max_concurrent, |spec| {
            let hash_mismatches = hash_mismatches.clone();
            let changes_with_summary = changes_with_summary.clone();
            let total_added = total_added.clone();

            async move {
                let pkg = json_to_pkg(spec.path()).unwrap();
                let metadata = pkg.metadata().unwrap();

                let remote_module_id = match remote_module_state(pkg.clone(), existing_specs)
                    .await
                    .unwrap()
                {
                    ModuleState::HashesMatch => return,
                    ModuleState::NeedsUpdate(remote_module_id) => {
                        hash_mismatches.fetch_add(1, Ordering::SeqCst);
                        remote_module_id
                    }
                    ModuleState::New => {
                        total_added.fetch_add(1, Ordering::SeqCst);
                        println!("[{}]: new module", metadata.name());
                        return;
                    }
                };

                let remote_ulid = Ulid::from_string(&remote_module_id).unwrap();

                let module_bytes = client.download_module(remote_ulid).await.unwrap();
                let current_pkg = SiPkg::load_from_bytes(&module_bytes)
                    .unwrap()
                    .to_spec()
                    .await
                    .unwrap()
                    .anonymize();

                let new_pkg = pkg.to_spec().await.unwrap().anonymize();
                let new_spec_json = rewrite_spec_for_diff(serde_json::to_value(new_pkg).unwrap());
                let current_spec_json =
                    rewrite_spec_for_diff(serde_json::to_value(current_pkg).unwrap());

                let patch = diff(&current_spec_json, &new_spec_json);

                let (maybe_change_summary, _this_changed_funcs) =
                    patch_list_to_summary(metadata.name(), patch.clone());

                if let Some(summary) = maybe_change_summary {
                    changes_with_summary.fetch_add(1, Ordering::SeqCst);
                    println!("{summary}");
                } else {
                    println!(
                        "[{}]: hash mismatch but no summary generated",
                        metadata.name()
                    );
                }
            }
        })
        .await;

    let total_added = total_added.load(Ordering::SeqCst);
    let changes_with_summary = changes_with_summary.load(Ordering::SeqCst);
    let hash_mismatches = hash_mismatches.load(Ordering::SeqCst);

    println!("Total: {total_added} new asset(s), {hash_mismatches} changed asset(s)");

    if changes_with_summary != hash_mismatches {
        println!(
            "Failed to generate change summary for {} assets that had hash mismatches",
            hash_mismatches - changes_with_summary,
        );
    }

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

async fn upload_pkg_spec(client: &ModuleIndexClient, pkg: &SiPkg) -> Result<()> {
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

async fn upload_pkg_specs(
    client: &ModuleIndexClient,
    target_dir: PathBuf,
    max_concurrent: usize,
    skip_confirmation: bool,
    non_interactive: bool,
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
        let msg = format!("Parsing module: {}", spec.file_name().to_string_lossy());
        pb.set_message(msg.to_string());
        if non_interactive {
            println!("{msg}");
        };

        let pkg = json_to_pkg(spec.path())?;
        let metadata = pkg.metadata()?;

        match remote_module_state(pkg.clone(), existing_specs).await? {
            ModuleState::HashesMatch => no_action_needed += 1,
            ModuleState::NeedsUpdate(_) => {
                modules_with_updates.push(metadata.name().to_string());
                categorized_modules.push((pkg.clone(), metadata));
            }
            ModuleState::New => {
                new_modules.push(metadata.name().to_string());
                categorized_modules.push((pkg, metadata));
            }
        }
    }

    println!("🟰 {no_action_needed} modules have matching hashes and will be skipped");
    println!(
        "🔼 {} modules exist and will be updated",
        modules_with_updates.len()
    );
    println!("➕ {} new modules will be uploaded", new_modules.len());

    if categorized_modules.is_empty() {
        println!("No new modules or update, nothing to do!");
        std::process::exit(0);
    }

    if !skip_confirmation {
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
                        println!("{module}");
                    }
                }
                "u" => {
                    for module in &modules_with_updates {
                        println!("{module}");
                    }
                }

                _ => return Ok(()),
            }
        }
    }

    // Set up progress bar
    let total = categorized_modules.len() as u64;
    let pb = setup_progress_bar(total);
    let msg = "⏰ Beginning uploads ...";
    pb.set_message(msg);
    if non_interactive {
        println!("{msg}");
    };

    // Generates the "X failed" message for various set_message() calls to use
    let failed = AtomicU64::new(0);
    let failed_message = || {
        let failed = failed.load(Ordering::Relaxed);
        if failed > 0 {
            format!(" ❌ {failed} failed.  ")
        } else {
            "".to_string()
        }
    };

    futures::stream::iter(categorized_modules)
        .for_each_concurrent(max_concurrent, |(pkg, metadata)| {
            let pb = pb.clone();
            let failed = &failed;
            let msg = format!("{}⏰ Uploading: {}", failed_message(), metadata.name());
            pb.set_message(msg.to_string());
            if non_interactive {
                println!("{msg}");
            };
            pb.inc(1);
            async move {
                let max_retries = 5;
                let mut retries = 0;
                let mut backoff_ms = 100;

                loop {
                    match upload_pkg_spec(client, &pkg).await {
                        Ok(_) => break,
                        Err(e) => {
                            retries += 1;
                            if retries > max_retries {
                                println!(
                                    "Failed to upload {} after {} retries due to {}",
                                    metadata.name(),
                                    max_retries,
                                    e
                                );
                                failed.fetch_add(1, Ordering::Relaxed);
                                pb.set_message(failed_message());
                                break;
                            }

                            println!(
                                "Retrying upload for {} (attempt {}/{}): {}",
                                metadata.name(),
                                retries,
                                max_retries,
                                e
                            );

                            let jitter = random::<u64>() % 50; // 0-49ms of jitter
                            tokio::time::sleep(Duration::from_millis(backoff_ms + jitter)).await;
                            backoff_ms *= 2;
                        }
                    }
                }
            }
        })
        .await;

    let msg = format!(
        "✨ {} uploads complete!{}",
        total - failed.load(Ordering::Relaxed),
        failed_message(),
    );
    pb.finish_with_message(msg.to_string());
    if non_interactive {
        println!("{msg}");
    };
    // If this message is not here, the console does not show the final message for some reason
    println!("Done");
    Ok(())
}

fn json_to_pkg(spec: PathBuf) -> Result<SiPkg> {
    Ok(SiPkg::load_from_spec(json_to_spec(spec)?)?)
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
    let schema = pkg.schemas()?[0].clone();

    // FIXME(victor, scott) Converting pkg to bytes changes the hash, and since we calculate hashes
    // on the module index, we need to make this conversion here too to get the same hashes
    let pkg = SiPkg::load_from_bytes(&pkg.write_to_bytes()?)?;

    let structural_hash = SiPkg::load_from_spec(pkg.to_spec().await?.anonymize())?
        .metadata()?
        .hash()
        .to_string();
    let schema_id = schema
        .unique_id()
        .expect("generated module does not have a schema_id");

    for module in modules {
        if Some(schema_id) == module.schema_id.as_deref() {
            if Some(structural_hash) == module.structural_hash {
                return Ok(ModuleState::HashesMatch);
            }
            return Ok(ModuleState::NeedsUpdate(module.id.clone()));
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
