use std::{
    env,
    fs,
    path::PathBuf,
};

use luminork_server::routes::openapi_handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_result = openapi_handler().await;

    let out_dir = env::args().nth(1).expect("No output passed");

    let api = match api_result {
        Ok(json_api) => json_api.0,
        Err((status, message)) => {
            return Err(format!("API Error ({status}): {message}").into());
        }
    };

    let output_file = PathBuf::from(out_dir);

    let json_content = serde_json::to_string_pretty(&api)?;
    fs::write(output_file, json_content)?;

    println!("OpenAPI spec written to data/openapi.json");
    Ok(())
}
