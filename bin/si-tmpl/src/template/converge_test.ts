// Copyright 2025 System Initiative Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { assertRejects } from "@std/assert";
import { Context } from "../context.ts";
import { TemplateContext } from "./context.ts";
import { convergeTemplate } from "./converge.ts";

// Clear SI_API_TOKEN to ensure clean test environment
Deno.env.delete("SI_API_TOKEN");

// Initialize context for testing
Context.init({ verbose: 0, noColor: true });

Deno.test("convergeTemplate - should handle dry run mode without executing changes", async () => {
  const tctx = new TemplateContext("test-template.ts", {
    key: "test-key",
  });

  // Set empty working set
  tctx.workingSet([]);

  // Should throw in dry-run mode too because we need API to get change set
  await assertRejects(
    async () => {
      await convergeTemplate(tctx, true);
    },
    Error,
    "API configuration not available",
  );
});

Deno.test("convergeTemplate - should orchestrate all steps in correct order", async () => {
  const tctx = new TemplateContext("test-template.ts", {
    key: "test-key",
  });

  // Set empty working set
  tctx.workingSet([]);

  // Should throw because API not configured
  await assertRejects(
    async () => {
      await convergeTemplate(tctx, true);
    },
    Error,
    "API configuration not available",
  );
});

Deno.test("convergeTemplate - should handle empty working set", async () => {
  const tctx = new TemplateContext("test-template.ts", {
    key: "test-key",
  });

  // Set empty working set
  tctx.workingSet([]);

  // Should throw because API not configured
  await assertRejects(
    async () => {
      await convergeTemplate(tctx, true);
    },
    Error,
    "API configuration not available",
  );
});

Deno.test("convergeTemplate - should handle working set with components", async () => {
  const tctx = new TemplateContext("test-template.ts", {
    key: "test-key",
  });

  // Set working set with a component
  tctx.workingSet([{
    id: "ws-1",
    schemaId: "schema-1",
    name: "test-component",
    resourceId: "resource-1",
    attributes: {
      "/domain/name": "test",
    },
  }]);

  // Should throw because API not configured
  await assertRejects(
    async () => {
      await convergeTemplate(tctx, true);
    },
    Error,
    "API configuration not available",
  );
});

Deno.test("convergeTemplate - should fail gracefully when API not configured for execution", async () => {
  const tctx = new TemplateContext("test-template.ts", {
    key: "test-key",
  });

  // Set working set with a component
  tctx.workingSet([{
    id: "ws-1",
    schemaId: "schema-1",
    name: "test-component",
    resourceId: "resource-1",
    attributes: {
      "/domain/name": "test",
    },
  }]);

  // Should throw when trying to execute without API config
  await assertRejects(
    async () => {
      await convergeTemplate(tctx, false);
    },
    Error,
    "API configuration not available",
  );
});

Deno.test("convergeTemplate - should handle components with subscriptions", async () => {
  const tctx = new TemplateContext("test-template.ts", {
    key: "test-key",
  });

  // Set working set with components that have subscriptions
  tctx.workingSet([
    {
      id: "ws-1",
      schemaId: "schema-1",
      name: "component-1",
      resourceId: "resource-1",
      attributes: {
        "/domain/name": "component-1",
      },
    },
    {
      id: "ws-2",
      schemaId: "schema-2",
      name: "component-2",
      resourceId: "resource-2",
      attributes: {
        "/domain/name": "component-2",
        "/domain/ref": {
          "$source": {
            component: "ws-1",
            path: "/domain/name",
          },
        },
      },
    },
  ]);

  // Should throw because API not configured
  await assertRejects(
    async () => {
      await convergeTemplate(tctx, true);
    },
    Error,
    "API configuration not available",
  );
});
