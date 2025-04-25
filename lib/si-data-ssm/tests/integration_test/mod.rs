#[cfg(test)]
mod integration_tests {
    use std::env;

    use si_data_ssm::ParameterStoreClient;

    const ENV_VAR_LOCALSTACK_URL: &str = "SI_TEST_LOCALSTACK_URL";
    #[tokio::test]
    async fn test_create_get_and_get_by_path() {
        let mut endpoint = "http://localhost:4566".to_string();
        #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
        if let Ok(value) = env::var(ENV_VAR_LOCALSTACK_URL) {
            endpoint = value;
        }
        dbg!("connecting to {}", &endpoint);
        let client = ParameterStoreClient::new_for_test(endpoint);

        let parameter_name = "/test/integration/parameter";
        let parameter_value = "test_value";
        let parameter_path = "/test/integration";

        let create_result = client
            .create_string_parameter(parameter_name.to_string(), parameter_value.to_string())
            .await;

        assert!(
            create_result.is_ok(),
            "Failed to create parameter: {:?}",
            create_result.err()
        );

        let get_result = client.get_parameter(parameter_name.to_string()).await;
        assert!(
            get_result.is_ok(),
            "Failed to get parameter: {:?}",
            get_result.err()
        );

        let parameter = get_result.expect("should get parameter");
        assert_eq!(
            parameter.name().expect("parameter should have a name"),
            parameter_name,
            "Unexpected parameter name"
        );
        assert_eq!(
            parameter.value().expect("parameter should have a value"),
            parameter_value,
            "Unexpected parameter value"
        );

        let get_by_path_result = client.parameters_by_path(parameter_path.to_string()).await;
        assert!(
            get_by_path_result.is_ok(),
            "Failed to get parameters by path: {:?}",
            get_by_path_result.err()
        );

        let parameters = get_by_path_result.expect("should get parameters by path");
        assert_eq!(
            parameters.len(),
            1,
            "Unexpected number of parameters under the path"
        );

        let path_parameter = &parameters[0];
        assert_eq!(
            path_parameter
                .name()
                .expect("path paramter should have a name"),
            parameter_name,
            "Unexpected parameter name in path"
        );
        assert_eq!(
            path_parameter
                .value()
                .expect("path parameter should have a value"),
            parameter_value,
            "Unexpected parameter value in path"
        );
    }
}
