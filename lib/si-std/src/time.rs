use std::time::Duration;

/// Returns a [`Duration`] with a small amount of randomness which is not greater than the given Duration.
///
// Note: Jitter implementation thanks to the `fure` crate, released under the MIT license.
//
// See: https://github.com/Leonqn/fure/blob/8945c35655f7e0f6966d8314ab21a297181cc080/src/backoff.rs#L44-L51
pub fn jitter_duration(duration: Duration) -> Duration {
    let jitter = rand::random::<f64>();
    let secs = ((duration.as_secs() as f64) * jitter).ceil() as u64;
    let nanos = ((f64::from(duration.subsec_nanos())) * jitter).ceil() as u32;
    Duration::new(secs, nanos)
}
