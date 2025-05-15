use bedrock_core::{Parameters, TestProfile, ExecutionParameters, TestResult, TestProfileResponse};

mod rebaser;
use rebaser::measure_rebase::MeasureRebase;
use si_data_nats::NatsClient;

pub fn load_profiles() -> TestProfileResponse {
    let instances: Vec<Box<dyn TestProfile>> = vec![
        Box::new(MeasureRebase),
        // Add other test types here
    ];

    TestProfileResponse {
        success: true,
        profiles: instances.iter().map(|p| p.get()).collect(),
    }
}

pub async fn run_test(
    service: &str,
    test: &str,
    parameters: &Parameters,
    exec: &ExecutionParameters,
    nats: &NatsClient,
) -> Option<TestResult> {
    match (service, test) {
        ("rebaser", "measure_rebase") => Some(MeasureRebase.run(parameters, exec, nats).await),
        _ => None,
    }
}