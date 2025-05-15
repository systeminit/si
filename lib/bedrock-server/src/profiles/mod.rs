use bedrock_core::{
    ExecutionParameters,
    Parameters,
    TestProfile,
    TestProfileResponse,
    TestResult,
};

mod rebaser;
use rebaser::measure_rebase::MeasureRebase;
use si_data_nats::NatsClient;

pub fn load_profiles() -> TestProfileResponse {
    let profiles: Vec<Box<dyn TestProfile>> = vec![
        Box::new(MeasureRebase),
        // Add other test types here
    ];

    TestProfileResponse {
        success: true,
        profiles: profiles.iter().map(|p| p.get()).collect(),
    }
}

pub async fn run_test(
    recording_id: &String,
    parameters: &Parameters,
    exec: &ExecutionParameters,
    nats: &NatsClient,
) -> Option<TestResult> {
    match recording_id.as_str() {
        "some_specific_inbuilt_test" => None,
        _ => Some(MeasureRebase.run(recording_id, parameters, exec, nats).await),
    }
}
