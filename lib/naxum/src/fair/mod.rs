// Fair scheduling for consuming from multiple NATS JetStream consumers.

mod config;
mod listener;
mod scheduler;
mod stream;

pub use config::FairSchedulingConfig;
pub use listener::{
    TaskListenerError,
    spawn_task_listener,
};
pub use scheduler::Scheduler;
pub use stream::{
    FairSchedulerError,
    FairSchedulerStream,
    KeyReady,
};
