fn main() {
    // Print environment variables to verify mise exec is working
    if let Ok(rust_toolchain) = std::env::var("RUST_TOOLCHAIN") {
        println!("RUST_TOOLCHAIN: {}", rust_toolchain);
    } else {
        println!("RUST_TOOLCHAIN: not set");
    }
    
    if let Ok(rustup_home) = std::env::var("RUSTUP_HOME") {
        println!("RUSTUP_HOME: {}", rustup_home);
    } else {
        println!("RUSTUP_HOME: not set");
    }
    
    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        println!("CARGO_HOME: {}", cargo_home);
    } else {
        println!("CARGO_HOME: not set");
    }
}