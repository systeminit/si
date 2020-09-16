use anyhow::Result;
use futures::StreamExt;
use lazy_static::lazy_static;
use tracing;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{self, fmt, EnvFilter, Registry};

use std::env;

use si_data::Db;
use si_settings::Settings;

mod filters;
mod handlers;
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
}

pub async fn test_setup() -> Result<TestAccount> {
    let reply = crate::filters::billing_accounts::signup().await;

    Ok(TestAccount {
        user_id: reply.user.id,
        billing_account_id: reply.billing_account.id,
        workspace_id: reply.workspace.id,
        organization_id: reply.organization.id,
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
