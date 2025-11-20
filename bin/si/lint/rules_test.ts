import { assertEquals } from "@std/assert";
import plugin from "../lint/mod.ts";

Deno.test("no-deno-env-get rule - detects Deno.env.get() calls", () => {
  const code = `
    const apiKey = Deno.env.get("API_KEY");
    const token = Deno.env.get("TOKEN");
  `;

  const diagnostics = Deno.lint.runPlugin(
    plugin,
    "test.ts",
    code,
  );

  // Should find 2 violations
  assertEquals(diagnostics.length, 2);
  assertEquals(diagnostics[0].id, "si-rules/no-deno-env-get");
  assertEquals(diagnostics[1].id, "si-rules/no-deno-env-get");
  assertEquals(
    diagnostics[0].message,
    "Direct usage of Deno.env.get() is not allowed. Use proper CLI parsing and configuration injection methods instead.",
  );
});

Deno.test("no-deno-env-get rule - allows other Deno API calls", () => {
  const code = `
    const cwd = Deno.cwd();
    const args = Deno.args;
    const value = Deno.env.set("KEY", "value");
  `;

  const diagnostics = Deno.lint.runPlugin(
    plugin,
    "test.ts",
    code,
  );

  // Should find no violations
  assertEquals(diagnostics.length, 0);
});
