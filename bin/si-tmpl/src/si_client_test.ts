import { assertEquals, assertExists } from "@std/assert";
import { jwtDecode } from "jwt-decode";

interface SIJwtPayload {
  workspaceId: string;
  userId: string;
  [key: string]: unknown;
}

Deno.test("JWT decoding - valid token with workspaceId and userId", () => {
  // Create a valid JWT token for testing (header.payload.signature)
  const payload = {
    workspaceId: "test-workspace-123",
    userId: "test-user-456",
    exp: Math.floor(Date.now() / 1000) + 3600, // expires in 1 hour
  };

  // Create a mock JWT (this is just for testing structure, not cryptographically valid)
  const base64Payload = btoa(JSON.stringify(payload));
  const mockJwt =
    `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.${base64Payload}.fake-signature`;

  const decoded = jwtDecode<SIJwtPayload>(mockJwt);

  assertEquals(decoded.workspaceId, "test-workspace-123");
  assertEquals(decoded.userId, "test-user-456");
});

Deno.test("JWT decoding - extracts workspaceId correctly", () => {
  const payload = {
    workspaceId: "workspace-abc-def",
    userId: "user-xyz",
  };

  const base64Payload = btoa(JSON.stringify(payload));
  const mockJwt =
    `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.${base64Payload}.fake-signature`;

  const decoded = jwtDecode<SIJwtPayload>(mockJwt);

  assertExists(decoded.workspaceId);
  assertEquals(decoded.workspaceId, "workspace-abc-def");
});

Deno.test("JWT decoding - extracts userId correctly", () => {
  const payload = {
    workspaceId: "workspace-123",
    userId: "user-789",
  };

  const base64Payload = btoa(JSON.stringify(payload));
  const mockJwt =
    `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.${base64Payload}.fake-signature`;

  const decoded = jwtDecode<SIJwtPayload>(mockJwt);

  assertExists(decoded.userId);
  assertEquals(decoded.userId, "user-789");
});

Deno.test("JWT decoding - handles additional claims", () => {
  const payload = {
    workspaceId: "workspace-123",
    userId: "user-456",
    email: "test@example.com",
    roles: ["admin", "user"],
  };

  const base64Payload = btoa(JSON.stringify(payload));
  const mockJwt =
    `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.${base64Payload}.fake-signature`;

  const decoded = jwtDecode<SIJwtPayload>(mockJwt);

  assertEquals(decoded.workspaceId, "workspace-123");
  assertEquals(decoded.userId, "user-456");
  assertEquals(decoded.email, "test@example.com");
  assertEquals(decoded.roles, ["admin", "user"]);
});
