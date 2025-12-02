//! Adaptive rate limiting for S3 writes.
//!
//! Adjusts delay between operations based on throttling responses:
//! - On throttle: Increase backoff delay
//! - After N consecutive successes: Decrease backoff delay
//! - Learning rate adapts: grows when throttled, shrinks during recovery
//!
//! See [`RateLimitConfig`] for configuration parameters.

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

    #[error("successes_before_reduction cannot be zero")]
    SuccessesZero,

    #[error("learning_rate_growth ({0}) must be greater than 1.0")]
    LearningRateGrowthTooSmall(f64),

    #[error("learning_rate_shrink ({0}) must be greater than 0.0 and less than 1.0")]
    LearningRateShrinkOutOfBounds(f64),

    #[error("min_learning_rate ({0}) must be less than max_learning_rate ({1})")]
    MinLearningRateGreaterThanMax(f64, f64),

    #[error(
        "initial_learning_rate ({0}) should be within [min_learning_rate ({1}), max_learning_rate ({2})] bounds"
    )]
    InitialLearningRateOutOfBounds(f64, f64, f64),
}

/// Configuration for adaptive rate limiting with gradient descent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Minimum delay to maintain between operations (in milliseconds).
    ///
    /// Even after successful operations, the delay will not be reduced below this value.
    /// Set to 0 to allow full reduction to zero delay.
    pub min_delay_ms: u64,

    /// Maximum delay cap for backoff (in milliseconds).
    ///
    /// When throttled, delays grow but will never exceed this value.
    /// This prevents indefinite waiting during sustained throttling.
    pub max_delay_ms: u64,

    /// Initial delay applied on the first throttling event (in milliseconds).
    ///
    /// When transitioning from zero delay to backoff, this value is used as the
    /// starting point. Should be large enough to meaningfully reduce request rate.
    pub initial_backoff_ms: u64,

    /// Base step size for backoff adjustments (in milliseconds).
    ///
    /// On each adjustment (increase or decrease), the actual change is:
    /// `learning_rate * adjustment_size_ms`
    ///
    /// - **Larger values** (e.g., 200ms): Faster convergence but larger oscillations
    /// - **Smaller values** (e.g., 50ms): Slower convergence but tighter oscillations
    pub adjustment_size_ms: u64,

    /// Starting learning rate when first throttled.
    ///
    /// The learning rate scales the adjustment size. Initial adjustment will be:
    /// `initial_learning_rate * adjustment_size_ms`
    ///
    /// Typical range: 0.5 to 2.0
    pub initial_learning_rate: f64,

    /// Minimum bound for learning rate.
    ///
    /// Prevents learning rate from shrinking too small, which would cause
    /// glacially slow convergence. Learning rate cannot go below this value.
    pub min_learning_rate: f64,

    /// Maximum bound for learning rate.
    ///
    /// Prevents learning rate from growing too large, which would cause
    /// huge oscillations. Learning rate cannot exceed this value.
    pub max_learning_rate: f64,

    /// Multiplier applied to learning rate on each throttle event.
    ///
    /// When throttled repeatedly, the learning rate grows to make larger adjustments,
    /// helping escape the throttling zone faster.
    ///
    /// - **Higher values** (e.g., 1.2): Learning rate grows faster (1.0 → 1.2 → 1.44...)
    ///   - Escapes throttling faster with larger adjustments
    ///   - Risk of overshooting if throttling is intermittent
    /// - **Lower values** (e.g., 1.05): Learning rate grows slower (1.0 → 1.05 → 1.10...)
    ///   - More gradual escalation, less overshoot
    ///   - Takes longer to reach stable backoff
    ///
    /// Must be > 1.0. Typical range: 1.05 to 1.2
    pub learning_rate_growth: f64,

    /// Multiplier applied to learning rate after each successful backoff reduction.
    ///
    /// After N consecutive successes, when we reduce the backoff, the learning rate
    /// also shrinks to make future adjustments smaller (approaching optimal rate).
    ///
    /// - **Higher values** (closer to 1.0, e.g., 0.95): More aggressive recovery
    ///   - Learning rate decreases only 5% per cycle
    ///   - Adjustments stay large: 100ms → 95ms → 90ms → 86ms...
    ///   - Backoff reduces faster, recovery is quicker
    /// - **Lower values** (closer to 0, e.g., 0.8): More conservative recovery
    ///   - Learning rate decreases 20% per cycle
    ///   - Adjustments shrink quickly: 100ms → 80ms → 64ms → 51ms...
    ///   - Backoff reduces slower, validates stability longer
    ///
    /// Must be > 0.0 and < 1.0. Typical range: 0.8 to 0.95
    pub learning_rate_shrink: f64,

    /// Number of consecutive successes required before reducing backoff.
    ///
    /// This validation period ensures the system is stable at the current rate
    /// before speeding up. Acts as a "confidence threshold" for recovery.
    ///
    /// - **Higher values** (e.g., 5): More conservative
    ///   - Validates stability longer at each rate level
    ///   - Slower recovery but less risk of overshooting
    /// - **Lower values** (e.g., 2): More aggressive
    ///   - Reduces backoff more eagerly
    ///   - Faster recovery but may overshoot and re-throttle
    ///
    /// Must be > 0. Typical range: 2 to 5
    pub successes_before_reduction: u32,

    /// Threshold below which delays jump directly to zero (in milliseconds).
    ///
    /// Solves Zeno's paradox: prevents getting stuck with tiny delays that
    /// never quite reach zero through repeated division. When backoff falls
    /// below this threshold, it's reset to zero instead.
    ///
    /// Should be small enough to not noticeably impact throughput (< 100ms).
    pub zeno_threshold_ms: u64,
}

impl RateLimitConfig {
    /// Validates the configuration parameters.
    ///
    /// Returns an error if any of the following conditions are violated:
    /// - `min_delay_ms` must be less than or equal to `max_delay_ms`
    /// - `successes_before_reduction` must be greater than 0
    /// - `learning_rate_growth` must be greater than 1.0
    /// - `learning_rate_shrink` must be greater than 0.0 and less than 1.0
    /// - `min_learning_rate` must be less than `max_learning_rate`
    /// - `initial_learning_rate` should be within [min_learning_rate, max_learning_rate] bounds
    pub fn validate(&self) -> Result<(), RateLimitConfigError> {
        if self.min_delay_ms > self.max_delay_ms {
            return Err(RateLimitConfigError::MinGreaterThanMax(
                self.min_delay_ms,
                self.max_delay_ms,
            ));
        }

        if self.successes_before_reduction == 0 {
            return Err(RateLimitConfigError::SuccessesZero);
        }

        if self.learning_rate_growth <= 1.0 {
            return Err(RateLimitConfigError::LearningRateGrowthTooSmall(
                self.learning_rate_growth,
            ));
        }

        if self.learning_rate_shrink <= 0.0 || self.learning_rate_shrink >= 1.0 {
            return Err(RateLimitConfigError::LearningRateShrinkOutOfBounds(
                self.learning_rate_shrink,
            ));
        }

        if self.min_learning_rate >= self.max_learning_rate {
            return Err(RateLimitConfigError::MinLearningRateGreaterThanMax(
                self.min_learning_rate,
                self.max_learning_rate,
            ));
        }

        if self.initial_learning_rate < self.min_learning_rate
            || self.initial_learning_rate > self.max_learning_rate
        {
            return Err(RateLimitConfigError::InitialLearningRateOutOfBounds(
                self.initial_learning_rate,
                self.min_learning_rate,
                self.max_learning_rate,
            ));
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
            adjustment_size_ms: 100,
            initial_learning_rate: 1.0,
            min_learning_rate: 0.1,
            max_learning_rate: 3.0,
            learning_rate_growth: 1.1,
            learning_rate_shrink: 0.9,
            successes_before_reduction: 3,
            zeno_threshold_ms: 50,
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
    current_backoff_ms: f64,
    learning_rate: f64,
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
            current_backoff_ms: 0.0,
            learning_rate: config.initial_learning_rate,
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
        Duration::from_millis(self.current_backoff_ms as u64)
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
    /// - Otherwise adds `learning_rate * adjustment_size_ms` to current delay
    /// - Grows `learning_rate` by `learning_rate_growth` multiplier
    /// - Both backoff and learning rate respect their max bounds
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
        if self.current_backoff_ms == 0.0 {
            self.current_backoff_ms = self.config.initial_backoff_ms as f64;
            self.learning_rate = self.config.initial_learning_rate;
        } else {
            let adjustment = self.learning_rate * self.config.adjustment_size_ms as f64;
            self.current_backoff_ms =
                (self.current_backoff_ms + adjustment).min(self.config.max_delay_ms as f64);

            self.learning_rate = (self.learning_rate * self.config.learning_rate_growth)
                .min(self.config.max_learning_rate);
        }
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
    /// - **Below `zeno_threshold_ms`**: Jumps directly to zero and resets learning rate
    ///   (prevents getting stuck with tiny delays that never quite reach zero - Zeno's paradox solution)
    /// - **Above threshold**: Subtracts `learning_rate * adjustment_size_ms` from current delay,
    ///   then shrinks learning rate by `learning_rate_shrink` multiplier
    /// - **Always**: Respects `min_delay_ms` as a lower bound for backoff
    /// - **Always**: Respects `min_learning_rate` as a lower bound for learning rate
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
        let zeno_threshold = self.config.zeno_threshold_ms as f64;

        if self.current_backoff_ms < zeno_threshold {
            self.current_backoff_ms = 0.0;
            self.learning_rate = self.config.initial_learning_rate;
        } else {
            let adjustment = self.learning_rate * self.config.adjustment_size_ms as f64;
            self.current_backoff_ms =
                (self.current_backoff_ms - adjustment).max(self.config.min_delay_ms as f64);

            self.learning_rate = (self.learning_rate * self.config.learning_rate_shrink)
                .max(self.config.min_learning_rate);
        }
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
        assert_eq!(config.adjustment_size_ms, 100);
        assert_eq!(config.initial_learning_rate, 1.0);
        assert_eq!(config.min_learning_rate, 0.1);
        assert_eq!(config.max_learning_rate, 3.0);
        assert_eq!(config.learning_rate_growth, 1.1);
        assert_eq!(config.learning_rate_shrink, 0.9);
        assert_eq!(config.successes_before_reduction, 3);
        assert_eq!(config.zeno_threshold_ms, 50);
    }

    #[test]
    fn test_rate_limiter_new_starts_with_zero_backoff() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        assert_eq!(limiter.current_backoff_ms, 0.0);
        assert_eq!(limiter.current_delay(), Duration::ZERO);
    }

    #[test]
    fn test_rate_limiter_starts_with_zero_successes() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        assert_eq!(limiter.consecutive_successes(), 0);
    }

    #[test]
    fn test_rate_limiter_new_initializes_learning_rate() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        assert_eq!(limiter.learning_rate, 1.0);
    }

    #[test]
    fn test_on_throttle_sets_initial_backoff_from_zero() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        limiter.on_throttle();

        assert_eq!(limiter.current_backoff_ms, 100.0);
        assert_eq!(limiter.learning_rate, 1.0);
        assert_eq!(limiter.consecutive_successes(), 0);
    }

    #[test]
    fn test_on_throttle_adds_scaled_adjustment() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        limiter.on_throttle();
        assert_eq!(limiter.current_backoff_ms, 100.0);

        limiter.on_throttle();
        assert_eq!(limiter.current_backoff_ms, 200.0);

        limiter.on_throttle();
        assert_eq!(limiter.current_backoff_ms, 310.0);
    }

    #[test]
    fn test_learning_rate_grows_on_throttle() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        limiter.on_throttle();
        assert_eq!(limiter.learning_rate, 1.0);

        limiter.on_throttle();
        assert!((limiter.learning_rate - 1.1).abs() < 0.001);

        limiter.on_throttle();
        assert!((limiter.learning_rate - 1.21).abs() < 0.001);
    }

    #[test]
    fn test_on_throttle_respects_max_delay() {
        let config = RateLimitConfig {
            max_delay_ms: 500,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        for _ in 0..10 {
            limiter.on_throttle();
        }

        assert!(limiter.current_backoff_ms <= 500.0);
    }

    #[test]
    fn test_learning_rate_respects_max_bound() {
        let config = RateLimitConfig {
            max_learning_rate: 2.0,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        for _ in 0..20 {
            limiter.on_throttle();
        }

        assert!(limiter.learning_rate <= 2.0);
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
    fn test_reduce_backoff_subtracts_scaled_adjustment() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        limiter.current_backoff_ms = 900.0;
        limiter.learning_rate = 1.0;

        limiter.reduce_backoff();

        assert_eq!(limiter.current_backoff_ms, 800.0);
    }

    #[test]
    fn test_learning_rate_shrinks_on_reduction() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        limiter.current_backoff_ms = 900.0;
        limiter.learning_rate = 1.0;

        limiter.reduce_backoff();

        assert!((limiter.learning_rate - 0.9).abs() < 0.001);
        assert_eq!(limiter.consecutive_successes(), 0);
    }

    #[test]
    fn test_reduce_backoff_jumps_to_zero_below_threshold() {
        let config = RateLimitConfig {
            zeno_threshold_ms: 50,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        limiter.current_backoff_ms = 40.0;
        limiter.learning_rate = 2.0;

        limiter.reduce_backoff();

        assert_eq!(limiter.current_backoff_ms, 0.0);
        assert_eq!(limiter.learning_rate, 1.0);
        assert_eq!(limiter.consecutive_successes(), 0);
    }

    #[test]
    fn test_reduce_backoff_respects_min_delay() {
        let config = RateLimitConfig {
            min_delay_ms: 50,
            zeno_threshold_ms: 25,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        limiter.current_backoff_ms = 60.0;
        limiter.learning_rate = 1.0;

        limiter.reduce_backoff();

        assert_eq!(limiter.current_backoff_ms, 50.0);
    }

    #[test]
    fn test_learning_rate_respects_min_bound() {
        let config = RateLimitConfig {
            min_learning_rate: 0.5,
            ..Default::default()
        };
        let zeno_threshold = config.zeno_threshold_ms as f64;
        let mut limiter = RateLimiter::new(config);

        limiter.current_backoff_ms = 900.0;
        limiter.learning_rate = 0.6;

        for _ in 0..10 {
            limiter.reduce_backoff();
            if limiter.current_backoff_ms < zeno_threshold {
                break;
            }
        }

        assert!(limiter.learning_rate >= 0.5);
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

    #[test]
    fn test_validate_learning_rate_growth_too_small() {
        let config = RateLimitConfig {
            learning_rate_growth: 0.9,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::LearningRateGrowthTooSmall(_))
        ));
    }

    #[test]
    fn test_validate_learning_rate_growth_exactly_one() {
        let config = RateLimitConfig {
            learning_rate_growth: 1.0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::LearningRateGrowthTooSmall(_))
        ));
    }

    #[test]
    fn test_validate_learning_rate_shrink_too_small() {
        let config = RateLimitConfig {
            learning_rate_shrink: 0.0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::LearningRateShrinkOutOfBounds(_))
        ));
    }

    #[test]
    fn test_validate_learning_rate_shrink_too_large() {
        let config = RateLimitConfig {
            learning_rate_shrink: 1.0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::LearningRateShrinkOutOfBounds(_))
        ));
    }

    #[test]
    fn test_validate_learning_rate_shrink_above_one() {
        let config = RateLimitConfig {
            learning_rate_shrink: 1.5,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::LearningRateShrinkOutOfBounds(_))
        ));
    }

    #[test]
    fn test_validate_min_learning_rate_greater_than_max() {
        let config = RateLimitConfig {
            min_learning_rate: 2.0,
            max_learning_rate: 1.0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::MinLearningRateGreaterThanMax(
                2.0, 1.0
            ))
        ));
    }

    #[test]
    fn test_validate_min_learning_rate_equal_to_max() {
        let config = RateLimitConfig {
            min_learning_rate: 1.0,
            max_learning_rate: 1.0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::MinLearningRateGreaterThanMax(
                1.0, 1.0
            ))
        ));
    }

    #[test]
    fn test_validate_initial_learning_rate_below_min() {
        let config = RateLimitConfig {
            initial_learning_rate: 0.05,
            min_learning_rate: 0.1,
            max_learning_rate: 2.0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::InitialLearningRateOutOfBounds(
                0.05, 0.1, 2.0
            ))
        ));
    }

    #[test]
    fn test_validate_initial_learning_rate_above_max() {
        let config = RateLimitConfig {
            initial_learning_rate: 4.0,
            min_learning_rate: 0.1,
            max_learning_rate: 2.0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(RateLimitConfigError::InitialLearningRateOutOfBounds(
                4.0, 0.1, 2.0
            ))
        ));
    }
}
