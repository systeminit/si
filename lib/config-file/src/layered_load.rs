use std::{collections::HashMap, env, ffi::OsStr, fmt::Debug};

use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, trace};

use crate::{ConfigFileError, FileFormat, Result, ToFileFormats};

mod ser;

#[derive(Debug)]
pub struct ConfigMap {
    inner: config::Config,
    empty: bool,
}

impl Default for ConfigMap {
    fn default() -> Self {
        Self {
            inner: config::Config::default(),
            empty: true,
        }
    }
}

impl ConfigMap {
    pub fn set(&mut self, key: impl AsRef<str>, value: impl Into<config::Value>) -> &mut Self {
        // We entirely own this inner `config::Config` and I can't any reference to where and how
        // this instance could get into a `ConfigKind::Frozen` state (I suspect their `Environment`
        // type uses this).
        //
        // As a result, this expect is expected to never panic. If it does, then the API for
        // `config::Config` has changed and this is a programming bug.
        self.inner
            .set(key.as_ref(), value)
            .expect("inner config should not ever be frozen");
        self.empty = false;
        self
    }

    fn into_inner(self) -> config::Config {
        self.inner
    }
}

pub fn layered_load<C, F>(
    app_name: impl AsRef<str>,
    file_formats: impl ToFileFormats,
    config_env_var: &Option<impl AsRef<OsStr>>,
    env_config_prefix: &Option<impl AsRef<str>>,
    set_func: F,
) -> Result<C>
where
    C: Clone + DeserializeOwned + Debug + Default + Send + Serialize + Sync + 'static,
    F: FnOnce(&mut ConfigMap),
{
    let app_name = app_name.as_ref();

    let mut builder = config::Config::default();
    add_defaults::<C>(&mut builder)?;
    add_config_file(app_name, file_formats, config_env_var, &mut builder)?;
    add_environment_config(env_config_prefix, &mut builder)?;
    add_programatic_config(set_func, &mut builder)?;
    into_config_file(builder)
}

fn add_defaults<C>(builder: &mut config::Config) -> Result<()>
where
    C: Clone + Debug + Default + Send + Serialize + Sync + 'static,
{
    let serde_source = SerdeSource::new(C::default());
    trace!("merging defaults config for defaults={:?}", &serde_source);
    builder.merge(serde_source)?;
    Ok(())
}

/// Add a config file, if relevant.
///
/// # Implementation
///
/// * Look for a config file, using `load::find` as it'll look in the right places, in
///   the right order while also checking for a location in an environment variable.
/// * We'll ultimately use the `config` crate to merge the config file in with other inputs, but to
///   feed its API, we need the found file as a relative path to the current directory (why, why,
///   why????), so compute this, using the `pathdiff` crate which was an extraction from internal
///   Rust core code.
/// * Then we need to get determine the file type for the `config` crate, so we'll convert from the
///   file type determined by the `crate::FileType` type.
/// * Finally, merge in the file into the `config::Config` builder.
/// * I mean...
fn add_config_file(
    app_name: impl AsRef<str>,
    file_formats: impl ToFileFormats,
    env_var: &Option<impl AsRef<OsStr>>,
    builder: &mut config::Config,
) -> Result<()> {
    let app_name = app_name.as_ref();

    if let Some((target, file_format)) = crate::find(app_name, file_formats, env_var)? {
        // Config crate requires a relative path to a config file. /me sideeyes crate...
        let current_dir = env::current_dir().map_err(ConfigFileError::CurrentDirectory)?;
        let relative_target = pathdiff::diff_paths(&target, &current_dir).ok_or_else(|| {
            ConfigFileError::RelativePath(
                target.to_string_lossy().to_string(),
                current_dir.to_string_lossy().to_string(),
            )
        })?;
        // Determine the file type for the config crate, using the response we got from config_file
        // crate, oi
        let file_format = match file_format {
            #[cfg(feature = "toml")]
            FileFormat::Toml => config::FileFormat::Toml,
            #[cfg(feature = "json")]
            FileFormat::Json => config::FileFormat::Json,
            #[cfg(feature = "yaml")]
            FileFormat::Yaml => config::FileFormat::Yaml,
            FileFormat::Custom(unknown) => {
                return Err(ConfigFileError::UnknownFileFormat(unknown.to_string()))
                    .map_err(Into::into)
            }
            // If another file type is compiled in via cargo features, this arm will match
            #[allow(unreachable_patterns)]
            unexpected => {
                unimplemented!(
                    "new file format brought in via cargo features: {}",
                    unexpected.as_str()
                )
            }
        };

        let file = config::File::new(relative_target.to_string_lossy().as_ref(), file_format)
            .required(true);
        trace!("merging file config for file={:?}", &file);
        builder.merge(file)?;
    }

    Ok(())
}

fn add_environment_config(
    env_prefix: &Option<impl AsRef<str>>,
    builder: &mut config::Config,
) -> Result<()> {
    if let Some(env_prefix) = env_prefix {
        let env = config::Environment::with_prefix(env_prefix.as_ref())
            .separator("__")
            .ignore_empty(true);
        trace!(
            env_prefix = %format!("{}_",env_prefix.as_ref()),
            ?env,
            "merging environment config with"
        );
        builder.merge(env)?;
    }
    Ok(())
}

fn add_programatic_config<F>(set_func: F, builder: &mut config::Config) -> Result<()>
where
    F: FnOnce(&mut ConfigMap),
{
    let mut config_map = ConfigMap::default();
    set_func(&mut config_map);

    if config_map.empty {
        trace!("nothing set for programatic config, not merging");
    } else {
        let config = config_map.into_inner();
        trace!("merging programatic config for config={:?}", &config);
        builder.merge(config)?;
    }
    Ok(())
}

fn into_config_file<C>(builder: config::Config) -> Result<C>
where
    C: Debug + DeserializeOwned,
{
    let config_file = builder.try_into()?;
    debug!(?config_file, "merged configuration into");
    Ok(config_file)
}

#[derive(Clone, Debug)]
struct SerdeSource<T> {
    source: T,
}

impl<T> SerdeSource<T> {
    fn new(source: T) -> Self {
        Self { source }
    }
}

impl<T: Serialize> config::Source for SerdeSource<T>
where
    T: 'static,
    T: Sync + Send + Debug + Clone,
{
    fn clone_into_box(&self) -> Box<dyn config::Source + Send + Sync> {
        Box::new(self.clone())
    }

    fn collect(&self) -> std::result::Result<HashMap<String, config::Value>, config::ConfigError> {
        ser::to_hash_map(&self.source).map_err(|err| config::ConfigError::Message(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};

    #[test]
    fn test_struct() {
        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        struct Test {
            int: u32,
            seq: Vec<String>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a".to_string(), "b".to_string()],
        };
        let expected = test.clone();

        let mut c = config::Config::default();
        c.merge(SerdeSource::new(test)).expect("failed to merge");

        assert_eq!(1, c.get_int("int").expect("failed to get int"));
        assert_eq!(
            vec![config::Value::new(None, "a"), config::Value::new(None, "b")],
            c.get_array("seq").expect("failed to get seq")
        );

        let round_trip: Test = c.try_into().expect("failed to deserialize");
        assert_eq!(expected, round_trip);
    }

    #[test]
    fn complex_struct() {
        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        struct Settings {
            debug: f64,
            production: Option<String>,
            code: AsciiCode,
            place: Place,
            #[serde(rename = "arr")]
            elements: Vec<String>,
        }

        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        struct Place {
            number: PlaceNumber,
            name: String,
            longitude: f64,
            latitude: f64,
            favorite: bool,
            telephone: Option<String>,
            reviews: u64,
            creator: HashMap<String, String>,
            rating: Option<f32>,
        }

        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        struct PlaceNumber(u8);

        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        struct AsciiCode(i8);

        let mut creator = HashMap::new();
        creator.insert("name".to_string(), "Jane Smith".to_string());
        creator.insert("username".to_string(), "jsmith".to_string());
        creator.insert("email".to_string(), "jsmith@localhost".to_string());

        let input = Settings {
            debug: 0.0,
            production: Some("false".to_string()),
            code: AsciiCode(53),
            place: Place {
                number: PlaceNumber(1),
                name: "Torre di Pisa".to_string(),
                longitude: 43.7224985,
                latitude: 10.3970522,
                favorite: false,
                telephone: None,
                reviews: 3866,
                creator,
                rating: Some(4.5),
            },
            elements: vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
                "10".to_string(),
            ],
        };
        let mut expected = input.clone();

        let mut c = config::Config::default();
        c.merge(SerdeSource::new(input)).expect("failed to merge");
        // Let's set a new value programmatically, deep in the structure
        c.set("place.creator.name", "Rick Astley".to_string())
            .expect("failed to set value");

        // Now to update what we're expecting
        let old_value = expected
            .place
            .creator
            .insert("name".to_string(), "Rick Astley".to_string());
        assert_eq!(Some("Jane Smith".to_string()), old_value);

        // Deserialize back into our type and make sure the round trip with update was successful
        let round_trip: Settings = c.try_into().expect("failed to deserialize");
        assert_eq!(expected, round_trip);
    }
}
