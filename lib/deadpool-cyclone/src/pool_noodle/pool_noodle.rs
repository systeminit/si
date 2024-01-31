use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio::{process::Command, time::Instant};
use tracing::{debug, trace, warn};

use std::{collections::VecDeque, result};
use thiserror::Error;
use tokio::time;

const MINUMUM_JAIL_PERCENTAGE: f32 = 0.25;
const CYCLONE_EXECUTION_TIMEOUT: u64 = 3600;

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
    /// Failed to clean a jail.
    #[error("Failed to clean the jail")]
    CleanJail,
    /// Failed to get a new jail ID.
    #[error("Failed to get a new jail from the execution pool!")]
    ExecutionPoolStarved,
    /// Failed to prepare a new jail.
    #[error("Failed to prepare the jail")]
    PrepareJail,
    /// Failed to set a jail to be cleaned.
    #[error("Failed to set a jail to be cleaned.")]
    SetClean,
}

/// Pool Noodle is a tool for ensuring that we maintain a bare minimum number of Firecracker Jails
/// for function execution. We wrap it in an Arc Mutex so we can update the queues it manages
/// across threads.
#[derive(Debug, Clone)]
pub struct PoolNoodle(pub Arc<Mutex<PoolNoodleInner>>);

/// Inner struct to excpsulate the queues of jails in different states.
///
/// pool_size: the total number of jails we want to manage
/// ready: jails that can currently be used to run functions
/// to_be_cleaned: jails that have been used and must be cleaned up
/// unprepared: jails that are available to be prepared and moved into a ready state
#[derive(Debug)]
pub struct PoolNoodleInner {
    pool_size: u32,
    active: BTreeMap<u32, Instant>,
    ready: Vec<u32>,
    to_be_cleaned: VecDeque<u32>,
    unprepared: VecDeque<u32>,
}

impl Default for PoolNoodle {
    fn default() -> Self {
        Self::new(0)
    }
}

impl PoolNoodle {
    /// Creates a new instance of PoolNoodle
    pub fn new(pool_size: u32) -> Self {
        PoolNoodle(Arc::new(
            PoolNoodleInner {
                pool_size,
                active: BTreeMap::new(),
                ready: Vec::new(),
                to_be_cleaned: VecDeque::new(),
                unprepared: VecDeque::from_iter(0..pool_size),
            }
            .into(),
        ))
    }

    /// Starts the loop responsible for jail lifetimes. The loop works by:
    /// 1. Check if we have fewer ready jails than the `[MINUMUM_JAIL_PERCENTAGE]` of `[pool_size]`
    /// 2. If so, go get an unprepared jail and prepare it!
    /// 3. If not, check if there are any jails that need to be cleaned.
    /// 4. If so, clean them and move them to `[unprepared]` so they can be made ready.
    /// 5. If not, let's go check if our oldest active jail is older than the timeout
    /// 6. If so, push it to to_be_cleaned
    /// 7. If not, do nothing!
    ///
    /// todo(scott): this is a brute force approach. I deally moving this to be event driving and
    /// talking over channels will lets us simplify the cross-thread vec fun and the forver-looping
    /// future that we just let run rampant.
    #[allow(clippy::let_underscore_future)] // This needs to just run in the background forever.
    pub fn start(&self) {
        let me = Arc::clone(&self.0);
        let _ = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(100));

            loop {
                interval.tick().await;

                let mut me = me.lock().await;
                let active_len = me.active.len();
                let ready_len = me.ready.len() as f32;
                let to_be_cleaned_len = me.to_be_cleaned.len();
                let unprepared_len = me.unprepared.len();
                let target = me.pool_size as f32 * MINUMUM_JAIL_PERCENTAGE;

                trace!(
                    "PoolNoodle Stats -- desired ready: {}, ready: {}, active: {}, to be cleaned: {}, unprepared: {}",
                    target,
                    ready_len,
                    active_len,
                    to_be_cleaned_len,
                    unprepared_len,
                );

                // we're at fewer than 25% of the total, let's make more jails
                if ready_len < target && unprepared_len != 0 {
                    if let Some(id) = me.unprepared.pop_back() {
                        match PoolNoodle::prepare_jail(id).await {
                            Ok(_) => {
                                debug!("PoolNoodle: jail readied: {}", id);
                                me.ready.push(id);
                            }
                            Err(_) => {
                                warn!("PoolNoodle: failed to ready jail: {}", id);
                                me.unprepared.push_front(id);
                            }
                        }
                    };

                // let's go clean some dead jails!
                } else if to_be_cleaned_len != 0 {
                    // go get a jail that needs to be cleaned
                    if let Some(id) = me.to_be_cleaned.pop_back() {
                        // attempt to clean it
                        match PoolNoodle::clean_jail(id).await {
                            // it worked!
                            Ok(_) => {
                                debug!("PoolNoodle: jail cleaned: {}", id);
                                me.unprepared.push_back(id);
                                // this jail should no longer be active, so let's make sure we
                                // remove it
                                me.active.remove(&id);
                            }
                            // it did not work. We should move on to a different jail. This one will be
                            // abandoned.
                            Err(_) => {
                                warn!("PoolNoodle: failed to clean jail: {}", id);
                            }
                        };
                    };

                // let's go see if any of our active jails have timed out
                } else if active_len != 0 {
                    // peak at the top jail to see if it needs to be cleaned
                    if let Some((id, start_time)) = me.active.last_key_value() {
                        let elapsed = start_time.elapsed();
                        if elapsed >= Duration::from_secs(CYCLONE_EXECUTION_TIMEOUT) {
                            debug!(
                                "PoolNoodle: jail active for {:?}, more than timeout of {}s: {}",
                                elapsed, CYCLONE_EXECUTION_TIMEOUT, id
                            );
                            if let Some((id, _)) = me.active.pop_last() {
                                me.set_as_to_be_cleaned(id).await;
                            }
                        }
                    };
                } else {
                    continue;
                };
            }
        });
    }

    /// This readies a jail. This script is place in the correct location during Veritech startup.
    /// todo(scott): This method should be replace with a Rust-native implementation.
    async fn prepare_jail(id: u32) -> Result<()> {
        let command = String::from("/firecracker-data/prepare_jailer.sh");
        let _status = Command::new("sudo")
            .arg(command)
            .arg(id.to_string())
            .status()
            .await
            .map_err(|_| PoolNoodleError::PrepareJail)?;

        Ok(())
    }

    /// This cleans a jail. This script is place in the correct location during Veritech startup.
    /// todo(scott): This method should be replace with a Rust-native implementation. This could
    /// also be made more efficient by only replacing the rootfs instead of cleaning everything.
    async fn clean_jail(id: u32) -> Result<()> {
        let command = String::from("/firecracker-data/stop.sh");
        let _status = Command::new("sudo")
            .arg(command)
            .arg(id.to_string())
            .output()
            .await
            .map_err(|_| PoolNoodleError::CleanJail)?;

        Ok(())
    }
}

impl PoolNoodleInner {
    /// This pops a ready jail from the stack and returns its Id so it can be executed.
    /// If there are no ready jails, it will retry until it either gets one or retries out.
    pub async fn get_ready_jail(&mut self) -> Result<u32> {
        let mut retries = 30;
        loop {
            trace!("PoolNoodle: getting a ready jail.");
            match self.ready.pop() {
                Some(id) => {
                    debug!("PoolNoodle: got ready jail: {}", id);
                    break Ok(id);
                }
                None => {
                    warn!("PoolNoodle: execution pool starved! Trying again.");
                }
            }

            if retries < 1 {
                return Err(PoolNoodleError::ExecutionPoolStarved);
            }
            retries -= 1;
            time::sleep(Duration::from_millis(1000)).await;
        }
    }

    /// This marks a jail as active
    pub async fn set_as_active(&mut self, id: u32, start_time: Instant) -> Result<Instant> {
        self.active
            .insert(id, start_time)
            .ok_or(PoolNoodleError::ExecutionPoolStarved)
    }

    /// This marks a jail as needing to be cleaned
    pub async fn set_as_to_be_cleaned(&mut self, id: u32) {
        self.to_be_cleaned.push_front(id)
    }
}
