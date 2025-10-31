import { expect, test, describe } from "vitest";
import { escapeJsonPointerSegment } from "./util";

/**
 * Tests for escapeJsonPointerSegment function
 *
 * This function escapes string segments for use in JSON Pointers according to RFC 6901.
 * JSON Pointers use '~' as an escape character:
 * - '~' must be encoded as '~0'
 * - '/' must be encoded as '~1'
 */

describe("escapeJsonPointerSegment", () => {
  test("escapes forward slash in key name", () => {
    // Given: A key with a forward slash
    const key = "test/paul";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: Forward slash should be escaped as ~1
    expect(escaped).toBe("test~1paul");
  });

  test("escapes tilde in key name", () => {
    // Given: A key with a tilde
    const key = "a~b";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: Tilde should be escaped as ~0
    expect(escaped).toBe("a~0b");
  });

  test("escapes both tilde and forward slash in correct order", () => {
    // Given: A key with both tilde and forward slash
    const key = "test/~foo";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: Tilde should be escaped first (~0), then forward slash (~1)
    expect(escaped).toBe("test~1~0foo");
  });

  test("does not modify key without special characters", () => {
    // Given: A key with only regular characters
    const key = "normalKey123";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: Key should remain unchanged
    expect(escaped).toBe("normalKey123");
  });

  test("escapes multiple forward slashes", () => {
    // Given: A key with multiple forward slashes
    const key = "path/to/resource";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: All forward slashes should be escaped
    expect(escaped).toBe("path~1to~1resource");
  });

  test("escapes multiple tildes", () => {
    // Given: A key with multiple tildes
    const key = "~a~b~c";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: All tildes should be escaped
    expect(escaped).toBe("~0a~0b~0c");
  });

  test("handles empty string", () => {
    // Given: An empty string
    const key = "";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: Empty string should remain empty
    expect(escaped).toBe("");
  });

  test("escapes complex key with mixed special characters", () => {
    // Given: A key with complex pattern of special characters
    const key = "~/test/~/path~";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: All special characters should be properly escaped
    expect(escaped).toBe("~0~1test~1~0~1path~0");
  });

  test("handles key with spaces and special characters", () => {
    // Given: A key with spaces and special characters
    const key = "user name/email~address";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: Only special characters should be escaped, spaces remain
    expect(escaped).toBe("user name~1email~0address");
  });

  test("escapes real-world AWS IAM path example", () => {
    // Given: A realistic key that might be used in AWS IAM paths
    const key = "arn:aws:iam::123456789/role";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: Forward slash should be escaped
    expect(escaped).toBe("arn:aws:iam::123456789~1role");
  });

  test("preserves other special characters", () => {
    // Given: A key with various special characters that don't need escaping
    const key = "key-with_special.chars@123!";

    // When: Escaping the key for JSON Pointer use
    const escaped = escapeJsonPointerSegment(key);

    // Then: Only ~ and / should be escaped, other characters remain
    expect(escaped).toBe("key-with_special.chars@123!");
  });
});
