use super::{
    Parameter,
    ParameterStoreError,
    ParameterStoreKind,
    ParameterStoreResult,
    ParameterType,
};

#[derive(Debug, Default, Clone)]
pub struct EnvParameterStorage;

impl EnvParameterStorage {
    pub fn new() -> Self {
        Self
    }

    /// Converts a parameter path to an environment variable name
    /// Example: `/si/todd/howard` -> `SI_TODD_HOWARD`
    fn path_to_env_var(path: &str) -> String {
        path.trim_start_matches('/')
            .replace('/', "_")
            .to_uppercase()
    }

    /// Checks if an environment variable key matches a path prefix
    fn env_var_matches_path(env_key: &str, path: &str) -> bool {
        let normalized = Self::path_to_env_var(path);
        // Match exact or with underscore separator to avoid false matches
        // e.g., SI_PROD matches SI_PROD_DB but not SI_PRODUCTION
        env_key == normalized || env_key.starts_with(&format!("{normalized}_"))
    }

    /// Converts an environment variable name back to a parameter path
    /// Example: `/si/todd/howard` -> `SI_TODD_HOWARD`
    fn env_var_to_path(env_var: &str) -> String {
        format!("/{}", env_var.replace('_', "/").to_lowercase())
    }
}

#[async_trait::async_trait]
impl ParameterStoreKind for EnvParameterStorage {
    async fn get_parameter(&self, name: String) -> ParameterStoreResult<Parameter> {
        let env_var = Self::path_to_env_var(&name);

        #[allow(clippy::disallowed_methods)]
        let value = std::env::var(&env_var)
            .map_err(|_| ParameterStoreError::ParameterNotFound(name.clone()))?;

        Ok(Parameter::new(name, value, ParameterType::String))
    }

    async fn parameters_by_path(&self, path: String) -> ParameterStoreResult<Vec<Parameter>> {
        if !path.starts_with("/") {
            return Err(ParameterStoreError::InvalidPath(path));
        }

        let mut parameters = Vec::new();

        #[allow(clippy::disallowed_methods)]
        for (key, value) in std::env::vars() {
            if Self::env_var_matches_path(&key, &path) {
                let param_name = Self::env_var_to_path(&key);
                parameters.push(Parameter::new(param_name, value, ParameterType::String));
            }
        }

        if parameters.is_empty() {
            return Err(ParameterStoreError::PathNotFound(path));
        }

        Ok(parameters)
    }

    async fn create_string_parameter(
        &self,
        _name: String,
        _value: String,
    ) -> ParameterStoreResult<()> {
        Err(ParameterStoreError::AttemptedWriteInEnvMode)
    }
}
