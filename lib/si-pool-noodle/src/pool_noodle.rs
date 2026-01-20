//! [`pool_noodle`] implementations.
//!
//! ---------------------------------------------------------------------
//! ---------------------------------------------------------------------
//! ---------------------------:::::::::::::::::::::---------------------
//! ---------------------:::::::::::::::::::::-------::::::--------------
//! -----------------:::::::::::::::::::==========------::::::-----------
//! ----#*+#-----::::::::::::::::::::::::---:::--==++++-:::::::::--------
//! ---+#%@@#----::::::::::::::::::===========++++***#*=::::::::::::-----
//! --=+*@@@@@*::::::::::::::::::::========++++****###*=::::::::::::::---
//! --=@@@@@@@@%+::::::::::::::::::-=======+++++**###%#+:::::::::::::::--
//! ----=@@@@@@@@%=:::::::::::::::::=========++++*###%#+-::::::::::::::::
//! ------#@@@@@@@@%-:::::::::::::::-....:===:....:*#%#+-::::::::::::::::
//! -----:::#@@@@@@@@%-::::::::::::..:-=.::+.:=**:-==%#*=::::::::::::::::
//! ----::::::%@@@@@@@@%=::::::::::.-#%@@--=.#%@@#*#*%%*=::::::::::::::::
//! ---:::::::::@@@@@@@@@%=::::::..:-*@@***#+-+####*%%%#+::::::::::::::::
//! -:::::::::::%@@@@@@@@@@%#:.......+***#%%###**#%%%%%#+::::::::::::::::
//! :::::::::::*#**#@@@@@@@@@%:......=+*#%%%%%@%#%%%%%%#*-:::::::::::::::
//! ::::::::::#@*+**%@@@@@@@@@=......:++*******####%%%%%*=:::::::::::::::
//! :::::::::::=%##%@@@#%@%@@+........++++******###%%%%%*=:::::::::::::::
//! :::::::::::=+*#@@%#%%%%@*.........++++++*****###%%%%#+.::::::::::::::
//! ::::::::::=++===+*%*===+=.........++++++*****####%%%#**=:::::::::::::
//! :::::::::===++++*#@*+++**=........-+++++******###%%%%#%###*-:::::::::
//! ::::::::-++===*%@@@@%%%##:........++++++******###%%%%%%%###**+:::::::
//! :::::::::*%***%@@@@@%%%#:.......=+=+++++******####%%%%#%###****+:::::
//! ::::::::::*@@@@@@@%%%%%#......-+==+++*********####%%%%+:.=##****+-:::
//! :::::::::::::%@@@-%%%%%#*:.:++++++************####%%%%+::::-***+**-::
//! --::::::::::::::::-#%%%###********#-**********####%%%#*.:::::**++**-:
//! --::::::::::::::::::*#%%%#######*:..**********####%%%#*.:::::=*++**=:
//! ---:::::::::::::::::::-#%%###*-.....**********#####%%##::::::=*++**+:
//! ---::::::::::::::::::::::::::.......+*********#####%%%#-:::::+++**#=:
//! ---::::::::::::::::::::::::::::.....-********######%%%#+::::*++**##::
//! ----:::::::::::::::::::::::::::::::::#*******######%%%#*****++**##=::
//! ------::::::::::::::::::::::::::::::.********######%%%%##**+***#%=:::
//! ---------::::::::::::::::::::::::::::********######%%%%%*****##%-::::
//! ---------::::::::::::::::::::::::::::+*******######%%%%%#**##%=:::---
//! ----------:::::::::::::::::::::::::::-#*****#######%%%%%-:+#=::::----
//! :------------:::::::::::::::::::::::::##****#######%%%%%+:::::::-----
//! =:----------::::::::::::::::::::::::::*#***########%%%%%*::::::------

use std::{
    fmt::Display,
    result,
    sync::Arc,
};

use crossbeam_queue::ArrayQueue;
use telemetry_utils::metric;
use tokio::{
    sync::{
        Mutex,
        Semaphore,
        mpsc::{
            self,
            Receiver,
            Sender,
        },
    },
    time::{
        Duration,
        sleep,
        timeout,
    },
};
use tokio_util::sync::CancellationToken;
use tracing::{
    debug,
    info,
    warn,
};

use crate::{
    Instance,
    Spec,
    errors::PoolNoodleError,
    lifeguard::LifeGuard,
    task::{
        PoolNoodleTask,
        PoolNoodleTaskType,
    },
};

type Result<T, E> = result::Result<T, PoolNoodleError<E>>;

#[derive(Clone, Debug)]
/// Configuration object for setting up pool noodle
pub struct PoolNoodleConfig<S> {
    /// Verify instances can be started and stopped before starting the pool management tasks
    pub check_health: bool,
    /// Max number of worker threads to run at once. Defaults to available_parallelism() or 16
    pub max_concurrency: u32,
    /// Maximum number of instances to manage at once
    pub pool_size: u32,
    /// Number of attempts to get from the pool before giving up with 10 ms between attempts
    pub retry_limit: u32,
    /// Shuts down the pool management tasks
    pub shutdown_token: CancellationToken,
    /// The spec for the type of instance to manage
    pub spec: S,
}

impl<S> Default for PoolNoodleConfig<S>
where
    S: Spec + Default,
{
    fn default() -> Self {
        Self {
            check_health: false,
            max_concurrency: 1000,
            pool_size: 100,
            retry_limit: 120, // * 100ms between tries, we will try for 2 minutes before giving up
            shutdown_token: CancellationToken::new(),
            spec: S::default(),
        }
    }
}

/// Pool Noodle is a tool for ensuring that we maintain a bare minimum number of Firecracker Jails
/// for function execution. We wrap it in an Arc Mutex so we can update the queues it manages
/// across threads.
#[derive(Debug)]
pub struct PoolNoodle<I, S: Spec>(Arc<PoolNoodleInner<I, S>>);

impl<I, E, S> Clone for PoolNoodle<I, S>
where
    I: Instance<Error = E> + Send + Sync + 'static,
    S: Spec<Error = E, Instance = I> + Send + Sync + 'static,
    E: Send,
{
    fn clone(&self) -> Self {
        PoolNoodle(self.0.clone())
    }
}

impl<I, E, S> PoolNoodle<I, S>
where
    I: Instance<Error = E> + Send + Sync + 'static,
    S: Spec<Error = E, Instance = I> + Clone + Send + Sync + 'static,
    E: Send + Sync + Display + 'static,
{
    /// Creates a new instance of PoolNoodle
    pub async fn new(config: PoolNoodleConfig<S>) -> Self {
        PoolNoodle(Arc::new(PoolNoodleInner::new(config)))
    }

    /// do the thing
    pub fn run(&mut self) -> Result<(), E> {
        if self.inner().check_health {
            if let Some(err) =
                futures::executor::block_on(timeout(Duration::from_secs(60), self.check_health()))
                    .err()
            {
                return Err(PoolNoodleError::UnhealthyTimeout(err));
            }
        }
        let inner = self.inner();

        tokio::spawn(async move {
            let mut q = inner.queue_rx.lock().await;
            let semaphore = Arc::new(Semaphore::new(inner.max_concurrency as usize));
            loop {
                tokio::select! {

                    _ = inner.shutdown_token.cancelled() => {
                        debug!("main loop received cancellation");
                        break;
                    }

                    Some(task_type) = q.recv() => {
                        let inner = inner.clone();
                        let permit = semaphore.clone().acquire_owned().await;

                        tokio::spawn(async move {
                            inner.handle_task(task_type).await;
                            drop(permit);
                        });
                    }
                }
            }
        });

        // start by cleaning jails just to make sure
        let inner = self.inner();
        tokio::spawn(async move {
            for id in 1..=inner.pool_size {
                inner.push_clean_task_to_work_queue(id).await;
            }
        });

        Ok(())
    }

    fn inner(&self) -> Arc<PoolNoodleInner<I, S>> {
        Arc::clone(&self.0)
    }

    /// Returns the admission semaphore for external backpressure control.
    pub fn admission_semaphore(&self) -> Arc<Semaphore> {
        self.inner().admission_semaphore.clone()
    }

    /// This will attempt to get a ready, healthy instance from the pool.
    /// If there are no instances, it will give the main loop a chance to fill the pool and try
    /// again. It will throw an error if there are no available instances after enough retries.
    pub async fn get(&self) -> Result<LifeGuard<I, E, S>, E> {
        metric!(counter.pool_noodle.get_requests = 1);
        let inner = self.inner();

        let max_retries = self.inner().retry_limit; // Set the maximum number of retries
        let mut retries = 0;
        loop {
            if retries >= max_retries {
                metric!(counter.pool_noodle.get_requests = -1);
                return Err(PoolNoodleError::ExecutionPoolStarved);
            }
            match inner.ready_queue.pop() {
                Some(mut instance) => {
                    metric!(counter.pool_noodle.ready = -1);
                    // Try to ensure the item is healthy
                    match &mut instance.ensure_healthy().await {
                        Ok(_) => {
                            metric!(counter.pool_noodle.get_requests = -1);
                            metric!(counter.pool_noodle.active = 1);
                            return Ok(LifeGuard::new(
                                Some(instance),
                                inner.queue_tx.clone(),
                                inner.spec.clone(),
                            ));
                        }
                        Err(_) => {
                            debug!("PoolNoodle: not healthy, cleaning up and getting a new one.");
                            drop(instance);
                        }
                    }
                }
                _ => {
                    retries += 1;
                    debug!(
                        "Failed to get from pool, retry ({} of {})",
                        retries, max_retries
                    );
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn check_health(&mut self) -> Result<(), E> {
        info!("verifying instance lifecycle health");
        let id = 0;
        let mut task = PoolNoodleTask::new(None, id, self.inner().spec.clone());
        info!("cleaning...");
        task.clean().await?;
        info!("preparing...");
        task.prepare().await?;
        info!("spawning...");
        let mut i = task.spawn().await?;
        info!("checking...");
        i.ensure_healthy()
            .await
            .map_err(|err| PoolNoodleError::Unhealthy(err))?;
        info!("terminating...");
        task.set_instance(Some(i));
        task.terminate().await?;
        self.inner()
            .spec
            .clean(id)
            .await
            .map_err(|err| PoolNoodleError::InstanceClean(err))?;
        info!("instance lifecycle is good!");
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct PoolNoodleInner<I, S>
where
    S: Spec,
{
    check_health: bool,
    max_concurrency: u32,
    pool_size: u32,
    ready_queue: ArrayQueue<I>,
    retry_limit: u32,
    shutdown_token: CancellationToken,
    spec: S,
    queue_rx: Mutex<Receiver<PoolNoodleTaskType<I, S>>>,
    queue_tx: Sender<PoolNoodleTaskType<I, S>>,
    admission_semaphore: Arc<Semaphore>,
}

impl<I, E, S> PoolNoodleInner<I, S>
where
    I: Instance<Error = E> + Send + Sync + 'static,
    S: Spec<Error = E, Instance = I> + Clone + Send + Sync + 'static,
    E: Send + Display + 'static,
{
    fn new(config: PoolNoodleConfig<S>) -> Self {
        info!(
            "creating a pool of size {} with concurrency of {} ",
            config.pool_size, config.max_concurrency
        );
        let (queue_tx, queue_rx) = mpsc::channel(config.pool_size as usize);
        Self {
            check_health: config.check_health,
            max_concurrency: config.max_concurrency,
            pool_size: config.pool_size,
            ready_queue: ArrayQueue::new(config.pool_size as usize),
            retry_limit: config.retry_limit,
            shutdown_token: config.shutdown_token,
            spec: config.spec,
            queue_rx: queue_rx.into(),
            queue_tx,
            admission_semaphore: Arc::new(Semaphore::new(0)),
        }
    }

    async fn handle_task(self: Arc<Self>, task_type: PoolNoodleTaskType<I, S>) {
        match task_type {
            PoolNoodleTaskType::Clean(task) => self.handle_clean(task).await,
            PoolNoodleTaskType::Drop(task) => self.handle_drop(task).await,
            PoolNoodleTaskType::Prepare(task) => self.handle_prepare(task).await,
        }
    }

    // clean instances with backoff to handle intermittent failures. If an instance fails enough it
    // will be abandoned
    async fn handle_clean(&self, task: PoolNoodleTask<I, S>) {
        metric!(counter.pool_noodle.task.clean = -1);
        let id = task.id();
        let max_retries = 5;
        let mut attempts = 0;
        loop {
            match task.clean().await {
                Ok(_) => {
                    self.push_prepare_task_to_work_queue(id).await;
                    break;
                }
                Err(e) => {
                    if attempts >= max_retries {
                        warn!(
                            "Failed to clean instance {} after {} attempts. Abandoning this instance",
                            id, max_retries
                        );
                        break;
                    }
                    warn!("PoolNoodle: failed to clean instance: {}", id);
                    warn!("{}", e);
                    warn!("Trying again, {} of {}", attempts, max_retries);
                    attempts += 1;
                    tokio::time::sleep(Duration::from_millis(100 * (attempts * attempts))).await;
                }
            }
        }
    }

    async fn handle_drop(&self, task: PoolNoodleTask<I, S>) {
        metric!(counter.pool_noodle.task.drop = -1);
        let id = task.id();
        match task.terminate().await {
            Ok(_) => {
                self.push_clean_task_to_work_queue(id).await;
            }
            Err(e) => {
                warn!("PoolNoodle: failed to drop instance: {}", id);
                warn!("{}", e);
            }
        }
    }

    async fn handle_prepare(&self, task: PoolNoodleTask<I, S>) {
        metric!(counter.pool_noodle.task.prepare = -1);
        let id = task.id();
        match &task.prepare().await {
            Ok(_) => match task.spawn().await {
                Ok(instance) => {
                    self.push_to_ready_queue(instance).await;
                }
                Err(e) => {
                    warn!("PoolNoodle: failed to start instance: {}", id);
                    warn!("{}", e);
                    self.push_clean_task_to_work_queue(id).await;
                }
            },
            Err(e) => {
                warn!("PoolNoodle: failed to ready instance: {}", id);
                warn!("{}", e);
                self.push_clean_task_to_work_queue(id).await;
            }
        }
    }

    async fn push_clean_task_to_work_queue(&self, id: u32) {
        let task = PoolNoodleTaskType::Clean(PoolNoodleTask::new(None, id, self.spec.clone()));
        if self.queue_tx.send(task).await.is_err() {
            warn!("failed to push instance to clean: {}", id);
        };
        metric!(counter.pool_noodle.task.clean = 1);
    }

    async fn push_prepare_task_to_work_queue(&self, id: u32) {
        let task = PoolNoodleTaskType::Prepare(PoolNoodleTask::new(None, id, self.spec.clone()));
        if self.queue_tx.send(task).await.is_err() {
            warn!("failed to push instance to prepare: {}", id);
        };
        metric!(counter.pool_noodle.task.prepare = 1);
    }

    async fn push_to_ready_queue(&self, instance: I) {
        let id = instance.id();
        if self.ready_queue.push(instance).is_err() {
            warn!("failed to push to ready queue: {}", id);
        }
        metric!(counter.pool_noodle.ready = 1);
        self.admission_semaphore.add_permits(1);
    }
}

#[cfg(test)]
mod tests {

    use std::fmt::{
        self,
        Formatter,
    };

    use async_trait::async_trait;
    use derive_builder::Builder;
    use tokio::time::{
        Duration,
        sleep,
    };

    use super::*;
    use crate::instance::SpecBuilder;

    pub struct DummyInstance {}

    #[derive(Clone)]
    pub struct DummyInstanceSpec {}
    #[async_trait]
    impl Spec for DummyInstanceSpec {
        type Instance = DummyInstance;
        type Error = DummyInstanceError;

        async fn clean(&self, _id: u32) -> result::Result<(), Self::Error> {
            Ok(())
        }
        async fn prepare(&self, _id: u32) -> result::Result<(), Self::Error> {
            Ok(())
        }
        async fn setup(&mut self) -> result::Result<(), Self::Error> {
            Ok(())
        }

        async fn spawn(&self, _id: u32) -> result::Result<Self::Instance, Self::Error> {
            Ok(DummyInstance {})
        }
    }
    #[derive(Builder, Default, Clone)]
    pub struct DummyInstanceBuilder {}
    impl SpecBuilder for DummyInstanceBuilder {
        type Spec = DummyInstanceSpec;
        type Error = DummyInstanceError;

        fn build(&self) -> result::Result<Self::Spec, Self::Error> {
            Ok(DummyInstanceSpec {})
        }
    }
    #[derive(Debug)]
    pub struct DummyInstanceError {}

    impl Display for DummyInstanceError {
        fn fmt(&self, _f: &mut Formatter) -> fmt::Result {
            Ok(())
        }
    }
    #[async_trait]
    impl Instance for DummyInstance {
        type SpecBuilder = DummyInstanceBuilder;
        type Error = DummyInstanceError;

        async fn terminate(&mut self) -> result::Result<(), Self::Error> {
            Ok(())
        }

        async fn ensure_healthy(&mut self) -> result::Result<(), Self::Error> {
            Ok(())
        }

        fn id(&self) -> u32 {
            0
        }
    }
    #[tokio::test]
    async fn pool_noodle_lifecycle() {
        let shutdown_token = CancellationToken::new();

        let spec = DummyInstanceSpec {};

        let config = PoolNoodleConfig {
            check_health: false,
            max_concurrency: 10,
            pool_size: 3,
            retry_limit: 3,
            shutdown_token: shutdown_token.clone(),
            spec,
        };
        let mut pool = PoolNoodle::new(config).await;
        pool.run().expect("failed to start");

        // give the pool time to create some instances
        sleep(Duration::from_millis(500)).await;
        // go get an instance
        let mut instance = pool.get().await.expect("should be able to get an instance");
        instance.ensure_healthy().await.expect("failed healthy");
        drop(instance);

        let a = pool.get().await.expect("should be able to get an instance");
        let b = pool.get().await.expect("should be able to get an instance");
        let c = pool.get().await.expect("should be able to get an instance");
        drop(a);
        drop(b);
        drop(c);
        shutdown_token.cancel();
        assert!(pool.get().await.is_err());
    }

    #[tokio::test]
    async fn admission_semaphore_tracks_ready_instances() {
        let shutdown_token = CancellationToken::new();
        let pool_size = 3_u32;

        let spec = DummyInstanceSpec {};

        let config = PoolNoodleConfig {
            check_health: false,
            max_concurrency: 10,
            pool_size,
            retry_limit: 3,
            shutdown_token: shutdown_token.clone(),
            spec,
        };
        let mut pool = PoolNoodle::new(config).await;

        // Before running, semaphore should have 0 permits
        let semaphore = pool.admission_semaphore();
        assert_eq!(
            semaphore.available_permits(),
            0,
            "semaphore should start with 0 permits"
        );

        pool.run().expect("failed to start");

        // Wait for the pool to initialize all instances
        // The semaphore should gain permits as instances become ready
        let timeout_result = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let permits = semaphore.available_permits();
                if permits >= pool_size as usize {
                    return permits;
                }
                sleep(Duration::from_millis(50)).await;
            }
        })
        .await;

        let final_permits = timeout_result.expect("timed out waiting for pool to initialize");
        assert_eq!(
            final_permits, pool_size as usize,
            "semaphore should have permits equal to pool size"
        );

        // When we take an instance from the pool, the permit count should not
        // change because permits are managed by naxum, not by get().
        let permits_before_get = semaphore.available_permits();
        let _instance = pool.get().await.expect("should be able to get an instance");
        assert_eq!(
            semaphore.available_permits(),
            permits_before_get,
            "pool.get() should not consume semaphore permits"
        );

        shutdown_token.cancel();
    }
}
