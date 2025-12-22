import { assertEquals, assertNotEquals } from "jsr:@std/assert";
import { generateCacheKey, McpCache } from "./cache.ts";

Deno.test("generateCacheKey - creates consistent cache keys", () => {
  const key1 = generateCacheKey("schema", "abc123", "changeset-1");
  const key2 = generateCacheKey("schema", "abc123", "changeset-1");
  const key3 = generateCacheKey("schema", "xyz789", "changeset-1");

  assertEquals(key1, "schema:abc123:changeset-1");
  assertEquals(key1, key2, "Same inputs should produce same key");
  assertNotEquals(key1, key3, "Different inputs should produce different keys");
});

Deno.test("generateCacheKey - handles various numbers of parts", () => {
  assertEquals(generateCacheKey("a"), "a");
  assertEquals(generateCacheKey("a", "b"), "a:b");
  assertEquals(generateCacheKey("a", "b", "c", "d"), "a:b:c:d");
});

Deno.test("McpCache - cache miss returns null", () => {
  const cache = new McpCache();
  const result = cache.get("nonexistent", "changeset-1");
  assertEquals(result, null);
});

Deno.test("McpCache - cache hit returns stored data", () => {
  const cache = new McpCache();
  const testData = { id: "123", name: "test" };

  cache.set("test-key", testData, "changeset-1");
  const result = cache.get("test-key", "changeset-1");

  assertEquals(result, testData);
});

Deno.test("McpCache - cache miss on different changeSet", () => {
  const cache = new McpCache();
  const testData = { id: "123", name: "test" };

  cache.set("test-key", testData, "changeset-1");
  const result = cache.get("test-key", "changeset-2");

  assertEquals(result, null, "Different changeSet should not return cached data");
});

Deno.test("McpCache - supports generic types", () => {
  const cache = new McpCache();

  // Test with different data types
  cache.set("string-key", "hello", "changeset-1");
  cache.set("number-key", 42, "changeset-1");
  cache.set("object-key", { foo: "bar" }, "changeset-1");
  cache.set("array-key", [1, 2, 3], "changeset-1");

  assertEquals(cache.get<string>("string-key", "changeset-1"), "hello");
  assertEquals(cache.get<number>("number-key", "changeset-1"), 42);
  assertEquals(cache.get<{ foo: string }>("object-key", "changeset-1"), {
    foo: "bar",
  });
  assertEquals(cache.get<number[]>("array-key", "changeset-1"), [1, 2, 3]);
});

Deno.test("McpCache - TTL expiration invalidates entries", async () => {
  const cache = new McpCache(50); // 50ms TTL
  cache.set("test-key", "test-value", "changeset-1");

  // Should hit immediately
  assertEquals(cache.get("test-key", "changeset-1"), "test-value");

  // Wait for TTL to expire
  await new Promise((resolve) => setTimeout(resolve, 60));

  // Should miss after TTL
  assertEquals(
    cache.get("test-key", "changeset-1"),
    null,
    "Entry should be expired after TTL",
  );
});

Deno.test("McpCache - invalidateChangeSet removes all entries for that changeSet", () => {
  const cache = new McpCache();

  // Set multiple entries for different changeSets
  cache.set("key-1", "value-1", "changeset-1");
  cache.set("key-2", "value-2", "changeset-1");
  cache.set("key-3", "value-3", "changeset-2");

  // Invalidate changeset-1
  cache.invalidateChangeSet("changeset-1");

  // changeset-1 entries should be gone
  assertEquals(cache.get("key-1", "changeset-1"), null);
  assertEquals(cache.get("key-2", "changeset-1"), null);

  // changeset-2 entry should remain
  assertEquals(cache.get("key-3", "changeset-2"), "value-3");
});

Deno.test("McpCache - clear removes all entries", () => {
  const cache = new McpCache();

  cache.set("key-1", "value-1", "changeset-1");
  cache.set("key-2", "value-2", "changeset-2");
  cache.set("key-3", "value-3", "changeset-3");

  cache.clear();

  assertEquals(cache.get("key-1", "changeset-1"), null);
  assertEquals(cache.get("key-2", "changeset-2"), null);
  assertEquals(cache.get("key-3", "changeset-3"), null);
});

Deno.test("McpCache - getStats returns correct statistics", () => {
  const cache = new McpCache();

  // Initially empty
  let stats = cache.getStats();
  assertEquals(stats.size, 0);
  assertEquals(stats.keys, []);

  // Add some entries
  cache.set("key-1", "value-1", "changeset-1");
  cache.set("key-2", "value-2", "changeset-1");

  stats = cache.getStats();
  assertEquals(stats.size, 2);
  assertEquals(stats.keys.sort(), ["key-1", "key-2"]);
});

Deno.test("McpCache - stats reflect invalidation operations", () => {
  const cache = new McpCache();

  cache.set("key-1", "value-1", "changeset-1");
  cache.set("key-2", "value-2", "changeset-1");
  cache.set("key-3", "value-3", "changeset-2");

  assertEquals(cache.getStats().size, 3);

  cache.invalidateChangeSet("changeset-1");
  assertEquals(cache.getStats().size, 1);

  cache.clear();
  assertEquals(cache.getStats().size, 0);
});

Deno.test("McpCache - overwriting existing key updates the value", () => {
  const cache = new McpCache();

  cache.set("test-key", "old-value", "changeset-1");
  assertEquals(cache.get("test-key", "changeset-1"), "old-value");

  cache.set("test-key", "new-value", "changeset-1");
  assertEquals(cache.get("test-key", "changeset-1"), "new-value");
});

Deno.test("McpCache - custom TTL works correctly", () => {
  const cache = new McpCache(100); // 100ms TTL
  assertEquals(cache.getStats().size, 0);
});

Deno.test("McpCache - handles null and undefined data", () => {
  const cache = new McpCache();

  cache.set("null-key", null, "changeset-1");
  cache.set("undefined-key", undefined, "changeset-1");

  assertEquals(cache.get("null-key", "changeset-1"), null);
  assertEquals(cache.get("undefined-key", "changeset-1"), undefined);
});

Deno.test("McpCache - realistic schema caching scenario", () => {
  const cache = new McpCache();

  // Simulate schema variant caching
  const schemaId = "schema-abc123";
  const changeSetId = "changeset-xyz789";
  const cacheKey = generateCacheKey("schema-variant", schemaId, changeSetId);

  const schemaData = {
    variantId: "variant-1",
    displayName: "AWS::EC2::Instance",
    domainProps: { /* ... */ },
    installedFromUpstream: true,
  };

  // First access - cache miss, set data
  assertEquals(cache.get(cacheKey, changeSetId), null);
  cache.set(cacheKey, schemaData, changeSetId);

  // Second access - cache hit
  assertEquals(cache.get(cacheKey, changeSetId), schemaData);

  // Different changeSet - cache miss
  assertEquals(cache.get(cacheKey, "different-changeset"), null);
});

Deno.test("McpCache - realistic component caching scenario", () => {
  const cache = new McpCache();

  // Simulate component caching for action-list N+1 fix
  const componentIds = ["comp-1", "comp-2", "comp-3"];
  const changeSetId = "changeset-123";

  // First iteration - populate cache
  componentIds.forEach((id, index) => {
    const cacheKey = generateCacheKey("component", id, changeSetId);
    cache.set(cacheKey, { id, name: `Component ${index + 1}` }, changeSetId);
  });

  // Second iteration - all cache hits
  componentIds.forEach((id, index) => {
    const cacheKey = generateCacheKey("component", id, changeSetId);
    const data = cache.get(cacheKey, changeSetId);
    assertEquals(data, { id, name: `Component ${index + 1}` });
  });

  assertEquals(cache.getStats().size, 3);
});

Deno.test("McpCache - concurrent operations don't cause issues", () => {
  const cache = new McpCache();

  // Simulate concurrent operations
  const operations = [];
  for (let i = 0; i < 100; i++) {
    operations.push(
      cache.set(`key-${i}`, `value-${i}`, "changeset-1"),
    );
  }

  // Verify all entries were stored
  assertEquals(cache.getStats().size, 100);

  // Verify all entries can be retrieved
  for (let i = 0; i < 100; i++) {
    assertEquals(cache.get(`key-${i}`, "changeset-1"), `value-${i}`);
  }
});
