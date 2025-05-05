use innit_client::InnitClient;

use super::AdminAPIResult;

const SERVICE_NAME: &str = "sdf";

pub async fn clear_parameter_cache() -> AdminAPIResult<()> {
    let client = InnitClient::new_from_environment(SERVICE_NAME.to_string()).await?;
    client.clear_parameter_cache().await?;
    Ok(())
}
