use chrono::Utc;

pub fn timestamp() -> u64 {
    u64::try_from(std::cmp::max(Utc::now().timestamp(), 0)).expect("timestamp not be negative")
}
