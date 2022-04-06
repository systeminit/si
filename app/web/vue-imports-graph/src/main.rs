use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, MAIN_SEPARATOR};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum Error {
    #[error("expected Some(T) for Option<T>, found None")]
    None,
}

fn main() -> Result<()> {
    // Split path lookups from results lookups for ergonomics.
    let mut results: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    let mut paths: HashMap<String, String> = HashMap::new();

    let cargo_manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = cargo_manifest_dir.parent().ok_or(Error::None)?.join("src");

    for entry in WalkDir::new(&src) {
        let entry = entry?;
        let name = entry.file_name().to_str().ok_or(Error::None)?;
        if name.ends_with(".vue") {
            let file = File::open(entry.path())?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;

                // Find the word "import" in local Vue files that do not contain comments.
                // This is likely broken.
                if line.contains("import") && line.contains(".vue") && !line.contains("//") {
                    let mut chunks = Vec::new();

                    // Handle in-line imports here and everything else in the "else" clause.
                    if line.contains("import(") {
                        for chunk in line.split(' ') {
                            if chunk.contains("import") {
                                chunks.push(strip_chunk(chunk));
                            }
                        }
                    } else {
                        // Grab anything that isn't from this set. The last chunk should
                        // always (in theory) be where the imports are coming from.
                        for chunk in line.split(' ') {
                            if chunk != "import"
                                && chunk != "from"
                                && chunk != "{"
                                && chunk != "}"
                                && chunk != "type"
                                && !chunk.is_empty()
                            {
                                chunks.push(strip_chunk(chunk));
                            }
                        }
                    }

                    results
                        .entry(name.to_string())
                        .or_insert(Vec::new())
                        .push(chunks);

                    let relevant_path = derive_relevant_path(entry.path(), &src)?;
                    paths.insert(name.to_string(), relevant_path);
                }
            }
        }
    }

    println!("{:#?}", results);
    println!("{:#?}", paths);
    Ok(())
}

fn strip_chunk(chunk: &str) -> String {
    // Handle extra quotes.
    let chunk = strip_prefix_or_return(chunk, "\"");
    let chunk = strip_suffix_or_return(chunk, "\"");

    // Handle where the import is coming from.
    let chunk = strip_suffix_or_return(chunk, "\";");

    // Handle scenarios where there are multiple imports.
    let chunk = strip_suffix_or_return(chunk, ",");

    chunk.to_string()
}

/// Strips the front of a [`Path`] with a given prefix.
fn derive_relevant_path(path: &Path, prefix: &Path) -> Result<String> {
    let path_string = path.as_os_str().to_str().ok_or(Error::None)?;
    let prefix_string = prefix.as_os_str().to_str().ok_or(Error::None)?;

    let relevant_path = strip_prefix_or_return(path_string, prefix_string);
    let main_seperator = MAIN_SEPARATOR.to_string();
    let relevant_path_without_leading_seperator =
        strip_prefix_or_return(relevant_path, &main_seperator);

    Ok(relevant_path_without_leading_seperator.to_string())
}

fn strip_prefix_or_return<'a>(string: &'a str, prefix: &'a str) -> &'a str {
    match string.strip_prefix(prefix) {
        Some(stripped) => stripped,
        None => string,
    }
}

fn strip_suffix_or_return<'a>(string: &'a str, suffix: &'a str) -> &'a str {
    match string.strip_suffix(suffix) {
        Some(stripped) => stripped,
        None => string,
    }
}
