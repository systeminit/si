use config_file::parameter_provider::{
    Parameter as ParameterProviderParameter,
    ParameterType as ParameterProviderParameterType,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: Option<String>,
    pub r#type: Option<ParameterType>,
}

impl From<Parameter> for ParameterProviderParameter {
    fn from(p: Parameter) -> Self {
        Self {
            name: p.name,
            value: p.value,
            r#type: p.r#type.map(|t| match t {
                ParameterType::String => ParameterProviderParameterType::String,
                ParameterType::StringList => ParameterProviderParameterType::StringList,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    StringList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckHealthResponse {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateParameterRequest {
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct CreateParameterResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParameterResponse {
    pub parameter: Parameter,
    pub is_cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListParametersResponse {
    pub parameters: Vec<Parameter>,
    pub is_cached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshCacheResponse {
    pub success: bool,
}
