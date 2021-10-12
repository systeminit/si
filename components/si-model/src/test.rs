use anyhow::Result;
use lazy_static::lazy_static;
use names::{Generator, Name};
use si_data::{EventLogFS, NatsConn, PgPool};
use crate::{Veritech, Workflow};
use si_settings::Settings;
use sodiumoxide::crypto::secretbox;
use std::env;
use std::sync::Arc;

pub mod model;
pub use model::billing_account::*;
pub use model::change_set::*;
pub use model::edge::*;
pub use model::edit_session::*;
pub use model::entity::*;
pub use model::event::*;
pub use model::event_log::*;
pub use model::group::*;
pub use model::key_pair::*;
pub use model::node::*;
pub use model::node_position::*;
pub use model::organization::*;
pub use model::schema::*;
pub use model::secret::*;
pub use model::user::*;
pub use model::workspace::*;

lazy_static! {
    pub static ref SETTINGS: Settings = {
        env::set_var("RUN_ENV", "testing");

        let settings = Settings::new().expect("settings should load");
        unsafe {
            crate::PAGE_SECRET_KEY = Some(settings.paging.key.clone());
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
        let veritech = Veritech::new(&settings.veritech, event_log_fs.clone())
            .await
            .expect("failed to create veritech client");
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
    crate::migrate(&pg).await.expect("migration failed!");

    let mut conn = pg.get().await?;
    let txn = conn.transaction().await?;

    let tmp_event_log_fs_root = tempfile::tempdir().expect("could not create temp dir");
    let elf_settings = si_settings::EventLogFs {
        root: tmp_event_log_fs_root.as_ref().into(),
    };
    let event_log_fs = EventLogFS::init(&elf_settings)
        .await
        .expect("failed to initialize EventLogFS");
    let veritech = Veritech::new(&SETTINGS.veritech, event_log_fs.clone())
        .await
        .expect("failed to create veritech client");

    Workflow::load_builtins(&pg, &veritech).await?;

    crate::create_jwt_key_if_missing(
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

pub fn generate_fake_name() -> String {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    return name;
}
