use super::{
    Parameter,
    ParameterStoreKind,
    ParameterStoreResult,
    ParameterType,
};

fn convert_aws_parameter(ssm_parameter: si_data_ssm::SsmParameter) -> Parameter {
    Parameter::new(
        ssm_parameter.name().unwrap_or_default().to_string(),
        ssm_parameter.value().unwrap_or_default().to_string(),
        ParameterType::String,
    )
}

/// Implementation of ParameterStorage for the SSM ParameterStoreClient.
#[async_trait::async_trait]
impl ParameterStoreKind for si_data_ssm::ParameterStoreClient {
    async fn get_parameter(&self, name: String) -> ParameterStoreResult<Parameter> {
        let aws_param = self.get_parameter(name).await?;
        Ok(convert_aws_parameter(aws_param))
    }

    async fn parameters_by_path(&self, path: String) -> ParameterStoreResult<Vec<Parameter>> {
        let aws_params = self.parameters_by_path(path).await?;
        Ok(aws_params.into_iter().map(convert_aws_parameter).collect())
    }

    async fn create_string_parameter(
        &self,
        name: String,
        value: String,
    ) -> ParameterStoreResult<()> {
        self.create_string_parameter(name, value)
            .await
            .map_err(Into::into)
    }
}
