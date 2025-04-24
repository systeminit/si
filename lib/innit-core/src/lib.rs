use aws_sdk_ssm::types::Parameter as AwsParameter;
use config_file::parameter_provider::Parameter as ParameterProviderParameter;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: Option<String>,
}

impl From<AwsParameter> for Parameter {
    fn from(p: AwsParameter) -> Self {
        Self {
            name: p.name().unwrap_or_default().to_string(),
            value: p.value().map(|s| s.to_string()),
        }
    }
}

impl From<Parameter> for ParameterProviderParameter {
    fn from(p: Parameter) -> Self {
        Self {
            name: p.name,
            value: p.value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckHealthResponse {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParameterResponse {
    pub parameter: Parameter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListParametersResponse {
    pub parameters: Vec<Parameter>,
}
