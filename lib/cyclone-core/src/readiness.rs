use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("failed to parse '{0}' into ReadinessStatus")]
pub struct ReadinessStatusParseError(String);

#[remain::sorted]
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ReadinessStatus {
    Ready,
}

impl ReadinessStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ReadinessStatus::Ready => "ready\n",
        }
    }
}

impl From<ReadinessStatus> for &'static str {
    fn from(value: ReadinessStatus) -> Self {
        value.as_str()
    }
}

impl FromStr for ReadinessStatus {
    type Err = ReadinessStatusParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "ready" => Ok(Self::Ready),
            invalid => Err(ReadinessStatusParseError(invalid.to_string())),
        }
    }
}
