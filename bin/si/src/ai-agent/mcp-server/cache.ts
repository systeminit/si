/**
 * MCP Server Cache Module
 *
 * Implements changeSet-based caching with TTL to reduce redundant API calls.
 * Addresses Token Reduction Plan Phase 1, Item #4.
 *
 * Key features:
 * - ChangeSet-aware: Cache entries are invalidated when changeSet changes
 * - TTL-based expiration: Entries expire after configurable duration
 * - Type-safe: Generic implementation supports any data type
 * - Memory-efficient: Single shared cache instance
 */

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  changeSetId: string;
}

/**
 * Simple in-memory cache with changeSet-based invalidation and TTL.
 *
 * Target: 50-70% reduction in repeated queries
 */
export class McpCache {
  private cache = new Map<string, CacheEntry<unknown>>();
  private readonly ttl: number;

  /**
   * @param ttl Time-to-live in milliseconds (default: 60000 = 60 seconds)
   */
  constructor(ttl = 60000) {
    this.ttl = ttl;
  }

  /**
   * Retrieve cached data if valid.
   *
   * Returns null if:
   * - Entry doesn't exist
   * - ChangeSet has changed
   * - TTL has expired
   */
  get<T>(key: string, currentChangeSetId: string): T | null {
    const entry = this.cache.get(key) as CacheEntry<T> | undefined;
    if (!entry) return null;

    // Invalidate if changeSet changed
    if (entry.changeSetId !== currentChangeSetId) {
      this.cache.delete(key);
      return null;
    }

    // Invalidate if TTL expired
    if (Date.now() - entry.timestamp > this.ttl) {
      this.cache.delete(key);
      return null;
    }

    return entry.data;
  }

  /**
   * Store data in cache with current changeSet and timestamp.
   */
  set<T>(key: string, data: T, changeSetId: string): void {
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      changeSetId,
    });
  }

  /**
   * Invalidate all entries for a specific changeSet.
   * Useful when a changeSet is applied or deleted.
   */
  invalidateChangeSet(changeSetId: string): void {
    for (const [key, entry] of this.cache) {
      if (entry.changeSetId === changeSetId) {
        this.cache.delete(key);
      }
    }
  }

  /**
   * Clear all cache entries.
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Get cache statistics for monitoring.
   */
  getStats(): { size: number; keys: string[] } {
    return {
      size: this.cache.size,
      keys: Array.from(this.cache.keys()),
    };
  }
}

// Singleton instance with default 5 minute TTL
// TODO: Make TTL configurable via CLI flag if needed
export const cache = new McpCache(300000);

/**
 * Helper function to generate consistent cache keys.
 *
 * @example
 * generateCacheKey("schema", schemaId, changeSetId) // "schema:abc123:xyz789"
 */
export function generateCacheKey(...parts: string[]): string {
  return parts.join(":");
}
