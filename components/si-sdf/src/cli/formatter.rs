use crate::cli::client::{CliMessage, ClientResult};
use crate::models::OutputLineStream;

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

#[derive(Debug)]
pub struct SimpleFormatter;

impl SimpleFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Formatter for SimpleFormatter {
    fn process_message(&mut self, message: CliMessage) -> ClientResult<()> {
        match message {
            CliMessage::Event(event) => {
                println!("--- {} ({})", event.message, event.start_timestamp);
            }
            CliMessage::EventLog(event_log) => {
                println!("  - {}", event_log.message);
            }
            CliMessage::OutputLine(output_line) => match output_line.stream {
                OutputLineStream::Stdout | OutputLineStream::All => {
                    println!("        {}", output_line.line)
                }
                OutputLineStream::Stderr => eprintln!("!!!     {}", output_line.line),
            },
        };
        Ok(())
    }
}
