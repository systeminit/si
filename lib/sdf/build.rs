#[cfg(debug_assertions)]
#[cfg(not(target_os = "windows"))]
const NEWLINE: &str = "\n";

#[cfg(debug_assertions)]
#[cfg(target_os = "windows")]
const NEWLINE: &str = "\r\n";

#[cfg(debug_assertions)]
const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[cfg(debug_assertions)]
#[derive(Debug, thiserror::Error)]
enum BuildError {
    #[error("git command failed (code: {0:?}) stderr: {1}")]
    GitCommandFailed(Option<i32>, String),
    #[error("stripping newline suffix returned empty for stdout: {0}")]
    StripNewlineSuffixReturnedEmpty(String),
}

// NOTE(nick): only use "build.rs" in "dev mode" for the Git SHA (we've been using "opt-level"
// to determine whether or not we are in "dev mode".
#[cfg(debug_assertions)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(CARGO_MANIFEST_DIR)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(BuildError::GitCommandFailed(output.status.code(), stderr).into());
    }
    let stdout = String::from_utf8(output.stdout)?;
    let sha = stdout
        .strip_suffix(NEWLINE)
        .ok_or_else(|| BuildError::StripNewlineSuffixReturnedEmpty(stdout.clone()))?;

    println!("cargo:rustc-env=SI_CURRENT_GIT_SHA={sha}");
    Ok(())
}

#[cfg(not(debug_assertions))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
