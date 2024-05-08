use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use tokio::time::sleep;

use std::sync::Arc;
use tokio::sync::Mutex;

use std::sync::mpsc::{self, Receiver, Sender};
use tokio::time::Duration;

use tracing::{debug, info, trace, warn};

use std::{collections::VecDeque, result};
use thiserror::Error;

use crate::{Instance, Spec};

type Result<T> = result::Result<T, PoolNoodleError>;
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

/// Error type for [`PoolNoodle`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PoolNoodleError {
    /// Failed to get a new instance ID.
    #[error("Failed to get a new instance from the execution pool!")]
    ExecutionPoolStarved,
    /// Failed to clean an instance.
    #[error("Failed to clean the instance: {0}")]
    InstanceClean(String),
    /// Failed to prepare a new instance.
    #[error("Failed to prepare the instance: {0}")]
    InstancePrepare(String),
    /// Failed to spawn a new instance.
    #[error("Failed to spawn the instance: {0}")]
    InstanceSpawn(String),
    /// Failed to terminate an instance.
    #[error("Failed to terminate the instance: {0}")]
    InstanceTerminate(String),
}

/// Pool Noodle is a tool for ensuring that we maintain a bare minimum number of Firecracker Jails
/// for function execution. We wrap it in an Arc Mutex so we can update the queues it manages
/// across threads.
#[derive(Debug)]
pub struct PoolNoodle<I, S: Spec>(Arc<Mutex<PoolNoodleInner<I, S>>>);

impl<I, S: Spec> Clone for PoolNoodle<I, S> {
    fn clone(&self) -> Self {
        PoolNoodle(self.0.clone())
    }
}

///
#[derive(Debug)]
struct PoolNoodleInner<I, S>
where
    S: Spec,
{
    pool_size: u32,
    spec: S,
    ready: Vec<I>,
    to_be_cleaned: VecDeque<u32>,
    to_be_terminated: VecDeque<I>,
    unprepared: VecDeque<u32>,
    drop_tx: Sender<I>,
    drop_rx: Receiver<I>,
}

impl<B, I, E, S> PoolNoodleInner<I, S>
where
    S: Spec<Error = E, Instance = I> + Send + Sync + 'static,
    I: Instance<SpecBuilder = B, Error = E> + Send + Sync + 'static,
    E: Send + Display,
{
    async fn clean(id: u32, spec: &mut S) -> Result<()> {
        spec.clean(id)
            .await
            .map_err(|e| PoolNoodleError::InstanceClean(e.to_string()))
    }

    async fn prepare(id: u32, spec: &mut S) -> Result<()> {
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
    async fn terminate(mut instance: I) -> Result<()> {
        instance
            .terminate()
            .await
            .map_err(|e| PoolNoodleError::InstanceTerminate(e.to_string()))
    }

    /// This outputs the current state of the pool
    pub async fn stats(&mut self) -> PoolNoodleStats {
        PoolNoodleStats {
            pool_size: self.pool_size as usize,
            ready: self.ready.len(),
            to_be_cleaned: self.to_be_cleaned.len(),
            to_be_terminated: self.to_be_terminated.len(),
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
    /// Total number of instances that need to be terminated
    pub to_be_terminated: usize,
    /// Total number of unclaimed instances waiting to be readied
    pub unprepared: usize,
}

impl Display for PoolNoodleStats {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "PoolNoodle Stats -- pool size: {}, ready: {}, to be cleaned: {}, to be terminated: {}, unprepared: {}",
                   self.pool_size,
                   self.ready,
                   self.to_be_cleaned,
                   self.to_be_terminated,
                   self.unprepared)
    }
}

/// LifeGuard is a wrapper for instances that come from the pool.
/// It is carries a Sender and implements Drop. When an instance gotten from the pool goes out of
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
        if self.drop_tx.send(item).is_err() {
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

impl<B, I, E, S> PoolNoodle<I, S>
where
    S: Spec<Error = E, Instance = I> + Send + Sync + 'static,
    I: Instance<SpecBuilder = B, Error = E> + Send + Sync + 'static,
    E: Send + Display,
{
    /// Creates a new instance of PoolNoodle
    pub fn new(pool_size: u32, spec: S) -> Self {
        let (drop_tx, drop_rx) = mpsc::channel();
        PoolNoodle(Arc::new(
            PoolNoodleInner {
                pool_size,
                spec,
                ready: Vec::new(),
                to_be_cleaned: VecDeque::new(),
                to_be_terminated: VecDeque::new(),
                unprepared: VecDeque::from_iter(0..pool_size),
                drop_tx,
                drop_rx,
            }
            .into(),
        ))
    }

    /// Gets the current pool stats from the inner struct
    /// Only used for tests
    pub async fn stats(&self) -> PoolNoodleStats {
        self.0.lock().await.stats().await
    }

    /// Starts the loop responsible for instance lifetimes. The loop works by:
    /// 1. Check if we have fewer ready instances than `[pool_size]`
    /// 2. If so, go get an unprepared instance and prepare it!
    /// 3. If not, let's go see if any of our instances have dropped.
    /// 4. If so, move them to `[to_be_terminated]` so they can be terminated.
    /// 3. If not, check if there are any instances that need to be terminated.
    /// 4. If so, terminate them and move them to `[to_be_cleaned]` so they can be cleaned.
    /// 3. If not, check if there are any instances that need to be cleaned.
    /// 4. If so, terminate them and move them to `[unprepared]` so they can be made ready.
    /// 9. If not, do nothing and loop again!
    ///
    #[allow(clippy::let_underscore_future)] // This needs to just run in the background forever.
    pub fn start(&mut self) {
        let me = Arc::clone(&self.0);

        let _ = tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(100)).await;

                let mut me = me.lock().await;
                let stats = me.stats().await;

                trace!("{}", stats);

                // we're at fewer than the desired pool, let's make more instances
                if stats.unprepared != 0 {
                    if let Some(id) = me.unprepared.pop_back() {
                        debug!("PoolNoodle: readying instance");
                        match PoolNoodleInner::prepare(id, &mut me.spec).await {
                            Ok(_) => {
                                debug!("PoolNoodle: instance readied: {}", id);
                                match PoolNoodleInner::spawn(id, &me.spec).await {
                                    Ok(instance) => {
                                        debug!("PoolNoodle: instance started: {}", id);
                                        me.ready.push(instance);
                                    }
                                    Err(e) => {
                                        warn!("PoolNoodle: failed to start instance: {}", id);
                                        warn!("{:?}", e);
                                        me.to_be_cleaned.push_front(id);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("PoolNoodle: failed to ready instance: {}", id);
                                warn!("{:?}", e);
                                me.to_be_cleaned.push_front(id);
                            }
                        }
                    }

                // an instance has dropped, let's make sure we clean it up
                } else if let Ok(instance) = me.drop_rx.try_recv() {
                    debug!("PoolNoodle: drop message receieved");
                    me.to_be_cleaned.push_front(instance.id());

                // let's go terminate some finished instances!
                } else if stats.to_be_terminated != 0 {
                    // go get a instance that needs to be terminated
                    if let Some(instance) = me.to_be_terminated.pop_back() {
                        debug!("PoolNoodle: terminating instance");
                        let id = instance.id();
                        // attempt to terminate it
                        match PoolNoodleInner::<I, S>::terminate(instance).await {
                            // it worked!
                            Ok(_) => {
                                debug!("PoolNoodle: instance terminated: {}", id);
                                me.to_be_cleaned.push_front(id);
                            }
                            // it did not work. We should move on to a different instance.
                            Err(e) => {
                                warn!("PoolNoodle: failed to terminate: {}", id);
                                warn!("{}", e);
                                me.to_be_cleaned.push_front(id);
                            }
                        };
                    }

                // let's go clean some dead instances!
                } else if stats.to_be_cleaned != 0 {
                    // go get a instance that needs to be cleaned
                    if let Some(id) = me.to_be_cleaned.pop_back() {
                        debug!("PoolNoodle: cleaning instance");
                        // attempt to clean it
                        match PoolNoodleInner::clean(id, &mut me.spec).await {
                            // it worked!
                            Ok(_) => {
                                debug!("PoolNoodle: instance cleaned: {}", id);
                                me.unprepared.push_front(id);
                            }
                            // it did not work. We should move on to a different instance.
                            Err(e) => {
                                warn!("PoolNoodle: failed to clean terminated: {}", id);
                                warn!("{}", e);
                                me.to_be_cleaned.push_front(id);
                            }
                        };
                    };
                }

                // make sure we drop the lock before looping again
                drop(me);
            }
        });
    }

    /// This will attempt to get an ready, healthy instance from the pool.
    /// If there are no instances, it will give the main loop a chance to fill the pool and try
    /// again. It will throw an error if there are no available instances after enough retries.
    pub async fn get(&mut self) -> Result<LifeGuard<I>> {
        let max_retries = 300; // Set the maximum number of retries
        let mut retries = 0;

        info!("PoolNoodle: getting instance for func execution");
        loop {
            if retries >= max_retries {
                return Err(PoolNoodleError::ExecutionPoolStarved);
            }
            let mut me = self.0.lock().await;
            if let Some(mut item) = me.ready.pop() {
                debug!("PoolNoodle: got instance: {}", item.id());
                // Try to ensure the item is healthy
                match &mut item.ensure_healthy().await {
                    Ok(_) => {
                        let drop_tx = me.drop_tx.clone();
                        return Ok(LifeGuard {
                            drop_tx,
                            item: Some(item),
                        });
                    }
                    Err(err) => {
                        debug!(
                            "PoolNoodle: not healthy, cleaning up and getting a new one: {}",
                            err
                        );
                        // Item will be dropped, we need to try again
                    }
                }
            }
            retries += 1;
            debug!(
                "Failed to get from pool, retry ({} of {})",
                retries, max_retries
            );
            // drop lock to let other thread continue
            drop(me);
            sleep(Duration::from_millis(101)).await;
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::instance::SpecBuilder;
    use async_trait::async_trait;
    use derive_builder::Builder;
    use tokio::time::{sleep, Duration};

    use super::*;

    pub struct DummyInstance {}

    pub struct DummyInstanceSpec {}
    #[async_trait]
    impl Spec for DummyInstanceSpec {
        type Instance = DummyInstance;
        type Error = DummyInstanceError;

        async fn clean(&mut self, _id: u32) -> result::Result<(), Self::Error> {
            Ok(())
        }
        async fn prepare(&mut self, _id: u32) -> result::Result<(), Self::Error> {
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

        let mut pool = PoolNoodle::new(3, spec);
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
