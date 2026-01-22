use std::{
    collections::{
        HashMap,
        HashSet,
    },
    hash::Hash,
};

use tracing::{
    trace,
    warn,
};

/// Fair scheduler that selects keys with lowest message count.
///
/// This implements fair queuing (not weighted) where each key gets
/// equal opportunity to process messages. The key with the fewest
/// processed messages is always selected next, ensuring balanced
/// distribution across all active keys.
#[derive(Debug)]
pub struct Scheduler<K> {
    message_counts: HashMap<K, u64>,
    keys_with_work: HashSet<K>,
    max_processed: u64,
    stale_threshold: u64,
}

impl<K> Default for Scheduler<K>
where
    K: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K> Scheduler<K>
where
    K: Clone + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            message_counts: HashMap::new(),
            keys_with_work: HashSet::new(),
            max_processed: 0,
            stale_threshold: 100,
        }
    }

    pub fn with_stale_threshold(mut self, threshold: u64) -> Self {
        self.stale_threshold = threshold;
        self
    }

    /// Marks key as having work. Resets message count if stale.
    pub fn notify_has_work(&mut self, key: K) {
        let count = self
            .message_counts
            .entry(key.clone())
            .or_insert(self.max_processed);

        // Reset count if this key is too far behind (stale)
        if self.max_processed.saturating_sub(*count) > self.stale_threshold {
            *count = self.max_processed;
        }

        self.keys_with_work.insert(key);
    }

    pub fn notify_no_work(&mut self, key: &K) {
        self.keys_with_work.remove(key);
    }

    pub fn keys_with_work_count(&self) -> usize {
        self.keys_with_work.len()
    }

    pub fn has_work(&self) -> bool {
        !self.keys_with_work.is_empty()
    }

    /// Returns key with lowest message count (for fairness).
    pub fn next(&self) -> Option<K> {
        self.keys_with_work
            .iter()
            .min_by_key(|key| self.message_counts.get(*key).copied().unwrap_or(0))
            .cloned()
    }

    /// Increments message count after processing a message.
    pub fn complete(&mut self, key: &K, messages_processed: u64) {
        if let Some(count) = self.message_counts.get_mut(key) {
            *count = count.saturating_add(messages_processed);
            self.max_processed = self.max_processed.max(*count);
            trace!(
                key_count = *count,
                max_processed = self.max_processed,
                "message count updated"
            );
        } else {
            // This indicates a programming error: complete() was called for a key
            // that was never registered via notify_has_work().
            warn!("complete() called for unknown key - this may indicate a bug");
            debug_assert!(
                false,
                "complete() called for key not in message_counts - was notify_has_work() called first?"
            );
        }
    }

    pub fn remove(&mut self, key: &K) {
        self.message_counts.remove(key);
        self.keys_with_work.remove(key);
    }

    pub fn iter_keys_with_work(&self) -> impl Iterator<Item = &K> {
        self.keys_with_work.iter()
    }

    pub fn message_count(&self, key: &K) -> Option<u64> {
        self.message_counts.get(key).copied()
    }

    pub fn max_processed(&self) -> u64 {
        self.max_processed
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::Scheduler;

    // Tests for Scheduler
    #[test]
    fn test_basic_scheduling() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();
        scheduler.notify_has_work("a");
        scheduler.notify_has_work("b");
        scheduler.notify_has_work("c");
        assert!(scheduler.has_work());
        assert_eq!(scheduler.keys_with_work_count(), 3);
    }

    #[test]
    fn test_fairness_with_multiple_keys() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();
        scheduler.notify_has_work("a");
        scheduler.notify_has_work("b");
        scheduler.notify_has_work("c");

        let mut counts = HashMap::new();
        counts.insert("a", 0);
        counts.insert("b", 0);
        counts.insert("c", 0);

        // Process 30 messages
        for _ in 0..30 {
            let next = scheduler.next().unwrap();
            *counts.get_mut(next).unwrap() += 1;
            scheduler.complete(&next, 1);
            // Keep all keys active
            scheduler.notify_has_work("a");
            scheduler.notify_has_work("b");
            scheduler.notify_has_work("c");
        }

        // Each key should get exactly equal processing
        assert_eq!(counts["a"], 10);
        assert_eq!(counts["b"], 10);
        assert_eq!(counts["c"], 10);
    }

    #[test]
    fn test_scheduler_selects_lowest_message_count() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();

        // Add all keys first
        scheduler.notify_has_work("a");
        scheduler.notify_has_work("b");
        scheduler.notify_has_work("c");

        // Now set up different message counts by completing work
        scheduler.complete(&"a", 10);
        scheduler.complete(&"b", 5);
        // c remains at 0

        // Check message counts
        assert_eq!(scheduler.message_count(&"a"), Some(10));
        assert_eq!(scheduler.message_count(&"b"), Some(5));
        assert_eq!(scheduler.message_count(&"c"), Some(0));

        // Should select c (lowest count = 0)
        assert_eq!(scheduler.next(), Some("c"));
        scheduler.complete(&"c", 6);

        // Now c has 6, b has 5, a has 10 - should select b
        assert_eq!(scheduler.next(), Some("b"));
        scheduler.complete(&"b", 2);

        // Now b has 7, c has 6, a has 10 - should select c
        assert_eq!(scheduler.next(), Some("c"));
    }

    #[test]
    fn test_message_count_cleanup_on_remove() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();

        // Add key and build up message count
        scheduler.notify_has_work("workspace1");
        scheduler.complete(&"workspace1", 100);
        assert_eq!(scheduler.message_count(&"workspace1"), Some(100));

        // Remove key
        scheduler.remove(&"workspace1");

        // Verify message count is gone
        assert_eq!(scheduler.message_count(&"workspace1"), None);
        assert!(!scheduler.has_work());

        // Re-add key, verify starts at max_processed
        scheduler.notify_has_work("workspace1");
        assert_eq!(scheduler.message_count(&"workspace1"), Some(100)); // max_processed is 100
    }

    #[test]
    fn test_scheduler_state_consistency() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();

        // Add keys
        scheduler.notify_has_work("a");
        scheduler.notify_has_work("b");

        // Both should be in keys_with_work
        assert_eq!(scheduler.keys_with_work_count(), 2);
        assert!(scheduler.message_count(&"a").is_some());
        assert!(scheduler.message_count(&"b").is_some());

        // Remove from work
        scheduler.notify_no_work(&"a");
        assert_eq!(scheduler.keys_with_work_count(), 1);
        // Message count should still exist
        assert!(scheduler.message_count(&"a").is_some());

        // Remove entirely
        scheduler.remove(&"b");
        assert_eq!(scheduler.keys_with_work_count(), 0);
        assert!(scheduler.message_count(&"b").is_none());
    }

    #[test]
    fn test_rapid_add_remove_same_key() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();

        for i in 0..10 {
            scheduler.notify_has_work("key");
            scheduler.complete(&"key", 1);
            if i % 2 == 0 {
                scheduler.notify_no_work(&"key");
            }
        }

        // Should handle rapid changes without issues
        assert!(scheduler.message_count(&"key").is_some());
        assert_eq!(scheduler.message_count(&"key"), Some(10));
    }

    #[test]
    fn test_scheduler_with_many_keys() {
        let mut scheduler: Scheduler<String> = Scheduler::new();

        // Add 100 keys
        for i in 0..100 {
            let key = format!("key_{i}");
            scheduler.notify_has_work(key);
        }

        assert_eq!(scheduler.keys_with_work_count(), 100);

        // Process some messages
        for _ in 0..200 {
            let next = scheduler.next().unwrap();
            scheduler.complete(&next, 1);
            scheduler.notify_has_work(next);
        }

        // Verify all keys still tracked
        assert_eq!(scheduler.keys_with_work_count(), 100);
    }

    #[test]
    fn test_stale_key_reset() {
        let mut scheduler: Scheduler<&str> = Scheduler::new().with_stale_threshold(100);

        scheduler.notify_has_work("a");
        scheduler.complete(&"a", 500);
        scheduler.notify_no_work(&"a");

        // Add new key, advances max_processed
        scheduler.notify_has_work("b");
        assert_eq!(scheduler.max_processed(), 500);

        // Re-add stale key
        scheduler.notify_has_work("a");
        // Should reset to max_processed due to staleness
        assert_eq!(scheduler.message_count(&"a"), Some(500));
    }

    #[test]
    fn test_no_work() {
        let scheduler: Scheduler<&str> = Scheduler::new();
        assert!(!scheduler.has_work());
        assert_eq!(scheduler.next(), None);
    }

    #[test]
    fn test_remove_key() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();
        scheduler.notify_has_work("a");
        scheduler.notify_has_work("b");
        scheduler.complete(&"a", 10);
        scheduler.remove(&"a");

        // Use public API to verify state
        assert_eq!(scheduler.keys_with_work_count(), 1);
        assert!(scheduler.message_count(&"a").is_none());
        assert_eq!(scheduler.next(), Some("b"));
    }

    #[test]
    fn test_interleaved_processing() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();
        scheduler.notify_has_work("a");
        scheduler.notify_has_work("b");

        let mut a_count = 0;
        let mut b_count = 0;

        for _ in 0..20 {
            let next = scheduler.next().unwrap();
            if next == "a" {
                a_count += 1;
            } else {
                b_count += 1;
            }
            scheduler.complete(&next, 1);
            scheduler.notify_has_work("a");
            scheduler.notify_has_work("b");
        }

        assert_eq!(a_count, 10);
        assert_eq!(b_count, 10);
    }

    #[test]
    fn test_consumer_replacement_preserves_message_count() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();

        // Add key and build up message count
        scheduler.notify_has_work("ws1");
        scheduler.complete(&"ws1", 50);
        assert_eq!(scheduler.message_count(&"ws1"), Some(50));

        // Simulate consumer replacement by just notifying again (new behavior)
        scheduler.notify_has_work("ws1");

        // Message count should be preserved at 50
        assert_eq!(scheduler.message_count(&"ws1"), Some(50));
    }

    #[test]
    fn test_keys_with_work_and_message_counts_consistency() {
        let mut scheduler: Scheduler<&str> = Scheduler::new();

        // Add some keys
        scheduler.notify_has_work("a");
        scheduler.notify_has_work("b");
        scheduler.notify_has_work("c");

        // All keys should have message counts
        assert!(scheduler.message_count(&"a").is_some());
        assert!(scheduler.message_count(&"b").is_some());
        assert!(scheduler.message_count(&"c").is_some());

        // Remove work notification
        scheduler.notify_no_work(&"b");

        // Key should not be in work set but message count persists
        assert_eq!(scheduler.keys_with_work_count(), 2);
        assert!(scheduler.message_count(&"b").is_some());

        // Complete removal
        scheduler.remove(&"c");

        // Should be gone from both
        assert_eq!(scheduler.keys_with_work_count(), 1);
        assert!(scheduler.message_count(&"c").is_none());
    }

    // Tests that verify behavior we'll change in refactoring
    #[test]
    fn test_replacement_behavior_preserves_fairness() {
        // After refactor: replacement preserves message count for fairness
        let mut scheduler: Scheduler<&str> = Scheduler::new();

        // Add two workspaces
        scheduler.notify_has_work("ws1");
        scheduler.notify_has_work("ws2");

        // ws1 processes more work (scheduler should alternate due to fairness)
        let mut ws1_count = 0;
        for _ in 0..10 {
            let next = scheduler.next().unwrap();
            if next == "ws1" {
                ws1_count += 1;
            }
            scheduler.complete(&next, 1);
            scheduler.notify_has_work("ws1");
            scheduler.notify_has_work("ws2");
        }

        // Both should have processed equal work due to fairness
        assert_eq!(ws1_count, 5);
        assert_eq!(scheduler.message_count(&"ws1"), Some(5));
        assert_eq!(scheduler.message_count(&"ws2"), Some(5));

        // Process one from ws1 to make it have higher message count
        assert!(scheduler.next().is_some());
        scheduler.complete(&"ws1", 1);

        // Now ws1 has count 6, ws2 has 5
        assert_eq!(scheduler.message_count(&"ws1"), Some(6));
        assert_eq!(scheduler.message_count(&"ws2"), Some(5));

        // Simulate ws1 consumer replacement
        scheduler.notify_has_work("ws1");
        scheduler.notify_has_work("ws2");

        // Message count preserved - ws2 should be selected next (lower count)
        assert_eq!(scheduler.next(), Some("ws2"));
    }
}
