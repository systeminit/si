use aws_sdk_ssm::types::Parameter as AwsParameter;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: Option<String>,
    pub version: Option<i64>,
}

impl From<AwsParameter> for Parameter {
    fn from(p: AwsParameter) -> Self {
        Self {
            name: p.name().unwrap_or_default().to_string(),
            value: p.value().map(|s| s.to_string()),
            version: Some(p.version()),
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
