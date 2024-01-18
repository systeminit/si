use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tracing::{debug, trace, warn};

use std::{collections::VecDeque, result};
use thiserror::Error;
use tokio::time;

const MINUMUM_JAIL_PERCENTAGE: f32 = 0.25;

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
/// Pool Noodle is a tool for ensuring that we maintain a bare minimum number of Firecracker Jails
/// for function execution. We wrap it in an Arc Mutex so we can update the queues it manages
/// across threads.
pub struct PoolNoodle(Arc<Mutex<PoolNoodleInner>>);

/// Error type for [`PoolNoodle`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PoolNoodleError {
    /// Failed to clean a jail.
    #[error("Failed to clean the jail")]
    CleanJail,
    /// Failed to get a new jail ID.
    #[error("Failed to get a new jail ID.")]
    GetJail,
    /// Failed to prepare a new jail.
    #[error("Failed to prepare the jail")]
    PrepareJail,
    /// Failed to set a jail to be cleaned.
    #[error("Failed to set a jail to be cleaned.")]
    SetClean,
}

/// Inner struct to excpsulate the queues of jails in different states.
///
/// pool_size: the total number of jails we want to manage
/// ready: jails that can currently be used to run functions
/// to_be_cleaned: jails that have been used and must be cleaned up
/// unprepared: jails that are available to be prepared and moved into a ready state
pub struct PoolNoodleInner {
    pool_size: u32,
    ready: Vec<u32>,
    to_be_cleaned: VecDeque<u32>,
    unprepared: VecDeque<u32>,
}

impl PoolNoodle {
    /// Creates a new instance of PoolNoodle
    pub fn new(pool_size: u32) -> Self {
        PoolNoodle(Arc::new(
            PoolNoodleInner {
                pool_size,
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
    /// 5. If not, do nothing!
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
                let ready_len = me.ready.len() as f32;
                let to_be_cleaned_len = me.to_be_cleaned.len();
                let unprepared_len = me.unprepared.len();
                let target = me.pool_size as f32 * MINUMUM_JAIL_PERCENTAGE;

                trace!(
                    "PoolNoodle Stats:\ndesired ready: {}\nready: {}\nto be cleaned: {}\nunprepared: {}",
                    target,
                    ready_len,
                    to_be_cleaned_len,
                    unprepared_len,
                );

                // we're at fewer than 25% of the total, let's make more jails
                if ready_len < target && unprepared_len != 0 {
                    debug!("PoolNoodle: readying jail");
                    let id = match me.unprepared.pop_back() {
                        Some(id) => id,
                        None => {
                            warn!("PoolNoodle: failed to pop_back() unprepared when it should not be empty!");
                            continue;
                        }
                    };

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
                // let's go clean some dead jails!
                } else if to_be_cleaned_len != 0 {
                    debug!("PoolNoodle: cleaning jails");
                    // go get a jail that needs to be cleaned
                    let id = match me.to_be_cleaned.pop_back() {
                        Some(id) => id,
                        None => {
                            warn!("PoolNoodle: failed to pop_back() to_be_cleaned when it should not be empty!");
                            continue;
                        }
                    };
                    // attempt to clean it
                    match PoolNoodle::clean_jail(id).await {
                        // it worked!
                        Ok(_) => {
                            debug!("PoolNoodle: jail {} cleaned", id);
                            me.unprepared.push_back(id);
                        }
                        // it did not work. We should move on to a different jail. This one will be
                        // abandoned.
                        Err(_) => {
                            warn!("PoolNoodle: failed to clean jail {}", id);
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
            .status()
            .await
            .map_err(|_| PoolNoodleError::CleanJail)?;

        Ok(())
    }

    /// This pops a ready jail from the stack and returns its Id so it can be executed
    pub async fn get_ready_jail(&mut self) -> Result<u32> {
        self.0
            .lock()
            .await
            .ready
            .pop()
            .ok_or(PoolNoodleError::GetJail)
    }

    /// This marks a jail as needing to be cleaned
    pub async fn set_as_to_be_cleaned(&mut self, id: u32) {
        self.0.lock().await.to_be_cleaned.push_front(id)
    }
}
