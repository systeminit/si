use std::fmt::Display;

use telemetry_utils::metric;
use tokio::sync::mpsc::Sender;
use tracing::{
    debug,
    info,
    warn,
};

use crate::{
    Instance,
    Spec,
    task::{
        PoolNoodleTask,
        PoolNoodleTaskType,
    },
};

/// LifeGuard is a wrapper for instances that come from the pool.
/// It is carries a Sender and implements Drop. When an instance goes out of
/// scope, it lets PoolNoodle know that the instance needs to be cleaned up.
#[derive(Debug)]
pub struct LifeGuard<I, E, S>
where
    I: Instance<Error = E> + Send + Sync + 'static,
    S: Spec<Error = E, Instance = I> + Clone + Send + Sync + 'static,
    E: Send + Display + 'static,
{
    drop_tx: Sender<PoolNoodleTaskType<I, S>>,
    instance: Option<I>,
    spec: S,
}

impl<I, E, S> LifeGuard<I, E, S>
where
    I: Instance<Error = E> + Send + Sync + 'static,
    S: Spec<Error = E, Instance = I> + Clone + Send + Sync + 'static,
    E: Send + Display,
{
    pub(crate) fn new(
        instance: Option<I>,
        drop_tx: Sender<PoolNoodleTaskType<I, S>>,
        spec: S,
    ) -> Self {
        Self {
            drop_tx,
            instance,
            spec,
        }
    }
}

impl<I, E, S> Drop for LifeGuard<I, E, S>
where
    I: Instance<Error = E> + Send + Sync + 'static,
    S: Spec<Error = E, Instance = I> + Clone + Send + Sync + 'static,
    E: Send + Display + 'static,
{
    fn drop(&mut self)
    where
        I: Instance + Send + Sync,
        S: Spec<Instance = I> + Clone + Send + Sync,
        E: Send,
    {
        let instance = self
            .instance
            .take()
            .expect("Item must be present as it is initialized with Some and never replaced.");

        let id = instance.id();
        debug!("PoolNoodle: dropping instance: {}", id);
        let task =
            PoolNoodleTaskType::Drop(PoolNoodleTask::new(Some(instance), id, self.spec.clone()));

        if futures::executor::block_on(self.drop_tx.send(task)).is_err() {
            warn!("failed to drop instance: {}", id);
        };
        metric!(counter.pool_noodle.active = -1);
        metric!(counter.pool_noodle.task.drop = 1);
        debug!("PoolNoodle: instance pushed to dropped");
    }
}

impl<I, E, S> std::ops::Deref for LifeGuard<I, E, S>
where
    I: Instance<Error = E> + Send + Sync,
    S: Spec<Error = E, Instance = I> + Clone + Send + Sync,
    E: Send + Display,
{
    type Target = I;

    fn deref(&self) -> &I {
        self.instance
            .as_ref()
            .expect("Item must be present as it is initialized with Some and never replaced.")
    }
}

impl<I, E, S> std::ops::DerefMut for LifeGuard<I, E, S>
where
    I: Instance<Error = E> + Send + Sync,
    S: Spec<Error = E, Instance = I> + Clone + Send + Sync,
    E: Send + Display,
{
    fn deref_mut(&mut self) -> &mut I {
        self.instance
            .as_mut()
            .expect("Item must be present as it is initialized with Some and never replaced.")
    }
}
