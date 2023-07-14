use crate::args::{
    CheckArgs, Commands, InstallArgs, LaunchArgs, Mode, RestartArgs, StartArgs, StopArgs,
};
use color_eyre::Result;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use console::Emoji;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use rand::seq::SliceRandom;
use rand::Rng;
use std::thread;
use std::time::{Duration, Instant};
use std::{cmp::min, fmt::Write};

mod args;

static PACKAGES: &[&str] = &[
    "systeminit/sdf",
    "systeminit/council",
    "systeminit/veritech",
    "systeminit/pinga",
    "systeminit/web",
    "jaeger",
    "otelcol",
    "postgres",
    "nats",
];

static START_COMMANDS: &[&str] = &["docker run"];
static STOP_COMMANDS: &[&str] = &["docker stop"];
static RESTART_COMMANDS: &[&str] = &["docker restart"];

static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = args::parse();
    let mode = args.mode();

    println!(
        "{}",
        format_args!(
            "Starting System Initiative Launcher in {:?} mode",
            mode.to_string()
        )
    );

    match args.command {
        Commands::Install(args) => {
            let command_args = args;
            if !command_args.skip_check {
                check_system(CheckArgs {}, mode)?;
            }
            download_containers(command_args, mode)
        }
        Commands::Check(args) => check_system(args, mode),
        Commands::Launch(args) => launch_web(args, mode),
        Commands::Start(args) => start_si(args, mode),
        Commands::Restart(args) => restart_si(args, mode),
        Commands::Stop(args) => stop_si(args, mode),
    }
}

fn start_si(_args: StartArgs, _mode: Mode) -> Result<()> {
    let mut rng = rand::thread_rng();
    let started = Instant::now();
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let m = MultiProgress::new();
    let handles: Vec<_> = (0..8u32)
        .map(|i| {
            let count = rng.gen_range(30..80);
            let pb = m.add(ProgressBar::new(count));
            pb.set_style(spinner_style.clone());
            pb.set_prefix(format!("[{}/?]", i + 1));
            thread::spawn(move || {
                let mut rng = rand::thread_rng();
                let pkg = PACKAGES.choose(&mut rng).unwrap();
                for _ in 0..count {
                    let cmd = START_COMMANDS.choose(&mut rng).unwrap();
                    thread::sleep(Duration::from_millis(rng.gen_range(25..200)));
                    pb.set_message(format!("{pkg}: {cmd}"));
                    pb.inc(1);
                }
                pb.finish_with_message("waiting...");
            })
        })
        .collect();
    for h in handles {
        let _ = h.join();
    }
    m.clear().unwrap();

    println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));

    Ok(())
}

fn stop_si(_args: StopArgs, _mode: Mode) -> Result<()> {
    let mut rng = rand::thread_rng();
    let started = Instant::now();
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let m = MultiProgress::new();
    let handles: Vec<_> = (0..8u32)
        .map(|i| {
            let count = rng.gen_range(30..80);
            let pb = m.add(ProgressBar::new(count));
            pb.set_style(spinner_style.clone());
            pb.set_prefix(format!("[{}/?]", i + 1));
            thread::spawn(move || {
                let mut rng = rand::thread_rng();
                let pkg = PACKAGES.choose(&mut rng).unwrap();
                for _ in 0..count {
                    let cmd = STOP_COMMANDS.choose(&mut rng).unwrap();
                    thread::sleep(Duration::from_millis(rng.gen_range(25..200)));
                    pb.set_message(format!("{pkg}: {cmd}"));
                    pb.inc(1);
                }
                pb.finish_with_message("waiting...");
            })
        })
        .collect();
    for h in handles {
        let _ = h.join();
    }
    m.clear().unwrap();

    println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));

    Ok(())
}

fn restart_si(_args: RestartArgs, _mode: Mode) -> Result<()> {
    let mut rng = rand::thread_rng();
    let started = Instant::now();
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let m = MultiProgress::new();
    let handles: Vec<_> = (0..8u32)
        .map(|i| {
            let count = rng.gen_range(30..80);
            let pb = m.add(ProgressBar::new(count));
            pb.set_style(spinner_style.clone());
            pb.set_prefix(format!("[{}/?]", i + 1));
            thread::spawn(move || {
                let mut rng = rand::thread_rng();
                let pkg = PACKAGES.choose(&mut rng).unwrap();
                for _ in 0..count {
                    let cmd = RESTART_COMMANDS.choose(&mut rng).unwrap();
                    thread::sleep(Duration::from_millis(rng.gen_range(25..200)));
                    pb.set_message(format!("{pkg}: {cmd}"));
                    pb.inc(1);
                }
                pb.finish_with_message("waiting...");
            })
        })
        .collect();
    for h in handles {
        let _ = h.join();
    }
    m.clear().unwrap();

    println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));

    Ok(())
}

fn launch_web(_args: LaunchArgs, mode: Mode) -> Result<()> {
    let path = match mode {
        Mode::Local => "http://localhost:8080",
    };
    match open::that(path) {
        Ok(()) => println!("Opened '{}' successfully.", path),
        Err(err) => eprintln!("An error occurred when opening '{}': {}", path, err),
    }
    Ok(())
}

fn check_system(_args: CheckArgs, _mode: Mode) -> Result<()> {
    println!("Preparing for System Initiative Installation");
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100)
        .set_header(vec![
            Cell::new("Dependency").add_attribute(Attribute::Bold),
            Cell::new("Success?").add_attribute(Attribute::Bold),
        ])
        .add_row(vec![
            Cell::new("Detected Docker Engine").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Detected Docker Command").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Docker Compose Available").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Found `bash` in Nix environment").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Found nix environment").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Reasonable value for max open files").add_attribute(Attribute::Bold),
            Cell::new("    ❌    "),
        ]);

    println!("{table}");

    Ok(())
}

fn download_containers(_args: InstallArgs, mode: Mode) -> Result<()> {
    format_args!("Starting {:?} install of System Initiative", mode);
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "{spinner:.red} [{elapsed_precise}] [{wide_bar:.yellow/blue}] {bytes}/{total_bytes} ({eta})",
    )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-");

    let mut downloaded = 0;
    let total_size = 231231231;

    let pb = m.add(ProgressBar::new(total_size));
    pb.set_style(sty.clone());

    let pb2 = m.insert_after(&pb, ProgressBar::new(total_size));
    pb2.set_style(sty.clone());

    let pb3 = m.insert_after(&pb2, ProgressBar::new(total_size * 2));
    pb3.set_style(sty);

    m.println("Downloading System Initiative artifacts")
        .unwrap();

    let h1 = thread::spawn(move || {
        while downloaded < total_size {
            let new = min(downloaded + 223211, total_size);
            downloaded = new;
            pb.set_position(new);
            thread::sleep(Duration::from_millis(12));
        }
    });

    let h2 = thread::spawn(move || {
        while downloaded < total_size {
            let new = min(downloaded + 223211, total_size);
            downloaded = new;
            pb2.set_position(new);
            thread::sleep(Duration::from_millis(12));
        }
    });

    let h3 = thread::spawn(move || {
        while downloaded < total_size {
            let new = min(downloaded + 223211, total_size);
            downloaded = new;
            pb3.set_position(new);
            thread::sleep(Duration::from_millis(12));
        }
    });

    let _ = h1.join();
    let _ = h2.join();
    let _ = h3.join();

    m.println("System Initiative Successfully Installed")
        .unwrap();
    m.clear().unwrap();

    Ok(())
}
