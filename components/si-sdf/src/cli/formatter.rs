use crate::cli::client::{CliMessage, ClientResult};

pub trait Formatter {
    fn process_message(&mut self, message: CliMessage) -> ClientResult<()>;
}

#[derive(Debug)]
pub struct DebugFormatter;

impl DebugFormatter {
    pub fn new() -> DebugFormatter {
        DebugFormatter {}
    }
}

impl Formatter for DebugFormatter {
    fn process_message(&mut self, message: CliMessage) -> ClientResult<()> {
        dbg!(message);
        Ok(())
    }
}
