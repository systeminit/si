use anyhow::{Context, Result};
use serde::Serialize;
use sodiumoxide::crypto::secretbox;
use toml;

#[derive(Debug, Serialize)]
struct Paging {
    key: sodiumoxide::crypto::secretbox::Key,
}

fn main() -> Result<()> {
    sodiumoxide::init();
    let paging = Paging {
        key: secretbox::gen_key(),
    };
    let result = toml::to_string(&paging)?;
    println!("{}", result);
    Ok(())
}
