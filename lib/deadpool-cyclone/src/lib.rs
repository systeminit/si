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

use deadpool::managed::{self, Metrics};
use tokio::sync::Mutex;

use async_trait::async_trait;
use pool_noodle::pool_noodle::PoolNoodleError;
use thiserror::Error;

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

/// Type alias for using [`managed::Pool`] with Cyclone.
pub type Pool<S> = managed::Pool<Manager<S>>;
/// Type alias for using [`managed::PoolBuilder`] with Cyclone.
pub type PoolBuilder<S> = managed::PoolBuilder<Manager<S>, Connection<S>>;
/// Type alias for using [`managed::PoolError`] with Cyclone.
pub type PoolError<E> = managed::PoolError<ManagerError<E>>;
/// Type alias for using [`managed::Object`] with Cyclone.
pub type Object<S> = managed::Object<Manager<S>>;
/// Type alias for using [`managed::Hook`] with Cyclone.
pub type Hook<S> = managed::Hook<Manager<S>>;
/// Type alias for using [`managed::HookError`] with Cyclone.
pub type HookError<S> = managed::HookError<Manager<S>>;

/// Type alias for using [`managed::HookErrorCause`] with Cyclone.
pub type Connection<S> = managed::Object<Manager<S>>;

/// Type alias for using [`managed::RecycleResult`] with Cyclone.
pub type RecycleResult<T> = managed::RecycleResult<ManagerError<T>>;

/// Error type for [`Manager<S>`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ManagerError<T> {
    /// An Instance error.
    #[error("instance error")]
    Instance(#[source] T),
    /// A Setup Error.
    #[error("setup error")]
    SetupError(#[source] T),
}

/// [`Manager`] for creating and recycling generic [`Instance`]s.
pub struct Manager<S> {
    pool_noodle: Mutex<PoolNoodle>,
    spec: S,
}

impl<S> Manager<S>
where
    S: Spec,
{
    /// Creates a new [`Manager`] from the given instance specification.
    pub fn new(spec: S) -> Self {
        Self {
            pool_noodle: PoolNoodle::new(spec.pool_size().into()).into(),
            spec,
        }
    }

    /// Peforms any necessary setup work to ensure the host can run the pool members.
    pub async fn setup(&mut self) -> Result<(), S::Error> {
        self.spec.setup().await?;
        if self.spec.use_pool_noodle() {
            self.pool_noodle.lock().await.start();
        }
        Ok(())
    }
}

#[async_trait]
impl<B, S, I, E> managed::Manager for Manager<S>
where
    S: Spec<Error = E, Instance = I> + Send + Sync,
    I: Instance<SpecBuilder = B, Error = E> + Send,
    E: Send,
{
    type Type = I;
    type Error = E;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let id = if self.spec.use_pool_noodle() {
            self
            .pool_noodle
            .lock()
            .await
            .get_ready_jail()
            .await
            .map_err(|_| PoolNoodleError::ExecutionPoolStarved)
            .expect("Function execution is impossible as the execution pool is starved and not recovering.")
        } else {
            0
        };
        self.spec.spawn(id).await
    }

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        _: &Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        match obj.ensure_healthy().await {
            Ok(_) => Ok(()),
            Err(err) => {
                if self.spec.use_pool_noodle() {
                    self.pool_noodle
                        .lock()
                        .await
                        .set_as_to_be_cleaned(obj.id())
                        .await;
                }
                Result::map_err(Err(err), Into::into)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use cyclone_client::{LivenessStatus, ReadinessStatus};

    use super::*;
    use crate::instance::cyclone::{
        LocalUdsInstance, LocalUdsRuntimeStrategy, LocalUdsSocketStrategy,
    };

    #[tokio::test]
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
        let manager = Manager::new(spec);

        let mut instance = managed::Manager::create(&manager)
            .await
            .expect("failed to create instance");

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
        let manager = Manager::new(spec);

        let mut instance = managed::Manager::create(&manager)
            .await
            .expect("failed to create instance");

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
        let manager = Manager::new(spec);

        let mut instance = managed::Manager::create(&manager)
            .await
            .expect("failed to create instance");

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
}
