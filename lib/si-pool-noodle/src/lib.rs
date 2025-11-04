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

pub use cyclone_client::{
    ClientError,
    CycloneClient,
    ExecutionError,
};
pub use cyclone_core::{
    ActionRunRequest,
    ActionRunResultSuccess,
    BeforeFunction,
    ComponentView,
    CycloneRequest,
    CycloneRequestable,
    DebugRequest,
    DebugResultSuccess,
    FunctionResult,
    FunctionResultFailure,
    FunctionResultFailureError,
    FunctionResultFailureErrorKind,
    KillExecutionRequest,
    ManagementRequest,
    ManagementResultSuccess,
    OutputStream,
    ProgressMessage,
    ResolverFunctionRequest,
    ResolverFunctionResultSuccess,
    ResourceStatus,
    SchemaVariantDefinitionRequest,
    SchemaVariantDefinitionResultSuccess,
    SensitiveStrings,
    ValidationRequest,
    ValidationResultSuccess,
};

pub use self::instance::{
    Instance,
    Spec,
};
pub use crate::pool_noodle::PoolNoodle;

/// [`PoolNoodleError`] implementations.
pub mod errors;
/// [`Instance`] implementations.
pub mod instance;
mod lifeguard;
/// [`PoolNoodle`] implementations.
pub mod pool_noodle;
mod task;

#[cfg(test)]
mod tests {

    use cyclone_client::{
        LivenessStatus,
        ReadinessStatus,
    };
    use tokio_util::sync::CancellationToken;

    use super::*;
    use crate::{
        instance::cyclone::{
            LocalUdsInstance,
            LocalUdsRuntimeStrategy,
            LocalUdsSocketStrategy,
        },
        pool_noodle::PoolNoodleConfig,
    };

    #[tokio::test]
    async fn boom() {
        // TODO(nick,fletcher): use the cancellation token in the test.
        let shutdown_token = CancellationToken::new();

        let mut config_file = veritech_server::ConfigFile::default_local_uds();
        veritech_server::detect_and_configure_development(&mut config_file)
            .expect("failed to determine test configuration");

        let spec = LocalUdsInstance::spec()
            .try_cyclone_cmd_path(config_file.cyclone.cyclone_cmd_path())
            .expect("failed to find cyclone program")
            .try_lang_server_cmd_path(config_file.cyclone.lang_server_cmd_path())
            .expect("failed to find lang server program")
            .limit_requests(2)
            .ping()
            .build()
            .expect("failed to build spec");

        let mut pool: PoolNoodle<LocalUdsInstance, instance::cyclone::LocalUdsInstanceSpec> =
            PoolNoodle::new(PoolNoodleConfig {
                shutdown_token,
                spec: spec.clone(),
                ..Default::default()
            })
            .await;
        pool.run().expect("failed to start");

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
        // TODO(nick,fletcher): use the cancellation token in the test.
        let shutdown_token = CancellationToken::new();

        let mut config_file = veritech_server::ConfigFile::default_local_uds();
        veritech_server::detect_and_configure_development(&mut config_file)
            .expect("failed to determine test configuration");

        let spec = LocalUdsInstance::spec()
            .try_cyclone_cmd_path(config_file.cyclone.cyclone_cmd_path())
            .expect("failed to find cyclone program")
            .try_lang_server_cmd_path(config_file.cyclone.lang_server_cmd_path())
            .expect("failed to find lang server program")
            .limit_requests(2)
            .runtime_strategy(LocalUdsRuntimeStrategy::LocalDocker)
            .socket_strategy(LocalUdsSocketStrategy::Random)
            .ping()
            .build()
            .expect("failed to build spec");

        let mut pool: PoolNoodle<LocalUdsInstance, instance::cyclone::LocalUdsInstanceSpec> =
            PoolNoodle::new(PoolNoodleConfig {
                shutdown_token,
                spec: spec.clone(),
                ..Default::default()
            })
            .await;
        pool.run().expect("failed to start");
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
    #[cfg(target_os = "linux")]
    async fn chop() {
        // TODO(nick,fletcher): use the cancellation token in the test.
        let shutdown_token = CancellationToken::new();

        let mut config_file = veritech_server::ConfigFile::default_local_uds();
        veritech_server::detect_and_configure_development(&mut config_file)
            .expect("failed to determine test configuration");

        let spec = LocalUdsInstance::spec()
            .try_cyclone_cmd_path(config_file.cyclone.cyclone_cmd_path())
            .expect("failed to find cyclone program")
            .try_lang_server_cmd_path(config_file.cyclone.lang_server_cmd_path())
            .expect("failed to find lang server program")
            .limit_requests(2)
            .runtime_strategy(LocalUdsRuntimeStrategy::LocalFirecracker)
            .socket_strategy(LocalUdsSocketStrategy::Random)
            .ping()
            .build()
            .expect("failed to build spec");

        let mut pool: PoolNoodle<LocalUdsInstance, instance::cyclone::LocalUdsInstanceSpec> =
            PoolNoodle::new(PoolNoodleConfig {
                shutdown_token,
                spec: spec.clone(),
                ..Default::default()
            })
            .await;
        pool.run().expect("failed to start");
        let mut instance = pool.get().await.expect("should be able to get an instance");
        instance.ensure_healthy().await.expect("failed healthy");
    }
}
