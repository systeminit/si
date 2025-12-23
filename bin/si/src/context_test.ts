/**
 * Tests for Context interceptor logging
 */

import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { printPayload } from "./context.ts";

// Mock logger for testing
const mockLogs: string[] = [];
const mockLogger = {
  trace: (msg: string) => mockLogs.push(msg),
};

Deno.test("printPayload - handles array of member objects", () => {
  mockLogs.length = 0;

  const response = {
    data: [
      { userId: "123", email: "user@example.com", role: "OWNER", nickname: "User" },
      { userId: "456", email: "user2@example.com", role: "APPROVER", nickname: "User2" },
    ],
  };

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 3);
  assertEquals(mockLogs[0], "    [Array with 2 items]");
  assertEquals(mockLogs[1].includes("userId=123"), true);
  assertEquals(mockLogs[1].includes("email=user@example.com"), true);
  assertEquals(mockLogs[2].includes("userId=456"), true);
});

Deno.test("printPayload - handles simple object", () => {
  mockLogs.length = 0;

  const response = {
    data: {
      email: "user@example.com",
      workspaceId: "abc123",
    },
  };

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 2);
  assertEquals(mockLogs[0], "    email: user@example.com");
  assertEquals(mockLogs[1], "    workspaceId: abc123");
});

Deno.test("printPayload - handles object with array property", () => {
  mockLogs.length = 0;

  const response = {
    data: {
      members: [1, 2, 3],
      count: 3,
    },
  };

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 2);
  assertEquals(mockLogs[0], "    members: [3 items]");
  assertEquals(mockLogs[1], "    count: 3");
});

Deno.test("printPayload - handles object with nested object", () => {
  mockLogs.length = 0;

  const response = {
    data: {
      user: { name: "John" },
      role: "admin",
    },
  };

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 2);
  assertEquals(mockLogs[0], "    user: [object]");
  assertEquals(mockLogs[1], "    role: admin");
});

Deno.test("printPayload - handles empty array", () => {
  mockLogs.length = 0;

  const response = {
    data: [],
  };

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 1);
  assertEquals(mockLogs[0], "    [Array with 0 items]");
});

Deno.test("printPayload - handles array with more than 3 items", () => {
  mockLogs.length = 0;

  const response = {
    data: [
      { id: 1 },
      { id: 2 },
      { id: 3 },
      { id: 4 },
      { id: 5 },
    ],
  };

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 5); // header + 3 items + "... more items"
  assertEquals(mockLogs[0], "    [Array with 5 items]");
  assertEquals(mockLogs[4], "      ... 2 more items");
});

Deno.test("printPayload - handles multiline string data", () => {
  mockLogs.length = 0;

  const response = {
    data: "line1\nline2\nline3",
  };

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 3);
  assertEquals(mockLogs[0], "    line1");
  assertEquals(mockLogs[1], "    line2");
  assertEquals(mockLogs[2], "    line3");
});

Deno.test("printPayload - handles simple string data", () => {
  mockLogs.length = 0;

  const response = {
    data: "simple string without newlines",
  };

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 1);
  assertEquals(mockLogs[0], "    simple string without newlines");
});

Deno.test("printPayload - handles null values in object", () => {
  mockLogs.length = 0;

  const response = {
    data: {
      name: "test",
      value: null,
    },
  };

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 2);
  assertEquals(mockLogs[0], "    name: test");
  assertEquals(mockLogs[1], "    value: null");
});

Deno.test("printPayload - handles undefined/no data", () => {
  mockLogs.length = 0;

  const response = {};

  printPayload(response, mockLogger);

  assertEquals(mockLogs.length, 0);
});
