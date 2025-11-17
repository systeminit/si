use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    fmt::Debug,
};

use serde::{
    Serialize,
    de::DeserializeOwned,
};
use tracing::trace;

use crate::{
    ConfigFileError,
    FileFormat,
    Result,
    ToFileFormats,
    parameter_provider::{
        ParameterProvider,
        ParameterSource,
    },
};

mod ser;

#[derive(Debug)]
pub struct ConfigMap {
    inner: std::collections::HashMap<String, config::Value>,
    empty: bool,
}

impl Default for ConfigMap {
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
            empty: true,
        }
    }
}

impl ConfigMap {
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<config::Value>) -> &mut Self {
        self.inner.insert(key.into(), value.into());
        self.empty = false;
        self
    }

    fn into_inner(self) -> HashMap<String, config::Value> {
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

    // Add defaults
    let mut builder = config::Config::builder();
    let serde_source = SerdeSource::new(<C>::default());
    trace!("merging defaults config for defaults={:?}", &serde_source);
    builder = builder.add_source(serde_source);

    // Add a config file, if relevant.
    //
    // # Implementation
    //
    // * Look for a config file, using `load::find` as it'll look in the right places, in
    //   the right order while also checking for a location in an environment variable.
    // * We'll ultimately use the `config` crate to merge the config file in with other inputs, but to
    //   feed its API, we need the found file as a relative path to the current directory (why, why,
    //   why????), so compute this, using the `pathdiff` crate which was an extraction from internal
    //   Rust core code.
    // * Then we need to get determine the file type for the `config` crate, so we'll convert from the
    //   file type determined by the `crate::FileType` type.
    // * Finally, merge in the file into the `config::Config` builder.
    // * I mean...
    if let Some((target, file_format)) = crate::find(app_name, file_formats, config_env_var)? {
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
                return Err(Into::into(ConfigFileError::UnknownFileFormat(
                    unknown.to_string(),
                )));
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
        builder = builder.add_source(file);
    }

    // Add environment config
    if let Some(env_prefix) = env_config_prefix {
        let env = config::Environment::with_prefix(env_prefix.as_ref())
            .separator("__")
            .ignore_empty(true);
        trace!(
            env_prefix = %format!("{}_",env_prefix.as_ref()),
            ?env,
            "merging environment config with"
        );
        builder = builder.add_source(env);
    }

    // Add programmatic config
    let mut config_map = ConfigMap::default();
    set_func(&mut config_map);

    if config_map.empty {
        trace!("nothing set for programmatic config, not merging");
    } else {
        let config_hash = config_map.into_inner();
        trace!("merging programmatic config for config={:?}", &config_hash);
        for (key, value) in config_hash.into_iter() {
            builder = builder
                .set_override(key, value)
                .expect("tried to set programmatic value that was impossible! bug!");
        }
    }

    // Deserialize it into a config file struct
    let config = builder.build()?;
    let config_file = config.try_deserialize()?;
    trace!(?config_file, "merged configuration into");
    Ok(config_file)
}

pub async fn layered_load_with_provider<C, F>(
    app_name: impl AsRef<str>,
    file_formats: impl ToFileFormats,
    config_env_var: &Option<impl AsRef<OsStr>>,
    env_config_prefix: &Option<impl AsRef<str>>,
    parameter_provider: Option<impl ParameterProvider + 'static>,
    set_func: F,
) -> Result<C>
where
    C: Clone + DeserializeOwned + Debug + Default + Send + Serialize + Sync + 'static,
    F: FnOnce(&mut ConfigMap),
{
    let app_name = app_name.as_ref();

    // Add defaults
    let mut builder = config::Config::builder();
    let serde_source = SerdeSource::new(<C>::default());
    trace!("merging defaults config for defaults={:?}", &serde_source);
    builder = builder.add_source(serde_source);

    // Parameter provider (if provided)
    if let Some(provider) = parameter_provider {
        let param_source = ParameterSource::new(provider, app_name.to_string());
        builder = param_source.load(builder).await?;
    }

    // Add a config file, if relevant.
    //
    // # Implementation
    //
    // * Look for a config file, using `load::find` as it'll look in the right places, in
    //   the right order while also checking for a location in an environment variable.
    // * We'll ultimately use the `config` crate to merge the config file in with other inputs, but to
    //   feed its API, we need the found file as a relative path to the current directory (why, why,
    //   why????), so compute this, using the `pathdiff` crate which was an extraction from internal
    //   Rust core code.
    // * Then we need to get determine the file type for the `config` crate, so we'll convert from the
    //   file type determined by the `crate::FileType` type.
    // * Finally, merge in the file into the `config::Config` builder.
    // * I mean...
    if let Some((target, file_format)) = crate::find(app_name, file_formats, config_env_var)? {
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
                return Err(Into::into(ConfigFileError::UnknownFileFormat(
                    unknown.to_string(),
                )));
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
        builder = builder.add_source(file);
    }

    // Add environment config
    if let Some(env_prefix) = env_config_prefix {
        let env = config::Environment::with_prefix(env_prefix.as_ref())
            .separator("__")
            .ignore_empty(true);
        trace!(
            env_prefix = %format!("{}_",env_prefix.as_ref()),
            ?env,
            "merging environment config with"
        );
        builder = builder.add_source(env);
    }

    // Add programmatic config
    let mut config_map = ConfigMap::default();
    set_func(&mut config_map);

    if config_map.empty {
        trace!("nothing set for programmatic config, not merging");
    } else {
        let config_hash = config_map.into_inner();
        trace!("merging programmatic config for config={:?}", &config_hash);
        for (key, value) in config_hash.into_iter() {
            builder = builder
                .set_override(key, value)
                .expect("tried to set programmatic value that was impossible! bug!");
        }
    }

    // Deserialize it into a config file struct
    let config = builder.build()?;
    let config_file = config.try_deserialize()?;
    trace!(?config_file, "merged configuration into");
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
    use serde::{
        Deserialize,
        Serialize,
    };

    use super::*;

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

        let c = config::Config::builder()
            .add_source(SerdeSource::new(test))
            .build()
            .expect("cannot build config from source");

        assert_eq!(1, c.get_int("int").expect("failed to get int"));
        assert_eq!(
            vec![config::Value::new(None, "a"), config::Value::new(None, "b")],
            c.get_array("seq").expect("failed to get seq")
        );

        let round_trip: Test = c.try_deserialize().expect("failed to deserialize");
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

        let c = config::Config::builder()
            .add_source(SerdeSource::new(input))
            // Let's set a new value programmatically, deep in the structure
            .set_override("place.creator.name", "Rick Astley".to_string())
            .expect("set override value")
            .build()
            .expect("failed to set value");

        // Now to update what we're expecting
        let old_value = expected
            .place
            .creator
            .insert("name".to_string(), "Rick Astley".to_string());
        assert_eq!(Some("Jane Smith".to_string()), old_value);

        // Deserialize back into our type and make sure the round trip with update was successful
        let round_trip: Settings = c.try_deserialize().expect("failed to deserialize");
        assert_eq!(expected, round_trip);
    }

    #[test]
    fn enum_with_struct_variant() {
        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        #[serde(rename_all = "snake_case")]
        enum AuthConfig {
            StaticCredentials {
                access_key: String,
                secret_key: String,
            },
            IamRole,
        }

        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        struct TestConfig {
            name: String,
            auth: AuthConfig,
        }

        let test = TestConfig {
            name: "test-service".to_string(),
            auth: AuthConfig::StaticCredentials {
                access_key: "key123".to_string(),
                secret_key: "secret456".to_string(),
            },
        };
        let expected = test.clone();

        let c = config::Config::builder()
            .add_source(SerdeSource::new(test))
            .build()
            .expect("cannot build config from source");

        let round_trip: TestConfig = c.try_deserialize().expect("failed to deserialize");
        assert_eq!(expected, round_trip);
    }

    #[test]
    fn enum_deserialization_from_toml() {
        // Test to understand what format config crate expects for enum deserialization
        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        #[serde(rename_all = "snake_case")]
        enum AuthConfig {
            StaticCredentials {
                access_key: String,
                secret_key: String,
            },
            IamRole,
        }

        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        struct TestConfig {
            name: String,
            auth: AuthConfig,
        }

        // Standard externally-tagged TOML representation
        let toml_str = r#"
            name = "test-service"

            [auth.static_credentials]
            access_key = "key123"
            secret_key = "secret456"
        "#;

        let c = config::Config::builder()
            .add_source(config::File::from_str(toml_str, config::FileFormat::Toml))
            .build()
            .expect("cannot build config from TOML");

        let result: TestConfig = c
            .try_deserialize()
            .expect("failed to deserialize from TOML");
        assert_eq!("test-service", result.name);
        assert_eq!(
            AuthConfig::StaticCredentials {
                access_key: "key123".to_string(),
                secret_key: "secret456".to_string(),
            },
            result.auth
        );
    }

    #[test]
    fn enum_with_flat_hashmap() {
        // Test if we can build the right structure with flat dotted keys
        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        #[serde(rename_all = "snake_case")]
        enum AuthConfig {
            StaticCredentials {
                access_key: String,
                secret_key: String,
            },
            IamRole,
        }

        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        struct TestConfig {
            name: String,
            auth: AuthConfig,
        }

        // Try building with fully dotted keys
        let mut builder = config::Config::builder();
        builder = builder
            .set_override("name", "test-service")
            .expect("set name")
            .set_override("auth.static_credentials.access_key", "key123")
            .expect("set access_key")
            .set_override("auth.static_credentials.secret_key", "secret456")
            .expect("set secret_key");

        let c = builder.build().expect("build config");
        let result: TestConfig = c.try_deserialize().expect("deserialize");

        assert_eq!("test-service", result.name);
        assert_eq!(
            AuthConfig::StaticCredentials {
                access_key: "key123".to_string(),
                secret_key: "secret456".to_string(),
            },
            result.auth
        );
    }

    #[test]
    fn enum_unit_variant_flat_hashmap() {
        // Test that unit variants still work with flat keys
        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        #[serde(rename_all = "snake_case")]
        enum AuthConfig {
            StaticCredentials {
                access_key: String,
                secret_key: String,
            },
            IamRole,
        }

        #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
        struct TestConfig {
            name: String,
            auth: AuthConfig,
        }

        let mut builder = config::Config::builder();
        builder = builder
            .set_override("name", "test-service")
            .expect("set name")
            .set_override("auth", "iam_role")
            .expect("set auth to unit variant");

        let c = builder.build().expect("build config");
        let result: TestConfig = c.try_deserialize().expect("deserialize");

        assert_eq!("test-service", result.name);
        assert_eq!(AuthConfig::IamRole, result.auth);
    }
}
