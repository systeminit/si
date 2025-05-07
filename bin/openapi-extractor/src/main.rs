use std::{
    fs,
    path::Path,
};

use luminork_server::routes::openapi_handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_result = openapi_handler().await;

    // Handle the result - properly handle the specific error type
    let api = match api_result {
        Ok(json_api) => json_api.0, // Extract the OpenApi from Json wrapper
        Err((status, message)) => {
            // Convert Axum error to standard error
            return Err(format!("API Error ({}): {}", status, message).into());
        }
    };

    // Create the output directory
    let output_dir = Path::new("data");
    fs::create_dir_all(output_dir)?;

    // Use serde_json for pretty printing
    let json_content = serde_json::to_string_pretty(&api)?;
    fs::write(output_dir.join("openapi.json"), json_content)?;

    println!("OpenAPI spec written to data/openapi.json");
    Ok(())
}
