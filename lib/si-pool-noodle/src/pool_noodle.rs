use crossbeam_queue::ArrayQueue;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::info;

use tokio::sync::mpsc::{self, Receiver, Sender};
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
        let (drop_tx, drop_rx) = mpsc::channel(16);
        let unprepared = ArrayQueue::new(pool_size as usize);
        for n in 1..=pool_size {
            let _ = unprepared.push(n);
        }
        PoolNoodle(Arc::new(PoolNoodleInner {
            pool_size,
            spec,
            ready: ArrayQueue::new(pool_size as usize),
            to_be_cleaned: ArrayQueue::new(pool_size as usize),
            unprepared,
            shutdown_rx: shutdown_rx.into(),
            drop_tx,
            drop_rx: drop_rx.into(),
        }))
    }

    /// Gets the current pool stats from the inner struct
    /// Only used for tests
    pub async fn stats(&self) -> PoolNoodleStats {
        self.0.stats().await
    }

    /// Start PoolNoodle. It will spin up various threads to handle Cyclone Instance lifecycles.
    #[allow(clippy::let_underscore_future)] // These needs to just run in the background forever.
    pub fn start(&mut self) {
        let stop = Arc::new(AtomicBool::new(false));
        let me = Arc::clone(&self.0);

        let _ = tokio::spawn(Self::handle_shutdown(me.clone(), stop.clone()));

        let _ = tokio::spawn(Self::handle_prepare(me.clone(), stop.clone()));

        let _ = tokio::spawn(Self::handle_clean(me.clone(), stop.clone()));

        let _ = tokio::spawn(Self::handle_drop(me.clone(), stop.clone()));
    }

    async fn handle_shutdown(me: Arc<PoolNoodleInner<I, S>>, stop: Arc<AtomicBool>) {
        debug!("PoolNoodle: starting shutdown handler...");
        loop {
            if me.shutdown_rx.lock().await.try_recv().is_ok() {
                debug!("PoolNoodle: received graceful shutdown signal, shutting down...");
                stop.store(true, Ordering::Relaxed);
                break;
            }
            sleep(Duration::from_millis(1)).await;
        }
    }

    async fn handle_prepare(me: Arc<PoolNoodleInner<I, S>>, stop: Arc<AtomicBool>) {
        debug!("PoolNoodle: starting prepare handler...");
        loop {
            if stop.load(Ordering::Relaxed) {
                debug!("PoolNoodle: received graceful shutdown signal, shutting down...");
                break;
            }

            // let's make more instances!
            if let Some(id) = me.unprepared.pop() {
                debug!("PoolNoodle: readying instance");
                match PoolNoodleInner::prepare(id, &me.spec).await {
                    Ok(_) => {
                        debug!("PoolNoodle: instance readied: {}", id);
                        match PoolNoodleInner::spawn(id, &me.spec).await {
                            Ok(instance) => {
                                debug!("PoolNoodle: instance started: {}", id);
                                let _ = me.ready.push(instance);
                            }
                            Err(e) => {
                                warn!("PoolNoodle: failed to start instance: {}", id);
                                warn!("{:?}", e);
                                let _ = me.to_be_cleaned.push(id);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("PoolNoodle: failed to ready instance: {}", id);
                        warn!("{:?}", e);
                        let _ = me.to_be_cleaned.push(id);
                    }
                }
            }
            sleep(Duration::from_millis(1)).await;
        }
    }

    async fn handle_clean(me: Arc<PoolNoodleInner<I, S>>, stop: Arc<AtomicBool>) {
        debug!("PoolNoodle: starting clean handler...");
        loop {
            if stop.load(Ordering::Relaxed) {
                debug!("PoolNoodle: received graceful shutdown signal, shutting down...");
                break;
            }

            // let's go clean some dead instances!
            if let Some(id) = me.to_be_cleaned.pop() {
                debug!("PoolNoodle: cleaning instance");
                match PoolNoodleInner::clean(id, &me.spec).await {
                    Ok(_) => {
                        debug!("PoolNoodle: instance cleaned: {}", id);
                        let _ = me.unprepared.push(id);
                    }
                    Err(e) => {
                        warn!("PoolNoodle: failed to clean: {}", id);
                        warn!("{}", e);
                        let _ = me.to_be_cleaned.push(id);
                    }
                };
            };
            sleep(Duration::from_millis(1)).await;
        }
    }

    async fn handle_drop(me: Arc<PoolNoodleInner<I, S>>, stop: Arc<AtomicBool>) {
        debug!("PoolNoodle: starting drop handler...");
        loop {
            if stop.load(Ordering::Relaxed) {
                debug!("PoolNoodle: received graceful shutdown signal, shutting down...");
                break;
            }

            // an instance has dropped, let's make sure we clean it up
            if let Ok(instance) = me.drop_rx.lock().await.try_recv() {
                debug!("PoolNoodle: drop message receieved");
                let _ = me.to_be_cleaned.push(instance.id());
            }
            sleep(Duration::from_millis(1)).await;
        }
    }

    /// This will attempt to get a ready, healthy instance from the pool.
    /// If there are no instances, it will give the main loop a chance to fill the pool and try
    /// again. It will throw an error if there are no available instances after enough retries.
    pub async fn get(&mut self) -> Result<LifeGuard<I>> {
        let me = Arc::clone(&self.0);

        let max_retries = 300; // Set the maximum number of retries
        let mut retries = 0;
        loop {
            if retries >= max_retries {
                return Err(PoolNoodleError::ExecutionPoolStarved);
            }
            if !me.ready.is_empty() {
                if let Some(mut instance) = me.ready.pop() {
                    debug!("PoolNoodle: got instance: {}", instance.id());
                    // Try to ensure the item is healthy
                    match &mut instance.ensure_healthy().await {
                        Ok(_) => {
                            info!(
                                "PoolNoodle: got instance for func execution: {}",
                                &instance.id()
                            );
                            let drop_tx = me.drop_tx.clone();
                            return Ok(LifeGuard {
                                drop_tx,
                                item: Some(instance),
                            });
                        }
                        Err(_) => {
                            debug!("PoolNoodle: not healthy, cleaning up and getting a new one.");
                            drop(instance);
                        }
                    }
                };
            } else {
                retries += 1;
                debug!(
                    "Failed to get from pool, retry ({} of {})",
                    retries, max_retries
                );
                sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

#[derive(Debug)]
struct PoolNoodleInner<I, S>
where
    S: Spec,
{
    pool_size: u32,
    spec: S,
    ready: ArrayQueue<I>,
    to_be_cleaned: ArrayQueue<u32>,
    unprepared: ArrayQueue<u32>,
    shutdown_rx: Mutex<tokio::sync::broadcast::Receiver<()>>,
    drop_tx: Sender<I>,
    drop_rx: Mutex<Receiver<I>>,
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

    /// This outputs the current state of the pool
    pub async fn stats(&self) -> PoolNoodleStats {
        PoolNoodleStats {
            pool_size: self.pool_size as usize,
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
    /// Total number of instances currently running and able to accept work
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
            "PoolNoodle Stats -- pool size: {}, ready: {}, to be cleaned: {}, unprepared: {}",
            self.pool_size, self.ready, self.to_be_cleaned, self.unprepared
        )
    }
}

/// LifeGuard is a wrapper for instances that come from the pool.
/// It is carries a Sender and implements Drop. When an instance goes out of
/// scope, it lets PoolNoodle know that the instance needs to be cleaned up.
#[derive(Debug)]
pub struct LifeGuard<I>
where
    I: Instance + Send + Sync,
{
    drop_tx: Sender<I>,
    item: Option<I>,
}

impl<I> Drop for LifeGuard<I>
where
    I: Instance + Send + Sync,
{
    fn drop(&mut self) {
        debug!("PoolNoodle: dropping instance");
        let item = self
            .item
            .take()
            .expect("Item must be present as it is initialized with Some and never replaced.");
        if futures::executor::block_on(self.drop_tx.send(item)).is_err() {
            warn!("Failed to send drop message for an instance. It will not be cleaned up!");
        };
    }
}

impl<I> std::ops::Deref for LifeGuard<I>
where
    I: Instance + Send + Sync,
{
    type Target = I;

    fn deref(&self) -> &I {
        self.item
            .as_ref()
            .expect("Item must be present as it is initialized with Some and never replaced.")
    }
}

impl<I> std::ops::DerefMut for LifeGuard<I>
where
    I: Instance + Send + Sync,
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
        pool.start();

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
