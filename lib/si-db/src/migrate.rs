#![warn(
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used
)]

use std::time::Duration;

use serde_with::{
    DeserializeFromStr,
    SerializeDisplay,
};
use si_data_pg::PgPool;
use strum::{
    Display,
    EnumString,
    VariantNames,
};
use telemetry::prelude::*;
use tokio::{
    time,
    time::Instant,
};

use crate::Result;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}

#[instrument(level = "info", skip_all)]
pub async fn migrate_all(pg_pool: &PgPool) -> Result<()> {
    migrate(pg_pool).await?;
    Ok(())
}

#[instrument(level = "info", skip_all)]
pub async fn migrate_all_with_progress(pg_pool: &PgPool) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(5));
    let instant = Instant::now();
    let migrate_all = migrate_all(pg_pool);
    tokio::pin!(migrate_all);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                info!(elapsed = instant.elapsed().as_secs_f32(), "migrating");
            }
            result = &mut migrate_all  => match result {
                Ok(_) => {
                    info!(elapsed = instant.elapsed().as_secs_f32(), "migrating completed");
                    break;
                }
                Err(err) => return Err(err),
            }
        }
    }

    Ok(())
}

#[instrument(level = "info", skip_all)]
pub async fn migrate(pg: &PgPool) -> Result<()> {
    pg.migrate(embedded::migrations::runner()).await?;
    Ok(())
}

#[remain::sorted]
#[derive(
    Clone,
    Debug,
    DeserializeFromStr,
    Display,
    EnumString,
    VariantNames,
    Eq,
    PartialEq,
    SerializeDisplay,
)]
#[strum(serialize_all = "camelCase")]
pub enum MigrationMode {
    BackfillLayerCache,
    GarbageCollectSnapshots,
    Run,
    RunAndQuit,
    Skip,
}

impl Default for MigrationMode {
    fn default() -> Self {
        Self::Run
    }
}

impl MigrationMode {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <MigrationMode as strum::VariantNames>::VARIANTS
    }

    pub fn is_run(&self) -> bool {
        matches!(self, Self::Run)
    }

    pub fn is_run_and_quit(&self) -> bool {
        matches!(self, Self::RunAndQuit)
    }

    pub fn is_garbage_collect_snapshots(&self) -> bool {
        matches!(self, Self::GarbageCollectSnapshots)
    }

    pub fn is_backfill_layer_cache(&self) -> bool {
        matches!(self, Self::BackfillLayerCache)
    }
}

#[cfg(test)]
mod tests {
    use serde::{
        Deserialize,
        Serialize,
    };

    use super::*;

    mod migration_mode {
        use super::*;

        #[test]
        fn display() {
            assert_eq!(
                "garbageCollectSnapshots",
                MigrationMode::GarbageCollectSnapshots.to_string()
            );
            assert_eq!("run", MigrationMode::Run.to_string());
            assert_eq!("runAndQuit", MigrationMode::RunAndQuit.to_string());
            assert_eq!("skip", MigrationMode::Skip.to_string());
        }

        #[test]
        fn from_str() {
            assert_eq!(
                MigrationMode::GarbageCollectSnapshots,
                "garbageCollectSnapshots".parse().expect("failed to parse")
            );
            assert_eq!(MigrationMode::Run, "run".parse().expect("failed to parse"));
            assert_eq!(
                MigrationMode::RunAndQuit,
                "runAndQuit".parse().expect("failed to parse")
            );
            assert_eq!(
                MigrationMode::Skip,
                "skip".parse().expect("failed to parse")
            );
        }

        #[test]
        fn deserialize() {
            #[derive(Deserialize)]
            struct Test {
                mode: MigrationMode,
            }

            let test: Test =
                serde_json::from_str(r#"{"mode":"runAndQuit"}"#).expect("failed to deserialize");
            assert_eq!(MigrationMode::RunAndQuit, test.mode);
        }

        #[test]
        fn serialize() {
            #[derive(Serialize)]
            struct Test {
                mode: MigrationMode,
            }

            let test = serde_json::to_string(&Test {
                mode: MigrationMode::RunAndQuit,
            })
            .expect("failed to serialize");
            assert_eq!(r#"{"mode":"runAndQuit"}"#, test);
        }
    }
}
