// local_docker_instance_test.rs

use super::LocalDockerInstanceSpecBuilder;  // Import the necessary code
use cyclone_client::Client;  // Import any other dependencies you need
use your_module_name::LocalDockerInstanceSpec;  // Replace 'your_module_name' with the actual module name where your code resides

// Define your test function
#[tokio::test]
async fn test_local_docker_instance() {
    // Create a LocalDockerInstanceSpec using the builder
    let spec = LocalDockerInstanceSpecBuilder::default()
        .lang_server_cmd_path("/path/to/lang_server")
        .cyclone_decryption_key_path("/path/to/decryption_key.pem")
        .random_port(0)  // Set to 0 to use a random port
        .resolver()
        .build()
        .expect("Failed to build LocalDockerInstanceSpec");

    // Spawn the LocalDockerInstance
    let instance = spec.spawn().await.expect("Failed to spawn LocalDockerInstance");

    // Your test logic goes here
    // You can use 'instance' to interact with your LocalDockerInstance

    // Terminate the instance when done
    instance.terminate().await.expect("Failed to terminate LocalDockerInstance");
}
