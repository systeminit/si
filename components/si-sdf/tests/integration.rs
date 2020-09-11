use anyhow::Result;
use futures::StreamExt;
use lazy_static::lazy_static;
use tracing;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{self, fmt, EnvFilter, Registry};

use std::env;

use si_account::{gen::service::Service, protobuf, protobuf::account_server::Account};
use si_data::Db;
use si_settings::Settings;

mod filters;
mod models;

pub struct TestAccount {
    pub user_id: String,
    pub billing_account_id: String,
    pub workspace_id: String,
    pub organization_id: String,
}

lazy_static! {
    pub static ref SETTINGS: Settings = {
        let fmt_layer = fmt::Layer::default();
        let env_filter_layer = EnvFilter::from_default_env();

        let subscriber = Registry::default().with(env_filter_layer).with(fmt_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("tracing global default should be set");

        env::set_var("RUN_ENV", "testing");

        Settings::new().expect("settings should load")
    };
    pub static ref DB: Db = {
        let db = Db::new(&SETTINGS).expect("cannot connect to database");
        db
    };
    pub static ref ACCOUNT_SERVICE: Service = {
        let service = Service::new(DB.clone());
        service
    };
}

pub async fn test_setup() -> Result<TestAccount> {
    let fake_name = si_sdf::models::generate_id("fakename");
    ACCOUNT_SERVICE.migrate().await?;
    let response = ACCOUNT_SERVICE
        .billing_account_signup(tonic::Request::new(protobuf::BillingAccountSignupRequest {
            billing_account: Some(protobuf::BillingAccountSignupRequestBillingAccount {
                name: Some(fake_name),
                display_name: Some("Bobo Corp".to_string()),
            }),
            user: Some(protobuf::BillingAccountSignupRequestUser {
                name: Some("Bobo T. Clown".to_string()),
                display_name: Some("Bobo T. Clown".to_string()),
                email: Some("bobo@tclown.com".to_string()),
                password: Some("bobor0cks".to_string()),
            }),
        }))
        .await
        .expect("failed to signup with the billing account");
    let signup_response = response.into_inner();
    let user_id = signup_response
        .user
        .expect("no user in response")
        .id
        .expect("no user id in response");
    let billing_account_id = signup_response.billing_account.unwrap().id.unwrap();
    let workspace_id = signup_response.workspace.unwrap().id.unwrap();
    let organization_id = signup_response.organization.unwrap().id.unwrap();

    Ok(TestAccount {
        user_id,
        billing_account_id,
        workspace_id,
        organization_id,
    })
}

pub async fn test_cleanup(test_account: TestAccount) -> Result<()> {
    let query = format!(
        "DELETE FROM si_integration AS s
        WHERE ANY t IN s.siStorable.tenantIds 
          SATISFIES t = \"{}\" END 
        RETURNING s",
        test_account.billing_account_id
    );
    let mut result = DB
        .cluster
        .query(query, None)
        .await
        .expect("could not delete the data for this billing account");
    let mut result_stream = result.rows_as::<serde_json::Value>()?;
    while let Some(r) = result_stream.next().await {
        match r {
            Ok(_) => (),
            Err(e) => return Err(anyhow::Error::from(e)),
        }
    }

    Ok(())
}
