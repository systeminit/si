//! Adaptive rate limiting with exponential backoff.
//!
//! # Overview
//!
//! The rate limiter implements exponential backoff that adapts to S3 throttling patterns:
//!
//! - **On throttle:** Delay increases exponentially
//! - **After consecutive successes:** Delay decreases gradually
//! - **Below Zeno threshold:** Delay resets to zero (solves dichotomy paradox)
//!
//! # State Machine
//!
//! ```text
//!             ┌─────────────┐
//!             │ Zero Delay  │
//!             └──────┬──────┘
//!                    │
//!          ┌─────────▼─────────┐
//!          │  Throttle (503)?  │
//!          └─────┬───────┬─────┘
//!                │       │
//!            Yes │       │ No (Success)
//!                │       │
//!      ┌─────────▼───┐   │
//!      │ Increase    │   │
//!      │ Backoff     │   │
//!      └─────┬───────┘   │
//!            │           │
//!            │     ┌─────▼──────────┐
//!            │     │ Increment      │
//!            │     │ Success Streak │
//!            │     └─────┬──────────┘
//!            │           │
//!            │     ┌─────▼──────────┐
//!            │     │ N Successes?   │
//!            │     └─────┬────┬─────┘
//!            │           │    │
//!            │       Yes │    │ No
//!            │           │    │
//!            │     ┌─────▼────▼─────┐
//!            │     │ Reduce Backoff │
//!            │     │ Reset Streak   │
//!            │     └────────┬───────┘
//!            │              │
//!            └──────────────┘
//! ```
//!
//! # Tuning Guidelines
//!
//! ## Aggressive Recovery
//!
//! Ramp up quickly after throttling resolves:
//!
//! - Lower `successes_before_reduction` (e.g., 2)
//! - Higher `success_divisor` (e.g., 2.0)
//!
//! **Trade-off:** May overshoot if S3 only marginally recovered
//!
//! ## Conservative Recovery
//!
//! Validate stability at each rate level:
//!
//! - Higher `successes_before_reduction` (e.g., 5)
//! - Lower `success_divisor` (e.g., 1.2)
//!
//! **Trade-off:** Stays throttled longer than necessary
//!
//! ## Faster Initial Backoff
//!
//! Reach max delay quickly during sustained throttling:
//!
//! - Higher `backoff_multiplier` (e.g., 3.0)
//!
//! **Trade-off:** May back off too aggressively on transient issues
//!
//! ## Slower Initial Backoff
//!
//! Take longer to reach max delay:
//!
//! - Lower `backoff_multiplier` (e.g., 1.5)
//!
//! **Trade-off:** More throttle attempts before backing off sufficiently
//!
//! # Example Configurations
//!
//! **Stable workload (default):**
//! ```json
//! {
//!   "backoff_multiplier": 2.0,
//!   "success_divisor": null,  // auto: 1.5
//!   "successes_before_reduction": 3
//! }
//! ```
//!
//! **Bursty workload (conservative):**
//! ```json
//! {
//!   "backoff_multiplier": 2.0,
//!   "success_divisor": 1.2,
//!   "successes_before_reduction": 5
//! }
//! ```
//!
//! **High-volume workload (aggressive):**
//! ```json
//! {
//!   "backoff_multiplier": 3.0,
//!   "success_divisor": 2.0,
//!   "successes_before_reduction": 2
//! }
//! ```

use std::time::Duration;

use serde::{
    Deserialize,
    Serialize,
};

/// Errors that can occur when validating RateLimitConfig
#[derive(Debug, Clone, thiserror::Error)]
pub enum RateLimitConfigError {
    #[error("min_delay_ms ({0}) cannot be greater than max_delay_ms ({1})")]
    MinGreaterThanMax(u64, u64),

    #[error("backoff_multiplier ({0}) must be greater than 1.0")]
    MultiplierTooSmall(f32),

    #[error("success_divisor ({0}) must be greater than 1.0")]
    DivisorTooSmall(f32),

    #[error("successes_before_reduction cannot be zero")]
    SuccessesZero,
}

/// Configuration for adaptive rate limiting with exponential backoff.
///
/// This configuration defines the parameters for an adaptive rate limiter that:
/// - Exponentially increases delays when throttled
/// - Gradually reduces delays after consecutive successes
/// - Prevents getting stuck near zero delay (Zeno's paradox solution)
///
/// # Example
///
/// ```
/// use si_layer_cache::rate_limiter::RateLimitConfig;
///
/// let config = RateLimitConfig {
///     min_delay_ms: 0,
///     max_delay_ms: 5000,
///     initial_backoff_ms: 100,
///     backoff_multiplier: 2.0,
///     success_divisor: None,  // Auto-calculated as backoff_multiplier * 0.75
///     zeno_threshold_ms: 50,
///     successes_before_reduction: 3,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Minimum delay to maintain between operations (in milliseconds).
    ///
    /// Even after successful operations, the delay will not be reduced below this value.
    /// Set to 0 to allow full reduction to zero delay.
    pub min_delay_ms: u64,

    /// Maximum delay cap for exponential backoff (in milliseconds).
    ///
    /// When throttled, delays grow exponentially but will never exceed this value.
    /// This prevents indefinite waiting during sustained throttling.
    pub max_delay_ms: u64,

    /// Initial delay applied on the first throttling event (in milliseconds).
    ///
    /// When transitioning from zero delay to backoff, this value is used as the
    /// starting point for exponential growth.
    pub initial_backoff_ms: u64,

    /// Multiplier for exponential backoff growth.
    ///
    /// On each throttling event, the current delay is multiplied by this factor.
    /// Typical values: 2.0 (doubling), 1.5 (50% increase).
    pub backoff_multiplier: f32,

    /// Divisor for reducing delay after consecutive successes.
    ///
    /// When `None`, auto-calculated as `backoff_multiplier * 0.75` to provide
    /// asymmetric behavior (fast increase, slower decrease for stability).
    /// Explicit values allow fine-tuning the recovery rate.
    pub success_divisor: Option<f32>,

    /// Threshold below which delays jump directly to zero (in milliseconds).
    ///
    /// Solves Zeno's paradox: prevents getting stuck with tiny delays that never
    /// quite reach zero. When the delay falls below this threshold, it's reset
    /// to zero instead of continuing to divide.
    pub zeno_threshold_ms: u64,

    /// Number of consecutive successes required before reducing backoff delay.
    ///
    /// This validation period ensures the system is stable at the current rate
    /// before speeding up. Higher values provide more conservative recovery.
    pub successes_before_reduction: u32,
}

impl RateLimitConfig {
    /// Returns the effective success divisor: configured value or auto-calculated.
    /// Auto-calculation: backoff_multiplier * 0.75
    pub fn success_divisor(&self) -> f32 {
        self.success_divisor
            .unwrap_or(self.backoff_multiplier * 0.75)
    }

    /// Validates the configuration parameters.
    ///
    /// Returns an error if any of the following conditions are violated:
    /// - `min_delay_ms` must be less than or equal to `max_delay_ms`
    /// - `backoff_multiplier` must be greater than 1.0
    /// - `success_divisor` (if set) must be greater than 1.0
    /// - `successes_before_reduction` must be greater than 0
    pub fn validate(&self) -> Result<(), RateLimitConfigError> {
        if self.min_delay_ms > self.max_delay_ms {
            return Err(RateLimitConfigError::MinGreaterThanMax(
                self.min_delay_ms,
                self.max_delay_ms,
            ));
        }

        if self.backoff_multiplier <= 1.0 {
            return Err(RateLimitConfigError::MultiplierTooSmall(
                self.backoff_multiplier,
            ));
        }

        if let Some(divisor) = self.success_divisor {
            if divisor <= 1.0 {
                return Err(RateLimitConfigError::DivisorTooSmall(divisor));
            }
        }

        if self.successes_before_reduction == 0 {
            return Err(RateLimitConfigError::SuccessesZero);
        }

        Ok(())
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 0,
            max_delay_ms: 5000,
            initial_backoff_ms: 100,
            backoff_multiplier: 2.0,
            success_divisor: None,
            zeno_threshold_ms: 50,
            successes_before_reduction: 3,
        }
    }
}

/// Adaptive rate limiter with exponential backoff and gradual recovery.
///
/// Tracks the current delay state and consecutive success count to implement
/// adaptive rate limiting. The rate limiter responds to throttling by exponentially
/// increasing delays, and gradually reduces delays after consecutive successes.
///
/// # State Transitions
///
/// - **On throttle**: Delay increases exponentially, success streak resets
/// - **On success**: Success streak increments
/// - **After N successes**: Delay reduces (divides or resets to zero)
///
/// # Usage Pattern
///
/// ```no_run
/// use si_layer_cache::rate_limiter::{RateLimiter, RateLimitConfig};
/// use std::time::Duration;
/// use tokio::time::sleep;
///
/// # async fn example() {
/// let config = RateLimitConfig::default();
/// let mut limiter = RateLimiter::new(config);
///
/// loop {
///     // Apply current delay before operation
///     let delay = limiter.current_delay();
///     if delay > Duration::ZERO {
///         sleep(delay).await;
///     }
///
///     // Attempt operation
///     match perform_operation().await {
///         Ok(_) => {
///             limiter.on_success();
///
///             // Check if we should reduce backoff
///             if limiter.should_reduce_backoff() {
///                 limiter.reduce_backoff();
///             }
///         }
///         Err(throttle_error) => {
///             limiter.on_throttle();
///             // Will use increased delay on next iteration
///         }
///     }
/// }
/// # }
/// # async fn perform_operation() -> Result<(), String> { Ok(()) }
/// ```
pub struct RateLimiter {
    current_delay: Duration,
    consecutive_successes: u32,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Creates a new rate limiter with the given configuration.
    ///
    /// The limiter starts with zero delay and zero consecutive successes,
    /// representing an initial state with no throttling.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            current_delay: Duration::ZERO,
            consecutive_successes: 0,
            config,
        }
    }

    /// Returns the current delay to apply before the next operation.
    ///
    /// This delay should be applied (e.g., via `tokio::time::sleep`) before
    /// attempting the next operation. Returns `Duration::ZERO` when no delay
    /// is needed.
    pub fn current_delay(&self) -> Duration {
        self.current_delay
    }

    /// Returns the current count of consecutive successful operations.
    ///
    /// This counter increments on each success and resets to zero when:
    /// - A throttling event occurs
    /// - The backoff delay is reduced
    pub fn consecutive_successes(&self) -> u32 {
        self.consecutive_successes
    }

    /// Records a throttling event and increases the delay.
    ///
    /// Call this when the operation fails due to rate limiting (e.g., HTTP 503,
    /// S3 SlowDown error, etc.). This method:
    /// - Sets delay to `initial_backoff_ms` if currently zero
    /// - Otherwise multiplies current delay by `backoff_multiplier`
    /// - Respects `max_delay_ms` cap
    /// - Resets consecutive success counter to zero
    ///
    /// # Example
    ///
    /// ```
    /// use si_layer_cache::rate_limiter::{RateLimiter, RateLimitConfig};
    /// use std::time::Duration;
    ///
    /// let mut limiter = RateLimiter::new(RateLimitConfig::default());
    /// assert_eq!(limiter.current_delay(), Duration::ZERO);
    ///
    /// limiter.on_throttle();
    /// assert_eq!(limiter.current_delay(), Duration::from_millis(100)); // initial_backoff_ms
    /// ```
    pub fn on_throttle(&mut self) {
        let new_delay_ms = if self.current_delay == Duration::ZERO {
            self.config.initial_backoff_ms
        } else {
            let current_ms = self.current_delay.as_millis() as f32;
            let multiplied = (current_ms * self.config.backoff_multiplier) as u64;
            multiplied.min(self.config.max_delay_ms)
        };

        self.current_delay = Duration::from_millis(new_delay_ms);
        self.consecutive_successes = 0;
    }

    /// Records a successful operation.
    ///
    /// Call this when an operation completes successfully without throttling.
    /// Increments the consecutive success counter, which is used to determine
    /// when to reduce the backoff delay.
    ///
    /// After calling this method, check [`should_reduce_backoff()`](Self::should_reduce_backoff)
    /// to determine if the delay should be reduced via [`reduce_backoff()`](Self::reduce_backoff).
    ///
    /// # Example
    ///
    /// ```
    /// use si_layer_cache::rate_limiter::{RateLimiter, RateLimitConfig};
    ///
    /// let config = RateLimitConfig {
    ///     successes_before_reduction: 3,
    ///     ..Default::default()
    /// };
    /// let mut limiter = RateLimiter::new(config);
    ///
    /// limiter.on_success();
    /// limiter.on_success();
    /// limiter.on_success();
    ///
    /// assert!(limiter.should_reduce_backoff());
    /// ```
    pub fn on_success(&mut self) {
        self.consecutive_successes += 1;
    }

    /// Checks if the backoff delay should be reduced.
    ///
    /// Returns `true` when the consecutive success count reaches or exceeds
    /// `successes_before_reduction` from the configuration. This indicates
    /// the system has been stable at the current rate and can attempt to
    /// speed up.
    ///
    /// After this returns `true`, call [`reduce_backoff()`](Self::reduce_backoff)
    /// to actually reduce the delay and reset the success counter.
    pub fn should_reduce_backoff(&self) -> bool {
        self.consecutive_successes >= self.config.successes_before_reduction
    }

    /// Reduces the backoff delay after a success streak.
    ///
    /// This method should be called after [`should_reduce_backoff()`](Self::should_reduce_backoff)
    /// returns `true`. The reduction behavior depends on the current delay:
    ///
    /// - **Below `zeno_threshold_ms`**: Jumps directly to zero (prevents getting
    ///   stuck with tiny delays that never quite reach zero - Zeno's paradox solution)
    /// - **Above threshold**: Divides current delay by `success_divisor` (or the
    ///   auto-calculated value if not explicitly configured)
    /// - **Always**: Respects `min_delay_ms` as a lower bound
    /// - **Always**: Resets consecutive success counter to validate stability at the new rate
    ///
    /// # Example
    ///
    /// ```
    /// use si_layer_cache::rate_limiter::{RateLimiter, RateLimitConfig};
    /// use std::time::Duration;
    ///
    /// let config = RateLimitConfig {
    ///     successes_before_reduction: 3,
    ///     ..Default::default()
    /// };
    /// let mut limiter = RateLimiter::new(config);
    ///
    /// // Simulate throttling
    /// limiter.on_throttle();
    /// assert_eq!(limiter.current_delay(), Duration::from_millis(100));
    ///
    /// // Simulate success streak
    /// for _ in 0..3 {
    ///     limiter.on_success();
    /// }
    ///
    /// // Reduce backoff
    /// assert!(limiter.should_reduce_backoff());
    /// limiter.reduce_backoff();
    /// assert!(limiter.current_delay() < Duration::from_millis(100));
    /// assert_eq!(limiter.consecutive_successes(), 0); // Counter resets
    /// ```
    pub fn reduce_backoff(&mut self) {
        let zeno_threshold = Duration::from_millis(self.config.zeno_threshold_ms);

        if self.current_delay < zeno_threshold {
            // Zeno's paradox solution: jump to zero below threshold
            self.current_delay = Duration::ZERO;
        } else {
            // Divide by success divisor
            let current_ms = self.current_delay.as_millis() as f32;
            let divisor = self.config.success_divisor();
            let new_ms = (current_ms / divisor) as u64;

            // Respect minimum delay
            let clamped_ms = new_ms.max(self.config.min_delay_ms);
            self.current_delay = Duration::from_millis(clamped_ms);
        }

        // Reset streak - validates stability at new rate
        self.consecutive_successes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = RateLimitConfig::default();

        assert_eq!(config.min_delay_ms, 0);
        assert_eq!(config.max_delay_ms, 5000);
        assert_eq!(config.initial_backoff_ms, 100);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.success_divisor, None);
        assert_eq!(config.zeno_threshold_ms, 50);
        assert_eq!(config.successes_before_reduction, 3);
    }

    #[test]
    fn test_success_divisor_auto_calculation() {
        let config = RateLimitConfig {
            backoff_multiplier: 2.0,
            success_divisor: None,
            ..Default::default()
        };

        assert_eq!(config.success_divisor(), 1.5); // 2.0 * 0.75
    }

    #[test]
    fn test_success_divisor_explicit_value() {
        let config = RateLimitConfig {
            backoff_multiplier: 2.0,
            success_divisor: Some(3.0),
            ..Default::default()
        };

        assert_eq!(config.success_divisor(), 3.0);
    }

    #[test]
    fn test_rate_limiter_new_starts_with_zero_delay() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        assert_eq!(limiter.current_delay(), Duration::ZERO);
    }

    #[test]
    fn test_rate_limiter_starts_with_zero_successes() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        assert_eq!(limiter.consecutive_successes(), 0);
    }

    #[test]
    fn test_on_throttle_increases_delay_from_zero() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        limiter.on_throttle();

        assert_eq!(limiter.current_delay(), Duration::from_millis(100)); // initial_backoff_ms
        assert_eq!(limiter.consecutive_successes(), 0);
    }

    #[test]
    fn test_on_throttle_multiplies_existing_delay() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        limiter.on_throttle(); // 0 -> 100ms
        limiter.on_throttle(); // 100ms -> 200ms (multiplier=2.0)

        assert_eq!(limiter.current_delay(), Duration::from_millis(200));
    }

    #[test]
    fn test_on_throttle_respects_max_delay() {
        let config = RateLimitConfig {
            max_delay_ms: 500,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        limiter.on_throttle(); // 0 -> 100ms
        limiter.on_throttle(); // 100ms -> 200ms
        limiter.on_throttle(); // 200ms -> 400ms
        limiter.on_throttle(); // 400ms -> 500ms (capped at max)

        assert_eq!(limiter.current_delay(), Duration::from_millis(500));
    }

    #[test]
    fn test_on_throttle_resets_success_streak() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        limiter.on_success();
        limiter.on_success();
        assert_eq!(limiter.consecutive_successes(), 2);

        limiter.on_throttle();
        assert_eq!(limiter.consecutive_successes(), 0);
    }

    #[test]
    fn test_on_success_increments_streak() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        limiter.on_success();
        assert_eq!(limiter.consecutive_successes(), 1);

        limiter.on_success();
        assert_eq!(limiter.consecutive_successes(), 2);
    }

    #[test]
    fn test_should_reduce_backoff_after_n_successes() {
        let config = RateLimitConfig {
            successes_before_reduction: 3,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        assert!(!limiter.should_reduce_backoff());

        limiter.on_success();
        assert!(!limiter.should_reduce_backoff());

        limiter.on_success();
        assert!(!limiter.should_reduce_backoff());

        limiter.on_success();
        assert!(limiter.should_reduce_backoff()); // After 3rd success
    }

    #[test]
    fn test_reduce_backoff_divides_delay() {
        let config = RateLimitConfig::default(); // divisor = 1.5
        let mut limiter = RateLimiter::new(config);

        // Set up delay at 900ms
        limiter.on_throttle(); // 0 -> 100ms
        limiter.on_throttle(); // 100ms -> 200ms
        limiter.on_throttle(); // 200ms -> 400ms
        limiter.on_throttle(); // 400ms -> 800ms
        limiter.on_throttle(); // 800ms -> 1600ms (would be, but we'll use 900 for cleaner test)
        limiter.current_delay = Duration::from_millis(900); // Set directly for test clarity

        limiter.reduce_backoff();

        assert_eq!(limiter.current_delay(), Duration::from_millis(600)); // 900 / 1.5
        assert_eq!(limiter.consecutive_successes(), 0); // Streak resets after reduction
    }

    #[test]
    fn test_reduce_backoff_resets_to_zero_below_threshold() {
        let config = RateLimitConfig {
            zeno_threshold_ms: 50,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        limiter.current_delay = Duration::from_millis(40); // Below threshold

        limiter.reduce_backoff();

        assert_eq!(limiter.current_delay(), Duration::ZERO);
        assert_eq!(limiter.consecutive_successes(), 0);
    }

    #[test]
    fn test_reduce_backoff_respects_min_delay() {
        let config = RateLimitConfig {
            min_delay_ms: 50,
            zeno_threshold_ms: 25, // Set threshold below min to test min clamping
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        // Set delay to 60ms, which is above zeno_threshold (25ms)
        // Division: 60 / 1.5 = 40ms, but min is 50ms, so should clamp to 50ms
        limiter.current_delay = Duration::from_millis(60);
        limiter.reduce_backoff();

        assert_eq!(limiter.current_delay(), Duration::from_millis(50));
    }

    #[test]
    fn test_validate_default_config() {
        let config = RateLimitConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_min_greater_than_max() {
        let config = RateLimitConfig {
            min_delay_ms: 1000,
            max_delay_ms: 500,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::MinGreaterThanMax(1000, 500))
        ));
    }

    #[test]
    fn test_validate_multiplier_too_small() {
        let config = RateLimitConfig {
            backoff_multiplier: 0.5,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::MultiplierTooSmall(_))
        ));
    }

    #[test]
    fn test_validate_divisor_too_small() {
        let config = RateLimitConfig {
            success_divisor: Some(0.8),
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::DivisorTooSmall(_))
        ));
    }

    #[test]
    fn test_validate_successes_zero() {
        let config = RateLimitConfig {
            successes_before_reduction: 0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::SuccessesZero)
        ));
    }
}
