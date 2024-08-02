//! A variant of a Snowflake ID, customized for System Initiative.

use lazy_static::lazy_static;
use std::env;
use std::sync::OnceLock;

use snowdon::{ClassicLayout, Epoch, Generator, MachineId, Snowflake};

pub struct SnowflakeParams;

impl Epoch for SnowflakeParams {
    fn millis_since_unix() -> u64 {
        // Thursday, August 1, 2024, 2:29:38 AM GMT
        1722479378
    }
}

impl MachineId for SnowflakeParams {
    fn machine_id() -> u64 {
        let machine_id = MACHINE_ID.get_or_init(|| {
            let machine_id = env::var("MACHINE_ID")
                .expect("You must set the MACHINE_ID to a unique number between 1-1024.")
                .parse()
                .expect("MACHINE_ID set, but is not a number");
            if machine_id < 1 || machine_id > 1024 {
                panic!("MACHINE_ID is not between 1 and 1024: {0}", machine_id);
            }
            machine_id
        });
        *machine_id
    }
}

pub type SiId = Snowflake<ClassicLayout<SnowflakeParams>, SnowflakeParams>;
pub type SiIdGenerator = Generator<ClassicLayout<SnowflakeParams>, SnowflakeParams>;

lazy_static! {
    static ref SI_ID_GENERATOR: SiIdGenerator = SiIdGenerator::default();
    static ref MACHINE_ID: OnceLock<u64> = OnceLock::new();
}

mod server;
mod client;

pub use server::{run_server, ServerError};
pub use client::{SiIdClient, ClientError};
