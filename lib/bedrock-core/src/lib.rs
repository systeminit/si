use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use async_trait::async_trait;
use si_data_nats::NatsClient;


/*

Example: 
{
  "postgres": ["si_layer_db", "si"],
  "nats": ["REBASER_REQUESTS", "STREAM_2"],
  "metadata": {
    "messages": 1000,
    "timeout": 300
  }
}

*/
#[derive(Debug, Deserialize)]
pub struct RecordRequest {
    pub recording_id: Option<String>,
    pub postgres: Option<Vec<String>>,
    pub nats: Option<Vec<String>>,
    pub metadata: Option<RecordMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct PrepareRequest {
    pub recording_id: String,
    pub metadata: Option<RecordMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PrepareResult {
    pub success: bool,
    pub message: String,
    pub recording_id: String,
    pub duration_ms: Option<u64>,
    pub output: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct RecordMetadata {
    pub messages: u64,
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecordResult {
    pub success: bool,
    pub message: String,
    pub recording_id: String,
    pub duration_ms: Option<u64>,
    pub output: Option<Value>,
}


#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub artifact_id: String,
    pub metadata: serde_json::Value,
}
#[derive(Debug, Serialize)]
pub struct PublishResult {
    pub success: bool,
    pub message: String,
    pub duration_ms: Option<u64>,
    pub output: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ArtifactStoreConfig {
    pub variant: String,
    pub metadata: serde_json::Value,
}

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
    pub recording_id: String,
    pub parameters: Parameters,
    pub execution_parameters: ExecutionParameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {
    pub workspace_id: String,
    pub change_set_id: String,
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
        recording_id: &String,
        parameters: &Parameters,
        exec: &ExecutionParameters,
        nats: &NatsClient,
    ) -> TestResult;

    fn get(&self) -> Profile;
}

