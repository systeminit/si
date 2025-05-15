use serde::{
    Deserialize,
    Serialize,
};
use async_trait::async_trait;
use si_data_nats::NatsClient;

// This is an example response list of test profiles activated:
/* 
{
  "profiles": [
     {
       "service": "rebaser",
       "test": "measure_rebase",
       "parameters": [
           "variant": "linear"
       ],
       "executionParameters": [
          "iterations": 5,
          "timeout": 60
       ]
     }
  ]
} 
*/

#[derive(Debug, Serialize, Deserialize)]
pub struct TestProfileResponse {
    pub success: bool,
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub service: String,
    pub test: String,
    pub parameters: Parameters,
    pub executionParameters: ExecutionParameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {
    pub variant: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionParameters {
    pub iterations: u32,
    pub timeout: u32,
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
    pub duration_ms: Option<u64>,
    pub output: Option<serde_json::Value>,
}

#[async_trait]
pub trait TestProfile: Send + Sync {
    async fn run(
        &self,
        parameters: &Parameters,
        exec: &ExecutionParameters,
        nats: &NatsClient,
    ) -> TestResult;

    fn get(&self) -> Profile;
}

