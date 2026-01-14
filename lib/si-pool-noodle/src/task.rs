use std::{
    fmt::Display,
    result,
    time::Instant,
};

use telemetry_utils::metric;
use tracing::{
    debug,
    info,
};

use crate::{
    Instance,
    Spec,
    errors::PoolNoodleError,
};

type Result<T, E> = result::Result<T, PoolNoodleError<E>>;

#[derive(Clone, Debug)]
pub(crate) enum PoolNoodleTaskType<I, S> {
    Clean(PoolNoodleTask<I, S>),
    Drop(PoolNoodleTask<I, S>),
    Prepare(PoolNoodleTask<I, S>),
}

#[derive(Clone, Debug)]
pub(crate) struct PoolNoodleTask<I, S> {
    instance: Option<I>,
    id: u32,
    spec: S,
}

impl<I, E, S> PoolNoodleTask<I, S>
where
    I: Instance<Error = E> + Send + Sync + 'static,
    S: Spec<Error = E, Instance = I> + Clone + Send + Sync + 'static,
    E: Send + Display,
{
    pub fn new(instance: Option<I>, id: u32, spec: S) -> Self {
        Self { instance, id, spec }
    }

    pub fn set_instance(&mut self, instance: Option<I>) {
        self.instance = instance;
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub async fn clean(&self) -> Result<(), E> {
        let start = Instant::now();
        let result = self
            .spec
            .clean(self.id)
            .await
            .map_err(|err| PoolNoodleError::InstanceClean(err));
        let duration_ms = start.elapsed().as_millis() as u64;
        debug!(
            id = self.id,
            duration_ms = duration_ms,
            success = result.is_ok(),
            "pool_noodle task: clean completed"
        );
        metric!(histogram.pool_noodle.task.clean_duration_ms = duration_ms);
        result
    }

    pub async fn prepare(&self) -> Result<(), E> {
        let start = Instant::now();
        let result = self
            .spec
            .prepare(self.id)
            .await
            .map_err(|err| PoolNoodleError::InstancePrepare(err));
        let duration_ms = start.elapsed().as_millis() as u64;
        debug!(
            id = self.id,
            duration_ms = duration_ms,
            success = result.is_ok(),
            "pool_noodle task: prepare completed"
        );
        metric!(histogram.pool_noodle.task.prepare_duration_ms = duration_ms);
        result
    }

    pub async fn spawn(&self) -> Result<I, E> {
        let start = Instant::now();
        let result = self
            .spec
            .spawn(self.id)
            .await
            .map_err(|err| PoolNoodleError::InstanceSpawn(err));
        let duration_ms = start.elapsed().as_millis() as u64;
        debug!(
            id = self.id,
            duration_ms = duration_ms,
            success = result.is_ok(),
            "pool_noodle task: spawn completed"
        );
        metric!(histogram.pool_noodle.task.spawn_duration_ms = duration_ms);
        result
    }

    pub async fn terminate(self) -> Result<(), E> {
        let start = Instant::now();
        if let Some(mut instance) = self.instance {
            let result = instance
                .terminate()
                .await
                .map_err(|err| PoolNoodleError::InstanceTerminate(err));
            let duration_ms = start.elapsed().as_millis() as u64;
            debug!(
                id = self.id,
                duration_ms = duration_ms,
                success = result.is_ok(),
                "pool_noodle task: terminate completed"
            );
            metric!(histogram.pool_noodle.task.terminate_duration_ms = duration_ms);
            return result;
        }
        Err(PoolNoodleError::InstanceNotFound)
    }
}
