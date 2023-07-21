use crate::CliResult;
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use si_posthog::PosthogClient;
use std::thread;
use std::time::Duration;
use std::{cmp::min, fmt::Write};

pub fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "install", "mode": mode}),
    );
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
