//! Cyclone instance pooling implementation.
//!

#![warn(
    missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::module_name_repetitions
)]

pub use self::instance::{Instance, Spec};
pub use crate::pool_noodle::pool_noodle::PoolNoodle;

pub use cyclone_client::{
    ClientError, CycloneClient, CycloneEncryptionKey, CycloneEncryptionKeyError, ExecutionError,
};

pub use cyclone_core::{
    ActionRunRequest, ActionRunResultSuccess, ComponentView, FunctionResult, FunctionResultFailure,
    FunctionResultFailureError, OutputStream, ProgressMessage, ReconciliationRequest,
    ReconciliationResultSuccess, ResolverFunctionRequest, ResolverFunctionResultSuccess,
    ResourceStatus, SchemaVariantDefinitionRequest, SchemaVariantDefinitionResultSuccess,
    ValidationRequest, ValidationResultSuccess,
};

/// [`Instance`] implementations.
pub mod instance;
/// [`PoolNoodle`] implementations.
pub mod pool_noodle;

#[cfg(test)]
mod tests {

    use cyclone_client::{LivenessStatus, ReadinessStatus};
    use tokio::sync::broadcast;

    use super::*;
    use crate::instance::cyclone::{
        LocalUdsInstance, LocalUdsRuntimeStrategy, LocalUdsSocketStrategy,
    };

    #[tokio::test]
    // #[ignore]
    async fn boom() {
        let mut config_file = veritech_server::ConfigFile::default_local_uds();
        veritech_server::detect_and_configure_development(&mut config_file)
            .expect("failed to determine test configuration");

        let spec = LocalUdsInstance::spec()
            .try_cyclone_cmd_path(config_file.cyclone.cyclone_cmd_path())
            .expect("failed to find cyclone program")
            .cyclone_decryption_key_path(config_file.cyclone.cyclone_decryption_key_path())
            .try_lang_server_cmd_path(config_file.cyclone.lang_server_cmd_path())
            .expect("failed to find lang server program")
            .limit_requests(2)
            .ping()
            .build()
            .expect("failed to build spec");

        let (shutdown_broadcast_tx, _) = broadcast::channel(16);
        let mut pool: PoolNoodle<LocalUdsInstance, instance::cyclone::LocalUdsInstanceSpec> =
            PoolNoodle::new(10, spec.clone(), shutdown_broadcast_tx.subscribe());
        pool.start();

        let mut instance = pool.get().await.expect("pool is empty!");

        instance
            .execute_ping()
            .await
            .expect("failed execute ping")
            .start()
            .await
            .expect("failed to start protocol");
        instance.ensure_healthy().await.expect("failed healthy");

        instance.terminate().await.expect("failed to terminate");
    }

    #[tokio::test]
    #[ignore]
    async fn pow() {
        let mut config_file = veritech_server::ConfigFile::default_local_uds();
        veritech_server::detect_and_configure_development(&mut config_file)
            .expect("failed to determine test configuration");

        let spec = LocalUdsInstance::spec()
            .try_cyclone_cmd_path(config_file.cyclone.cyclone_cmd_path())
            .expect("failed to find cyclone program")
            .cyclone_decryption_key_path(config_file.cyclone.cyclone_decryption_key_path())
            .try_lang_server_cmd_path(config_file.cyclone.lang_server_cmd_path())
            .expect("failed to find lang server program")
            .limit_requests(2)
            .runtime_strategy(LocalUdsRuntimeStrategy::LocalDocker)
            .socket_strategy(LocalUdsSocketStrategy::Random)
            .ping()
            .build()
            .expect("failed to build spec");

        let (shutdown_broadcast_tx, _) = broadcast::channel(16);
        let mut pool = PoolNoodle::new(10, spec.clone(), shutdown_broadcast_tx.subscribe());
        pool.start();

        let mut instance = pool.get().await.expect("pool is empty!");

        let status = instance
            .liveness()
            .await
            .expect("failed to run liveness check");
        assert_eq!(status, LivenessStatus::Ok);
        instance.ensure_healthy().await.expect("failed healthy");

        let status = instance
            .readiness()
            .await
            .expect("failed to run readiness check");
        assert_eq!(status, ReadinessStatus::Ready);
        instance.ensure_healthy().await.expect("failed healthy");

        instance
            .execute_ping()
            .await
            .expect("failed execute ping")
            .start()
            .await
            .expect("failed to start protocol");
        instance.ensure_healthy().await.expect("failed healthy");

        instance.terminate().await.expect("failed to terminate");
    }

    #[tokio::test]
    #[ignore]
    async fn chop() {
        let mut config_file = veritech_server::ConfigFile::default_local_uds();
        veritech_server::detect_and_configure_development(&mut config_file)
            .expect("failed to determine test configuration");

        let spec = LocalUdsInstance::spec()
            .try_cyclone_cmd_path(config_file.cyclone.cyclone_cmd_path())
            .expect("failed to find cyclone program")
            .cyclone_decryption_key_path(config_file.cyclone.cyclone_decryption_key_path())
            .try_lang_server_cmd_path(config_file.cyclone.lang_server_cmd_path())
            .expect("failed to find lang server program")
            .limit_requests(2)
            .runtime_strategy(LocalUdsRuntimeStrategy::LocalFirecracker)
            .socket_strategy(LocalUdsSocketStrategy::Random)
            .ping()
            .build()
            .expect("failed to build spec");

        let (shutdown_broadcast_tx, _) = broadcast::channel(16);
        let mut pool = PoolNoodle::new(10, spec.clone(), shutdown_broadcast_tx.subscribe());
        pool.start();
        let mut instance = pool.get().await.expect("should be able to get an instance");
        instance.ensure_healthy().await.expect("failed healthy");
    }
}
