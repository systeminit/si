use anyhow::Result;
use futures::StreamExt;
use lazy_static::lazy_static;
use nats::asynk::Connection;
use tracing;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{self, fmt, EnvFilter, Registry};

use std::env;
use std::sync::Arc;

use si_sdf::data::Db;
use si_sdf::models::{BillingAccount, User};
use si_settings::Settings;

mod filters;
mod handlers;
mod models;
mod use_case;

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
    pub static ref NATS: Connection = {
        let nats = futures::executor::block_on(nats::asynk::connect("localhost"))
            .expect("failed to connect to nats");
        nats
    };
    pub static ref INIT_LOCK: Arc<tokio::sync::Mutex<bool>> =
        Arc::new(tokio::sync::Mutex::new(false));
}

pub struct TestAccount {
    pub user_id: String,
    pub billing_account_id: String,
    pub workspace_id: String,
    pub organization_id: String,
    pub user: User,
    pub billing_account: BillingAccount,
    pub authorization: String,
    pub system_ids: Option<Vec<String>>,
}

pub async fn one_time_setup() -> Result<()> {
    let mut finished = INIT_LOCK.lock().await;
    if *finished {
        return Ok(());
    }
    match si_sdf::data::create_indexes(&DB).await {
        Ok(_) => (),
        Err(err) => println!("failed to create indexes: {}", err),
    }
    match si_sdf::data::delete_data(&DB).await {
        Ok(_) => (),
        Err(err) => println!("failed to delete data: {}", err),
    }
    *finished = true;
    return Ok(());
}

pub async fn test_setup() -> Result<TestAccount> {
    one_time_setup().await?;
    si_sdf::models::jwt_key::create_if_missing(
        &DB,
        &NATS,
        "config/public.pem",
        "config/private.pem",
        &SETTINGS.jwt_encrypt.key,
    )
    .await?;
    let reply = crate::filters::billing_accounts::signup().await;
    let jwt =
        crate::filters::users::login_user(&reply.billing_account.name, &reply.user.email).await;
    let mut test_account = TestAccount {
        user_id: reply.user.id.clone(),
        billing_account_id: reply.billing_account.id.clone(),
        workspace_id: reply.workspace.id,
        organization_id: reply.organization.id,
        billing_account: reply.billing_account,
        user: reply.user,
        authorization: format!("Bearer {}", jwt),
        system_ids: None,
    };

    let system_ids = crate::filters::nodes::create_system(&test_account).await;
    test_account.system_ids = Some(system_ids);

    Ok(test_account)
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
