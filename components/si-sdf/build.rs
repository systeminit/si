use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/data/migrations");
    for entry in fs::read_dir("./src/data/migrations")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    Ok(())
}
