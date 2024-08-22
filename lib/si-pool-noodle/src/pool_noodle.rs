use crossbeam_queue::ArrayQueue;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use telemetry_utils::metric;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::info;

use tokio::time::Duration;

use tracing::{debug, warn};

use std::result;

use crate::errors::PoolNoodleError;
use crate::{Instance, Spec};

/// [`pool_noodle`] implementations.

///---------------------------------------------------------------------
///---------------------------------------------------------------------
///---------------------------:::::::::::::::::::::---------------------
///---------------------:::::::::::::::::::::-------::::::--------------
///-----------------:::::::::::::::::::==========------::::::-----------
///----#*+#-----::::::::::::::::::::::::---:::--==++++-:::::::::--------
///---+#%@@#----::::::::::::::::::===========++++***#*=::::::::::::-----
///--=+*@@@@@*::::::::::::::::::::========++++****###*=::::::::::::::---
///--=@@@@@@@@%+::::::::::::::::::-=======+++++**###%#+:::::::::::::::--
///----=@@@@@@@@%=:::::::::::::::::=========++++*###%#+-::::::::::::::::
///------#@@@@@@@@%-:::::::::::::::-....:===:....:*#%#+-::::::::::::::::
///-----:::#@@@@@@@@%-::::::::::::..:-=.::+.:=**:-==%#*=::::::::::::::::
///----::::::%@@@@@@@@%=::::::::::.-#%@@--=.#%@@#*#*%%*=::::::::::::::::
///---:::::::::@@@@@@@@@%=::::::..:-*@@***#+-+####*%%%#+::::::::::::::::
///-:::::::::::%@@@@@@@@@@%#:.......+***#%%###**#%%%%%#+::::::::::::::::
///:::::::::::*#**#@@@@@@@@@%:......=+*#%%%%%@%#%%%%%%#*-:::::::::::::::
///::::::::::#@*+**%@@@@@@@@@=......:++*******####%%%%%*=:::::::::::::::
///:::::::::::=%##%@@@#%@%@@+........++++******###%%%%%*=:::::::::::::::
///:::::::::::=+*#@@%#%%%%@*.........++++++*****###%%%%#+.::::::::::::::
///::::::::::=++===+*%*===+=.........++++++*****####%%%#**=:::::::::::::
///:::::::::===++++*#@*+++**=........-+++++******###%%%%#%###*-:::::::::
///::::::::-++===*%@@@@%%%##:........++++++******###%%%%%%%###**+:::::::
///:::::::::*%***%@@@@@%%%#:.......=+=+++++******####%%%%#%###****+:::::
///::::::::::*@@@@@@@%%%%%#......-+==+++*********####%%%%+:.=##****+-:::
///:::::::::::::%@@@-%%%%%#*:.:++++++************####%%%%+::::-***+**-::
///--::::::::::::::::-#%%%###********#-**********####%%%#*.:::::**++**-:
///--::::::::::::::::::*#%%%#######*:..**********####%%%#*.:::::=*++**=:
///---:::::::::::::::::::-#%%###*-.....**********#####%%##::::::=*++**+:
///---::::::::::::::::::::::::::.......+*********#####%%%#-:::::+++**#=:
///---::::::::::::::::::::::::::::.....-********######%%%#+::::*++**##::
///----:::::::::::::::::::::::::::::::::#*******######%%%#*****++**##=::
///------::::::::::::::::::::::::::::::.********######%%%%##**+***#%=:::
///---------::::::::::::::::::::::::::::********######%%%%%*****##%-::::
///---------::::::::::::::::::::::::::::+*******######%%%%%#**##%=:::---
///----------:::::::::::::::::::::::::::-#*****#######%%%%%-:+#=::::----
///:------------:::::::::::::::::::::::::##****#######%%%%%+:::::::-----
///=:----------::::::::::::::::::::::::::*#***########%%%%%*::::::------

type Result<T> = result::Result<T, PoolNoodleError>;

/// Pool Noodle is a tool for ensuring that we maintain a bare minimum number of Firecracker Jails
/// for function execution. We wrap it in an Arc Mutex so we can update the queues it manages
/// across threads.
#[derive(Debug)]
pub struct PoolNoodle<I, S: Spec>(Arc<PoolNoodleInner<I, S>>);

impl<I, S: Spec> Clone for PoolNoodle<I, S> {
    fn clone(&self) -> Self {
        PoolNoodle(self.0.clone())
    }
}

impl<B: 'static, I, E, S> PoolNoodle<I, S>
where
    S: Spec<Error = E, Instance = I> + Send + Sync + 'static,
    I: Instance<SpecBuilder = B, Error = E> + Send + Sync + 'static,
    E: Send + Display + 'static,
{
    /// Creates a new instance of PoolNoodle
    pub fn new(pool_size: u32, spec: S, shutdown_rx: tokio::sync::broadcast::Receiver<()>) -> Self {
        // start by cleaning jails just to make sure
        let to_be_cleaned = ArrayQueue::new(pool_size as usize);
        let pool = PoolNoodle(Arc::new(PoolNoodleInner {
            pool_size,
            spec,
            dropped: ArrayQueue::new(pool_size as usize),
            ready: ArrayQueue::new(pool_size as usize),
            to_be_cleaned,
            unprepared: ArrayQueue::new(pool_size as usize),
            shutdown_rx: shutdown_rx.into(),
        }));
        for n in 1..=pool_size {
            let me = Arc::clone(&pool.0);
            Self::push_to_clean(me, n);
        }
        pool
    }

    /// Gets the current pool stats from the inner struct
    /// Only used for tests
    pub async fn stats(&self) -> PoolNoodleStats {
        self.0.stats().await
    }

    /// Start PoolNoodle. It will spin up various threads to handle Cyclone Instance lifecycles.
    #[allow(clippy::let_underscore_future)] // These needs to just run in the background forever.
    pub fn start(&mut self, check_health: bool) -> Result<()> {
        if check_health {
            if let Some(err) = futures::executor::block_on(self.check_health()).err() {
                return Err(err);
            }
        }
        let stop = Arc::new(AtomicBool::new(false));
        let me = Arc::clone(&self.0);

        let _ = tokio::spawn(Self::handle_shutdown(me.clone(), stop.clone()));

        for _ in 0..10 {
            let _ = tokio::spawn(Self::handle_prepare(me.clone(), stop.clone()));

            let _ = tokio::spawn(Self::handle_clean(me.clone()));

            let _ = tokio::spawn(Self::handle_drop(me.clone()));
        }
        Ok(())
    }

    async fn handle_shutdown(me: Arc<PoolNoodleInner<I, S>>, stop: Arc<AtomicBool>) {
        debug!("PoolNoodle: starting shutdown handler...");

        while me.shutdown_rx.lock().await.try_recv().is_err() {
            sleep(Duration::from_millis(1)).await;
        }

        debug!("PoolNoodle: received graceful shutdown signal, shutting down...");
        stop.store(true, Ordering::Relaxed);
    }

    async fn handle_prepare(me: Arc<PoolNoodleInner<I, S>>, stop: Arc<AtomicBool>) {
        debug!("PoolNoodle: starting prepare handler...");

        while !stop.load(Ordering::Relaxed) {
            // let's make more instances!
            if let Some(id) = Self::pop_from_unprepared(me.clone()) {
                debug!("PoolNoodle: readying instance");
                match PoolNoodleInner::prepare(id, &me.spec).await {
                    Ok(_) => {
                        debug!("PoolNoodle: instance readied: {}", id);
                        match PoolNoodleInner::spawn(id, &me.spec).await {
                            Ok(instance) => {
                                debug!("PoolNoodle: instance started: {}", id);
                                Self::push_to_ready(me.clone(), instance);
                            }
                            Err(e) => {
                                warn!("PoolNoodle: failed to start instance: {}", id);
                                warn!("{:?}", e);
                                Self::push_to_clean(me.clone(), id);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("PoolNoodle: failed to ready instance: {}", id);
                        warn!("{:?}", e);
                        Self::push_to_clean(me.clone(), id);
                    }
                }
            }
            sleep(Duration::from_millis(1)).await;
        }
        debug!("PoolNoodle: received graceful shutdown signal, shutting down...");
    }

    async fn handle_clean(me: Arc<PoolNoodleInner<I, S>>) {
        debug!("PoolNoodle: starting clean handler...");

        loop {
            if let Some(id) = Self::pop_from_clean(me.clone()) {
                debug!("PoolNoodle: cleaning instance {}", id);
                match PoolNoodleInner::clean(id, &me.spec).await {
                    Ok(_) => {
                        debug!("PoolNoodle: instance cleaned: {}", id);
                        Self::push_to_unprepared(me.clone(), id)
                    }
                    Err(e) => {
                        warn!("PoolNoodle: failed to clean instance: {}", id);
                        warn!("{:?}", e);
                        Self::push_to_clean(me.clone(), id);
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    async fn handle_drop(me: Arc<PoolNoodleInner<I, S>>) {
        debug!("PoolNoodle: starting drop handler...");

        loop {
            if let Some(instance) = me.dropped.pop() {
                let id = instance.id();
                debug!("PoolNoodle: dropping: {}", id);
                match PoolNoodleInner::terminate(instance, &me.spec).await {
                    Ok(_) => {
                        debug!("PoolNoodle: instance terminated: {}", id);
                        Self::push_to_clean(me.clone(), id);
                    }
                    Err(e) => {
                        warn!("PoolNoodle: failed to terminate instance: {}", id);
                        warn!("{:?}", e);
                        Self::push_to_clean(me.clone(), id);
                    }
                }
            }
            sleep(Duration::from_millis(1)).await;
        }
    }

    fn pop_from_clean(me: Arc<PoolNoodleInner<I, S>>) -> Option<u32> {
        me.to_be_cleaned.pop().map(|id| {
            metric!(counter.pool_noodle.to_be_cleaned = -1);
            Some(id)
        })?
    }

    fn pop_from_unprepared(me: Arc<PoolNoodleInner<I, S>>) -> Option<u32> {
        me.unprepared.pop().map(|id| {
            metric!(counter.pool_noodle.unprepared = -1);
            Some(id)
        })?
    }

    fn pop_from_ready(me: Arc<PoolNoodleInner<I, S>>) -> Option<I> {
        me.ready.pop().map(|id| {
            metric!(counter.pool_noodle.ready = -1);
            Some(id)
        })?
    }

    fn push_to_clean(me: Arc<PoolNoodleInner<I, S>>, id: u32) {
        if let Err(e) = me.to_be_cleaned.push(id) {
            warn!(
                "PoolNoodle: failed to push instance to to_be_cleaned: {}",
                id
            );
            warn!("{:?}", e);
        }
        metric!(counter.pool_noodle.to_be_cleaned = 1);
    }

    fn push_to_ready(me: Arc<PoolNoodleInner<I, S>>, instance: I) {
        if let Err(i) = me.ready.push(instance) {
            warn!("PoolNoodle: failed to push instance to ready: {}", i.id());
        }
        metric!(counter.pool_noodle.ready = 1);
    }

    fn push_to_unprepared(me: Arc<PoolNoodleInner<I, S>>, id: u32) {
        if let Err(e) = me.unprepared.push(id) {
            warn!("PoolNoodle: failed to push instance to unprepared: {}", id);
            warn!("{:?}", e);
        }
        metric!(counter.pool_noodle.unprepared = 1);
    }

    /// This will attempt to get a ready, healthy instance from the pool.
    /// If there are no instances, it will give the main loop a chance to fill the pool and try
    /// again. It will throw an error if there are no available instances after enough retries.
    pub async fn get(&mut self) -> Result<LifeGuard<I, S>> {
        let me = Arc::clone(&self.0);

        let max_retries = 6000; // Set the maximum number of retries
        let mut retries = 0;
        loop {
            if retries >= max_retries {
                return Err(PoolNoodleError::ExecutionPoolStarved);
            }
            if let Some(mut instance) = Self::pop_from_ready(me.clone()) {
                debug!("PoolNoodle: got instance: {}", instance.id());
                // Try to ensure the item is healthy
                match &mut instance.ensure_healthy().await {
                    Ok(_) => {
                        debug!(
                            "PoolNoodle: got instance for func execution: {}",
                            &instance.id()
                        );
                        metric!(counter.pool_noodle.active = 1);
                        return Ok(LifeGuard {
                            pool: me.clone(),
                            item: Some(instance),
                        });
                    }
                    Err(_) => {
                        debug!("PoolNoodle: not healthy, cleaning up and getting a new one.");
                        drop(instance);
                    }
                }
            } else {
                retries += 1;
                debug!(
                    "Failed to get from pool, retry ({} of {})",
                    retries, max_retries
                );
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    async fn check_health(&mut self) -> Result<()> {
        info!("verifying instance lifecycle health");
        let me = Arc::clone(&self.0);
        let id = 0;
        info!("cleaning...");
        PoolNoodleInner::clean(id, &me.spec).await?;
        info!("preparing...");
        PoolNoodleInner::prepare(id, &me.spec).await?;
        info!("spawning...");
        let mut i = PoolNoodleInner::spawn(id, &me.spec).await?;
        info!("checking...");
        i.ensure_healthy()
            .await
            .map_err(|e| PoolNoodleError::Unhealthy(e.to_string()))?;
        info!("terminating...");
        PoolNoodleInner::terminate(i, &me.spec).await?;
        PoolNoodleInner::clean(id, &me.spec).await?;
        info!("instance lifecycle is good!");
        Ok(())
    }
}

#[derive(Debug)]
struct PoolNoodleInner<I, S>
where
    S: Spec,
{
    pool_size: u32,
    spec: S,
    dropped: ArrayQueue<I>,
    ready: ArrayQueue<I>,
    to_be_cleaned: ArrayQueue<u32>,
    unprepared: ArrayQueue<u32>,
    shutdown_rx: Mutex<tokio::sync::broadcast::Receiver<()>>,
}

impl<B, I, E, S> PoolNoodleInner<I, S>
where
    S: Spec<Error = E, Instance = I> + Send + Sync + 'static,
    I: Instance<SpecBuilder = B, Error = E> + Send + Sync + 'static,
    E: Send + Display,
{
    async fn clean(id: u32, spec: &S) -> Result<()> {
        spec.clean(id)
            .await
            .map_err(|e| PoolNoodleError::InstanceClean(e.to_string()))
    }

    async fn prepare(id: u32, spec: &S) -> Result<()> {
        spec.prepare(id)
            .await
            .map_err(|e| PoolNoodleError::InstancePrepare(e.to_string()))
    }

    /// This starts the instance. It will be available to .get() to execute functions
    async fn spawn(id: u32, spec: &S) -> Result<I> {
        spec.spawn(id)
            .await
            .map_err(|e| PoolNoodleError::InstanceSpawn(e.to_string()))
    }

    /// This terminates the instance
    async fn terminate(mut instance: I, _: &S) -> Result<()> {
        instance
            .terminate()
            .await
            .map_err(|e| PoolNoodleError::InstanceTerminate(e.to_string()))
    }

    /// This outputs the current state of the pool
    pub async fn stats(&self) -> PoolNoodleStats {
        PoolNoodleStats {
            pool_size: self.pool_size as usize,
            dropped: self.dropped.len(),
            ready: self.ready.len(),
            to_be_cleaned: self.to_be_cleaned.len(),
            unprepared: self.unprepared.len(),
        }
    }
}

#[derive(Debug)]
/// Gets the current stats for the pool
pub struct PoolNoodleStats {
    /// Total number of instances allowed in the pool
    pub pool_size: usize,
    /// Total number of instances dropped and awating to be cleaned
    pub dropped: usize,
    /// Total number of instances that have been fetched from the pool and not yet dropped
    pub ready: usize,
    /// Total number of instances that need to be cleaned up
    pub to_be_cleaned: usize,
    /// Total number of unclaimed instances waiting to be readied
    pub unprepared: usize,
}

impl Display for PoolNoodleStats {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "PoolNoodle Stats -- pool size: {}, dropped: {}, ready: {}, to be cleaned: {}, unprepared: {}",
            self.pool_size,  self.dropped, self.ready, self.to_be_cleaned, self.unprepared
        )
    }
}

/// LifeGuard is a wrapper for instances that come from the pool.
/// It is carries a Sender and implements Drop. When an instance goes out of
/// scope, it lets PoolNoodle know that the instance needs to be cleaned up.
#[derive(Debug)]
pub struct LifeGuard<I, S>
where
    I: Instance + Send + Sync,
    S: Spec,
{
    pool: Arc<PoolNoodleInner<I, S>>,
    item: Option<I>,
}

impl<I, S> Drop for LifeGuard<I, S>
where
    I: Instance + Send + Sync,
    S: Spec,
{
    fn drop(&mut self) {
        let item = self
            .item
            .take()
            .expect("Item must be present as it is initialized with Some and never replaced.");
        debug!("PoolNoodle: dropping instance: {}", item.id());

        if let Err(i) = self.pool.dropped.push(item) {
            warn!(
                "PoolNoodle: failed to push instance to dropped: {}",
                &i.id()
            );
        }
        metric!(counter.pool_noodle.active = -1);
        debug!("PoolNoodle: instance pushed to dropped");
    }
}

impl<I, S> std::ops::Deref for LifeGuard<I, S>
where
    I: Instance + Send + Sync,
    S: Spec,
{
    type Target = I;

    fn deref(&self) -> &I {
        self.item
            .as_ref()
            .expect("Item must be present as it is initialized with Some and never replaced.")
    }
}

impl<I, S> std::ops::DerefMut for LifeGuard<I, S>
where
    I: Instance + Send + Sync,
    S: Spec,
{
    fn deref_mut(&mut self) -> &mut I {
        self.item
            .as_mut()
            .expect("Item must be present as it is initialized with Some and never replaced.")
    }
}

#[cfg(test)]
mod tests {

    use crate::instance::SpecBuilder;
    use async_trait::async_trait;
    use derive_builder::Builder;
    use tokio::{
        sync::broadcast,
        time::{sleep, Duration},
    };

    use super::*;

    pub struct DummyInstance {}

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
        let spec = DummyInstanceSpec {};

        let (shutdown_broadcast_tx, _) = broadcast::channel(16);
        let mut pool = PoolNoodle::new(3, spec, shutdown_broadcast_tx.subscribe());
        pool.start(false).expect("failed to start");

        // give the pool time to create some instances
        sleep(Duration::from_millis(500)).await;
        assert_eq!(3, pool.stats().await.ready, "{}", pool.stats().await);
        // go get an instance
        let mut instance = pool.get().await.expect("should be able to get an instance");
        instance.ensure_healthy().await.expect("failed healthy");
        assert_eq!(2, pool.stats().await.ready, "{}", pool.stats().await);
        drop(instance);
        let a = pool.get().await.expect("should be able to get an instance");
        let b = pool.get().await.expect("should be able to get an instance");
        let c = pool.get().await.expect("should be able to get an instance");
        assert_eq!(0, pool.stats().await.ready, "{}", pool.stats().await);
        drop(a);
        drop(b);
        drop(c);
        // pool should refill
        sleep(Duration::from_millis(1000)).await;
        assert_eq!(3, pool.stats().await.ready, "{}", pool.stats().await);
    }
}
