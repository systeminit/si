import { assertEquals, assertExists } from "@std/assert";
import OpenAI from "jsr:@openai/openai@^4";
import {
  DEFAULT_MODEL,
  DEFAULT_SCHEMA_TEMPERATURE,
  getClient,
  initClient,
  MAX_CONTENT_LENGTH,
} from "./client.ts";

Deno.test("client.ts - constants have expected values", () => {
  assertEquals(DEFAULT_MODEL, "gpt-4o", "DEFAULT_MODEL should be 'gpt-4o'");
  assertEquals(
    MAX_CONTENT_LENGTH,
    250000,
    "MAX_CONTENT_LENGTH should be 250000",
  );
  assertEquals(
    DEFAULT_SCHEMA_TEMPERATURE,
    0.0,
    "DEFAULT_SCHEMA_TEMPERATURE should be 0.0",
  );
});

Deno.test("client.ts - getClient returns a client instance", () => {
  const client = getClient();
  assertExists(client, "getClient should return a client instance");
  assertEquals(
    client instanceof OpenAI,
    true,
    "Client should be an instance of OpenAI",
  );
});

Deno.test("client.ts - getClient returns the same instance on subsequent calls", () => {
  const client1 = getClient();
  const client2 = getClient();
  assertEquals(
    client1 === client2,
    true,
    "getClient should return the same instance on subsequent calls",
  );
});

Deno.test("client.ts - initClient creates a new client instance", () => {
  const client1 = getClient();
  initClient(); // Reset the client
  const client2 = getClient();
  assertEquals(
    client1 !== client2,
    true,
    "initClient should create a new client instance",
  );
});

Deno.test("client.ts - initClient accepts custom API key", () => {
  // Store the original API key to restore it later
  const origApiKey = Deno.env.get("OPENAI_API_KEY");
  const testApiKey = "test-api-key";

  try {
    // Set up a clean environment for testing
    Deno.env.delete("OPENAI_API_KEY");

    // Initialize with a custom API key
    initClient({ apiKey: testApiKey });

    // We can't directly access the private apiKey property of the OpenAI instance,
    // but we can inspect the instance to verify it's been created with the expected config
    const client = getClient();
    assertExists(
      client,
      "Client should exist after initialization with custom API key",
    );

    // Check if client is an OpenAI instance
    assertEquals(
      client instanceof OpenAI,
      true,
      "Client should be an instance of OpenAI",
    );
  } finally {
    // Restore the original API key
    if (origApiKey) {
      Deno.env.set("OPENAI_API_KEY", origApiKey);
    } else {
      Deno.env.delete("OPENAI_API_KEY");
    }

    // Re-init the client with the original API key
    initClient();
  }
});

// This test makes an actual API call, only run if OPENAI_API_KEY is set
// and the user explicitly wants to test the API integration
if (Deno.env.has("OPENAI_API_KEY") && Deno.env.has("TEST_API_INTEGRATION")) {
  Deno.test("client.ts - integration test with actual API call", async () => {
    const client = getClient();
    const prompt = "Hello, world!";

    // Make a simple API call
    const response = await client.chat.completions.create({
      model: DEFAULT_MODEL,
      messages: [{ role: "user", content: prompt }],
      max_tokens: 50,
    });

    assertExists(response, "API response should exist");
    assertExists(
      response.choices[0].message.content,
      "API response should include content",
    );
    assertEquals(
      typeof response.choices[0].message.content,
      "string",
      "Response content should be a string",
    );
  });
}
