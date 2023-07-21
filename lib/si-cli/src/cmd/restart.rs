use crate::CliResult;
use crate::{PACKAGES, RESTART_COMMANDS, SPARKLE};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::Rng;
use si_posthog::PosthogClient;
use std::thread;
use std::time::{Duration, Instant};

pub fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "restart-system", "mode": mode}),
    );
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
