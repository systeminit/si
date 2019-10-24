use si_settings::Settings;
use sodiumoxide;
use toml;

use si_ssh_key::ssh_key::{client::SshKeyClient, GetComponentRequest, ListComponentsRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _settings = Settings::new()?;

    let key = sodiumoxide::crypto::secretbox::gen_key();
    println!("SECRET KEY");
    println!("{}", toml::to_string(&key).unwrap());

    let mut client = SshKeyClient::connect("http://[::1]:50051").await?;
    //let mut client = SshKeyClient::connect("http://[::1]:5151")?;

    let request = tonic::Request::new(GetComponentRequest {
        component_id: "component:sshkey:ae14cd21-4667-442a-be8e-7ed3180b32bc".into(),
    });

    let response = client.get_component(request).await?;
    println!("GET COMPONENT");
    println!("RESPONSE={:#?}", response);

    let request = tonic::Request::new(ListComponentsRequest {
        page_size: 20,
        order_by: "bits".into(),
        ..Default::default()
    });

    println!("LIST COMPONENTS");
    let response = client.list_components(request).await?;
    println!("LIST_RESPONSE:{:#?}", response);

    println!("LIST COMPONENTS NEXT PAGE");
    let request = tonic::Request::new(ListComponentsRequest {
        page_token: response.into_inner().next_page_token,
        ..Default::default()
    });
    let response = client.list_components(request).await?;
    println!("LIST_RESPONSE:{:#?}", response);

    Ok(())
}
