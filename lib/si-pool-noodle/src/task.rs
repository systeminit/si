use std::{
    fmt::Display,
    result,
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
        self.spec
            .clean(self.id)
            .await
            .map_err(|err| PoolNoodleError::InstanceClean(err))
    }

    pub async fn prepare(&self) -> Result<(), E> {
        self.spec
            .prepare(self.id)
            .await
            .map_err(|err| PoolNoodleError::InstancePrepare(err))
    }

    pub async fn spawn(&self) -> Result<I, E> {
        self.spec
            .spawn(self.id)
            .await
            .map_err(|err| PoolNoodleError::InstanceSpawn(err))
    }

    pub async fn terminate(self) -> Result<(), E> {
        if let Some(mut instance) = self.instance {
            return instance
                .terminate()
                .await
                .map_err(|err| PoolNoodleError::InstanceTerminate(err));
        }
        Err(PoolNoodleError::InstanceNotFound)
    }
}
