use crate::pool_noodle::PoolNoodleInner;
use std::sync::Arc;
use tracing::info;

use std::fmt::Display;

use crate::Spec;

use crate::Instance;

use telemetry_utils::metric;

use tracing::debug;

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
    item: Option<I>,
    pool: Arc<PoolNoodleInner<I, S>>,
}

impl<I, E, S> LifeGuard<I, E, S>
where
    I: Instance<Error = E> + Send + Sync + 'static,
    S: Spec<Error = E, Instance = I> + Clone + Send + Sync + 'static,
    E: Send + Display,
{
    pub(crate) fn new(item: Option<I>, pool: Arc<PoolNoodleInner<I, S>>) -> Self {
        Self { item, pool }
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
        let item = self
            .item
            .take()
            .expect("Item must be present as it is initialized with Some and never replaced.");
        debug!("PoolNoodle: dropping instance: {}", item.id());

        self.pool.push_drop_task_to_work_queue(item);
        metric!(counter.pool_noodle.active = -1);
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
        self.item
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
        self.item
            .as_mut()
            .expect("Item must be present as it is initialized with Some and never replaced.")
    }
}
