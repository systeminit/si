use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("failed to parse '{0}' into LivenessStatus")]
pub struct LivenessStatusParseError(String);

#[remain::sorted]
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum LivenessStatus {
    Ok,
}

impl LivenessStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            LivenessStatus::Ok => "ok\n",
        }
    }
}

impl From<LivenessStatus> for &'static str {
    fn from(value: LivenessStatus) -> Self {
        value.as_str()
    }
}

impl FromStr for LivenessStatus {
    type Err = LivenessStatusParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "ok" => Ok(Self::Ok),
            invalid => Err(LivenessStatusParseError(invalid.to_string())),
        }
    }
}
