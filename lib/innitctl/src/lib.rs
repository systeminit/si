use std::{
    io,
    path::PathBuf,
};

use handlebars::Handlebars;
pub use si_settings::{
    ConfigMap,
    ParameterProvider,
};
use telemetry::tracing::{
    debug,
    info,
};
use thiserror::Error;

pub use self::config::{
    Config,
    ConfigError,
    ConfigFile,
    StandardConfigFile,
    detect_and_configure_development,
};
pub mod config;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum InnitCtlError {
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("handlebars error: {0}")]
    HandlebarsRender(#[from] handlebars::RenderError),
    #[error("handlebars error: {0}")]
    HandlebarsTemplate(#[from] handlebars::TemplateError),
    #[error("io error: {0}")]
    IO(#[from] io::Error),
}

type Result<T> = std::result::Result<T, InnitCtlError>;

pub async fn templatize(config: &Config) -> Result<()> {
    debug!("Reading config directory...");
    let mut files = tokio::fs::read_dir(config.config_directory()).await?;

    while let Some(file) = files.next_entry().await? {
        let file_name = file.file_name().to_string_lossy().to_string();
        debug!("Processing: {file_name}");
        if !file_name.ends_with(".template") {
            info!("Skipping non-template file: {file_name}");
            continue;
        }

        let output_name = file_name.strip_suffix(".template").unwrap_or(&file_name);
        let rendered = generate_from_config(config, &file.path()).await?;
        debug!("Rendered: {output_name}");
        let output_path = config.output_directory().as_path().join(output_name);

        if let Some(parent) = output_path.parent() {
            debug!("Creating parent directory for: {}", output_path.display());
            tokio::fs::create_dir_all(parent).await?;
        }
        debug!("Writing rendered file: {output_name}");
        tokio::fs::write(output_path, rendered).await?;
        info!("Rendered and wrote: {output_name}");
    }
    Ok(())
}

async fn generate_from_config(config: &Config, template: &PathBuf) -> Result<String> {
    let hb = Handlebars::new();
    let template = tokio::fs::read_to_string(template).await?;
    let config_file: ConfigFile = config.try_into()?;
    Ok(hb.render_template(&template, &config_file)?)
}
