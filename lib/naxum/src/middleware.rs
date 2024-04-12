pub mod ack;
pub mod delay;
pub mod trace;

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum LatencyUnit {
    Seconds,
    Millis,
    Micros,
    Nanos,
}
