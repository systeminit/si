use anyhow::Result;
use lazy_static::lazy_static;
use sodiumoxide::crypto::secretbox;
use tracing;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{self, fmt, EnvFilter, Registry};

use std::env;
use std::sync::Arc;

use si_sdf::data::{EventLogFS, NatsConn, PgPool};
use si_sdf::models::{BillingAccount, User};
use si_sdf::veritech::Veritech;
use si_settings::Settings;

mod bugs;
mod data;
mod filters;
mod handlers;
mod models;
//mod use_case;

lazy_static! {
    pub static ref SETTINGS: Settings = {
        let fmt_layer = fmt::Layer::default();
        let env_filter_layer = EnvFilter::from_default_env();

        let subscriber = Registry::default().with(env_filter_layer).with(fmt_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("tracing global default should be set");

        env::set_var("RUN_ENV", "testing");

        let settings = Settings::new().expect("settings should load");
        unsafe {
            si_sdf::PAGE_SECRET_KEY = Some(settings.paging.key.clone());
        }
        settings
    };
    pub static ref INIT_LOCK: Arc<tokio::sync::Mutex<bool>> =
        Arc::new(tokio::sync::Mutex::new(false));
    pub static ref INIT_PG_LOCK: Arc<tokio::sync::Mutex<bool>> =
        Arc::new(tokio::sync::Mutex::new(false));
}

pub struct TestContext {
    // we need to keep this in scope to keep the tempdir from auto-cleaning itself
    #[allow(dead_code)]
    tmp_event_log_fs_root: tempfile::TempDir,
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    event_log_fs: EventLogFS,
    secret_key: secretbox::Key,
}

impl TestContext {
    pub async fn init() -> Self {
        Self::init_with_settings(&SETTINGS).await
    }

    pub async fn init_with_settings(settings: &Settings) -> Self {
        let tmp_event_log_fs_root = tempfile::tempdir().expect("could not create temp dir");
        let pg = PgPool::new(&settings.pg)
            .await
            .expect("failed to connect to postgres");
        let nats_conn = NatsConn::new(&settings.nats)
            .await
            .expect("failed to connect to NATS");
        let elf_settings = si_settings::EventLogFs {
            root: tmp_event_log_fs_root.as_ref().into(),
        };
        let event_log_fs = EventLogFS::init(&elf_settings)
            .await
            .expect("failed to initialize EventLogFS");
        let veritech = Veritech::new(&settings.veritech, event_log_fs.clone());
        let secret_key = settings.jwt_encrypt.key.clone();

        Self {
            tmp_event_log_fs_root,
            pg,
            nats_conn,
            veritech,
            event_log_fs,
            secret_key,
        }
    }

    pub fn entries(&self) -> (&PgPool, &NatsConn, &Veritech, &EventLogFS, &secretbox::Key) {
        (
            &self.pg,
            &self.nats_conn,
            &self.veritech,
            &self.event_log_fs,
            &self.secret_key,
        )
    }
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
    pub pg: PgPool,
}

pub async fn one_time_setup() -> Result<()> {
    let mut finished = INIT_PG_LOCK.lock().await;
    if *finished {
        return Ok(());
    }

    sodiumoxide::init().expect("crypto failed to init");

    let pg = PgPool::new(&SETTINGS.pg)
        .await
        .expect("failed to connect to postgres");
    pg.drop_and_create_public_schema()
        .await
        .expect("failed to drop the database");
    pg.migrate().await.expect("migration failed!");

    si_sdf::models::update_clock::init_update_clock_service(&SETTINGS);

    let mut conn = pg.pool.try_get().await?;
    let txn = conn.transaction().await?;

    si_sdf::models::create_jwt_key_if_missing(
        &txn,
        "config/public.pem",
        "config/private.pem",
        &SETTINGS.jwt_encrypt.key,
    )
    .await?;
    txn.commit().await?;

    *finished = true;
    Ok(())
}
