#[cfg(feature = "load-sync")]
pub fn load<C>(
    app_name: impl AsRef<str>,
    file_formats: impl crate::ToFileFormats,
    env_var: &Option<impl AsRef<std::ffi::OsStr>>,
) -> Result<Option<C>, crate::ConfigFileError>
where
    C: serde::de::DeserializeOwned,
{
    use crate::find;

    let (target, file_format) = match find(app_name, file_formats, env_var)? {
        Some(target) => target,
        None => return Ok(None),
    };
    let buf = read_from_file(target)?;
    Ok(Some(load_from_str(&buf, file_format)?))
}

#[cfg(feature = "load-async")]
pub async fn load_async<C>(
    app_name: impl AsRef<str>,
    file_formats: impl crate::ToFileFormats,
    env_var: &Option<impl AsRef<std::ffi::OsStr>>,
) -> Result<Option<C>, crate::ConfigFileError>
where
    C: serde::de::DeserializeOwned,
{
    use crate::find;

    let (target, file_format) = match find(app_name, file_formats, env_var)? {
        Some(target) => target,
        None => return Ok(None),
    };
    let buf = read_from_file_async(&target).await?;
    Ok(Some(load_from_str(&buf, file_format)?))
}

#[cfg(feature = "load-sync")]
pub fn load_or_default<C>(
    app_name: impl AsRef<str>,
    file_formats: impl crate::ToFileFormats,
    env_var: &Option<impl AsRef<std::ffi::OsStr>>,
) -> Result<C, crate::ConfigFileError>
where
    C: serde::de::DeserializeOwned + Default,
{
    match load(app_name, file_formats, env_var)? {
        Some(c) => Ok(c),
        None => Ok(C::default()),
    }
}

#[cfg(feature = "load-async")]
pub async fn load_or_default_async<C>(
    app_name: impl AsRef<str>,
    file_formats: impl crate::ToFileFormats,
    env_var: &Option<impl AsRef<std::ffi::OsStr>>,
) -> Result<C, crate::ConfigFileError>
where
    C: serde::de::DeserializeOwned + Default,
{
    match load_async(app_name, file_formats, env_var).await? {
        Some(c) => Ok(c),
        None => Ok(C::default()),
    }
}

#[cfg(feature = "load-str")]
pub fn load_from_str<C>(
    s: &str,
    file_format: crate::FileFormat,
) -> Result<C, crate::ConfigFileError>
where
    C: serde::de::DeserializeOwned,
{
    use crate::{
        ConfigFileError,
        FileFormat,
    };

    match file_format {
        #[cfg(feature = "load-toml")]
        FileFormat::Toml => serde_toml::from_str(s).map_err(Into::into),
        #[cfg(feature = "load-json")]
        FileFormat::Json => serde_json::from_str(s).map_err(Into::into),
        #[cfg(feature = "load-yaml")]
        FileFormat::Yaml => serde_yaml::from_str(s).map_err(Into::into),
        FileFormat::Custom(custom) => Err(ConfigFileError::UnknownFileFormat(custom.to_string())),
    }
}

#[cfg(feature = "load-sync")]
fn read_from_file(path: impl AsRef<std::path::Path>) -> Result<String, crate::ConfigFileError> {
    use std::{
        fs::File,
        io::{
            BufReader,
            Read,
        },
    };

    use crate::ConfigFileError;

    let mut buf = String::with_capacity(
        path.as_ref()
            .metadata()
            .map_err(ConfigFileError::ReadIO)?
            .len()
            .try_into()?,
    );

    let mut file = BufReader::new(File::open(&path).map_err(ConfigFileError::ReadIO)?);
    file.read_to_string(&mut buf)
        .map_err(ConfigFileError::ReadIO)?;

    Ok(buf)
}

#[cfg(feature = "load-async")]
async fn read_from_file_async(
    path: impl AsRef<std::path::Path>,
) -> Result<String, crate::ConfigFileError> {
    use tokio::{
        fs::File,
        io::{
            AsyncReadExt,
            BufReader,
        },
    };

    use crate::ConfigFileError;

    let mut buf = String::with_capacity(
        path.as_ref()
            .metadata()
            .map_err(ConfigFileError::ReadIO)?
            .len()
            .try_into()?,
    );

    let mut file = BufReader::new(File::open(&path).await.map_err(ConfigFileError::ReadIO)?);
    file.read_to_string(&mut buf)
        .await
        .map_err(ConfigFileError::ReadIO)?;

    Ok(buf)
}
