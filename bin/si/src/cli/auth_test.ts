/**
 * Tests for auth module - isTokenAboutToExpire function
 */

import { assertEquals } from "@std/assert";
import { isTokenAboutToExpire } from "./auth.ts";

/**
 * Helper function to create a mock JWT token with specified expiration
 * @param expirationMinutesFromNow - Minutes from now when the token expires (can be negative for past)
 * @param includeExp - Whether to include the exp field
 * @returns Mock JWT token string
 */
function createMockToken(
  expirationMinutesFromNow: number,
  includeExp: boolean = true,
): string {
  const header = { alg: "HS256", typ: "JWT" };

  const now = Math.floor(Date.now() / 1000);
  const exp = now + expirationMinutesFromNow * 60;

  const payload: { workspaceId: string; userId: string; exp?: number } = {
    workspaceId: "test-workspace-123",
    userId: "test-user-456",
  };

  if (includeExp) {
    payload.exp = exp;
  }

  // Base64url encode the header and payload
  const base64urlEncode = (str: string): string => {
    return btoa(str).replace(/=/g, "").replace(/\+/g, "-").replace(/\//g, "_");
  };

  const encodedHeader = base64urlEncode(JSON.stringify(header));
  const encodedPayload = base64urlEncode(JSON.stringify(payload));

  // Mock signature (not cryptographically valid, but sufficient for testing)
  const mockSignature = "mock-signature-for-testing";

  return `${encodedHeader}.${encodedPayload}.${mockSignature}`;
}

Deno.test("isTokenAboutToExpire - token already expired", () => {
  // Token expired 1 hour ago
  const expiredToken = createMockToken(-60);
  const result = isTokenAboutToExpire(expiredToken);
  assertEquals(result, true, "Should return true for expired token");
});

Deno.test("isTokenAboutToExpire - token expired just now", () => {
  // Token expired 1 minute ago
  const expiredToken = createMockToken(-1);
  const result = isTokenAboutToExpire(expiredToken);
  assertEquals(result, true, "Should return true for recently expired token");
});

Deno.test(
  "isTokenAboutToExpire - token expires within default grace period (60 minutes)",
  () => {
    // Token expires in 30 minutes (within default 60 minute grace)
    const soonToken = createMockToken(30);
    const result = isTokenAboutToExpire(soonToken);
    assertEquals(
      result,
      true,
      "Should return true for token expiring within grace period",
    );
  },
);

Deno.test(
  "isTokenAboutToExpire - token expires at edge of grace period",
  () => {
    // Token expires in exactly 60 minutes
    const edgeToken = createMockToken(60);
    const result = isTokenAboutToExpire(edgeToken);
    assertEquals(
      result,
      true,
      "Should return true for token expiring at grace period boundary",
    );
  },
);

Deno.test("isTokenAboutToExpire - token expires outside grace period", () => {
  // Token expires in 90 minutes (outside default 60 minute grace)
  const validToken = createMockToken(90);
  const result = isTokenAboutToExpire(validToken);
  assertEquals(
    result,
    false,
    "Should return false for token expiring after grace period",
  );
});

Deno.test("isTokenAboutToExpire - token expires well in the future", () => {
  // Token expires in 24 hours
  const futureToken = createMockToken(24 * 60);
  const result = isTokenAboutToExpire(futureToken);
  assertEquals(
    result,
    false,
    "Should return false for token expiring far in the future",
  );
});

Deno.test("isTokenAboutToExpire - custom grace period shorter", () => {
  // Token expires in 20 minutes, grace period is 15 minutes
  const token = createMockToken(20);
  const result = isTokenAboutToExpire(token, 15);
  assertEquals(
    result,
    false,
    "Should return false when outside custom grace period",
  );
});

Deno.test("isTokenAboutToExpire - custom grace period longer", () => {
  // Token expires in 90 minutes, grace period is 120 minutes
  const token = createMockToken(90);
  const result = isTokenAboutToExpire(token, 120);
  assertEquals(
    result,
    true,
    "Should return true when within longer custom grace period",
  );
});

Deno.test("isTokenAboutToExpire - zero grace period", () => {
  // Token expires in 5 minutes, grace period is 0
  const token = createMockToken(5);
  const result = isTokenAboutToExpire(token, 0);
  assertEquals(
    result,
    false,
    "Should return false when grace period is zero and token not expired",
  );
});

Deno.test("isTokenAboutToExpire - zero grace period with expired token", () => {
  // Token expired 5 minutes ago, grace period is 0
  const token = createMockToken(-5);
  const result = isTokenAboutToExpire(token, 0);
  assertEquals(
    result,
    true,
    "Should return true when token is expired even with zero grace",
  );
});

Deno.test("isTokenAboutToExpire - token without exp field", () => {
  // Create token without expiration field
  const noExpToken = createMockToken(0, false);
  const result = isTokenAboutToExpire(noExpToken);
  assertEquals(result, true, "Should return true for token without expiration");
});

Deno.test(
  "isTokenAboutToExpire - invalid token format (not three parts)",
  () => {
    const invalidToken = "invalid.token";
    const result = isTokenAboutToExpire(invalidToken);
    assertEquals(result, true, "Should return true for invalid token format");
  },
);

Deno.test("isTokenAboutToExpire - malformed JWT payload", () => {
  const malformedToken = "eyJhbGciOiJIUzI1NiJ9.not-valid-base64.signature";
  const result = isTokenAboutToExpire(malformedToken);
  assertEquals(result, true, "Should return true for malformed JWT");
});

Deno.test("isTokenAboutToExpire - empty token string", () => {
  const result = isTokenAboutToExpire("");
  assertEquals(result, true, "Should return true for empty token");
});

Deno.test("isTokenAboutToExpire - whitespace only token", () => {
  const result = isTokenAboutToExpire("   ");
  assertEquals(result, true, "Should return true for whitespace-only token");
});

Deno.test("isTokenAboutToExpire - token with missing required fields", () => {
  // Create a token with valid structure but missing workspaceId/userId
  const header = { alg: "HS256", typ: "JWT" };
  const payload = { exp: Math.floor(Date.now() / 1000) + 3600 }; // expires in 1 hour

  const base64urlEncode = (str: string): string => {
    return btoa(str).replace(/=/g, "").replace(/\+/g, "-").replace(/\//g, "_");
  };

  const encodedHeader = base64urlEncode(JSON.stringify(header));
  const encodedPayload = base64urlEncode(JSON.stringify(payload));
  const incompleteToken = `${encodedHeader}.${encodedPayload}.signature`;

  const result = isTokenAboutToExpire(incompleteToken);
  assertEquals(
    result,
    true,
    "Should return true for token missing required fields",
  );
});

Deno.test("isTokenAboutToExpire - very large grace period", () => {
  // Token expires in 1 hour, grace period is 1 week
  const token = createMockToken(60);
  const result = isTokenAboutToExpire(token, 7 * 24 * 60); // 7 days in minutes
  assertEquals(
    result,
    true,
    "Should handle very large grace periods correctly",
  );
});

Deno.test(
  "isTokenAboutToExpire - negative grace period should be treated as zero",
  () => {
    // Token expires in 30 minutes, negative grace period
    const token = createMockToken(30);
    const result = isTokenAboutToExpire(token, -10);
    assertEquals(result, false, "Should treat negative grace period as zero");
  },
);

Deno.test("isTokenAboutToExpire - fractional minutes in grace period", () => {
  // Token expires in 30.5 minutes, grace period is 30.1 minutes
  const token = createMockToken(30.5);
  const result = isTokenAboutToExpire(token, 30.1);
  assertEquals(result, false, "Should handle fractional minutes correctly");
});
