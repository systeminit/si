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

import type { TemplateContext } from "./context.ts";
import type { ExistingSetComponent } from "./converge_types.ts";
import { queryExistingSet } from "./existing_set.ts";
import { buildPendingChanges } from "./pending_changes.ts";
import { rewriteSubscriptions } from "./subscriptions.ts";
import { topologicalSort } from "./topology.ts";
import { executeChanges, getOrCreateChangeSet } from "./execute.ts";

/**
 * Main orchestrator for the template converge pipeline.
 *
 * This function coordinates the entire convergence process:
 * 1. Gets or creates the change set to work in
 * 2. Queries existing components in that change set with template tags
 * 3. Computes creates/updates/deletes by comparing working set vs existing
 * 4. Rewrites subscriptions from working set IDs to SI component IDs
 * 5. Orders changes topologically based on dependencies
 * 6. Pretty prints the change plan
 * 7. Executes changes (unless in dry-run mode)
 *
 * @param ctx - Template context with working set and configuration
 * @param dryRun - If true, shows plan but doesn't execute changes
 */
export async function convergeTemplate(
  ctx: TemplateContext,
  dryRun: boolean,
): Promise<void> {
  ctx.logger.debug("Template: {name}", { name: ctx.name() });
  ctx.logger.debug("Invocation key: {key}", { key: ctx.invocationKey() });
  ctx.logger.debug("Change set: {changeSet}", { changeSet: ctx.changeSet() });
  ctx.logger.debug("Dry run: {dryRun}", { dryRun });

  const apiConfig = ctx.apiConfig();
  const workspaceId = ctx.workspaceId();

  // Get or create change set
  const changeSetName = ctx.changeSet() as string;
  ctx.logger.info("Getting or creating change set: {name}", {
    name: changeSetName,
  });
  const changeSetId = await getOrCreateChangeSet(
    apiConfig,
    workspaceId,
    changeSetName,
    ctx,
  );
  ctx.logger.debug("Using change set ID: {changeSetId}", { changeSetId });

  // Query existing set from the change set
  const existingSet = await queryExistingSet(ctx, changeSetId);

  // Normalize working set subscriptions to match existing component IDs
  normalizeWorkingSetSubscriptions(ctx, existingSet);

  // Build pending changes
  ctx.logger.info("Computing delta");
  let pending = buildPendingChanges(ctx, existingSet);

  // Rewrite subscriptions
  ctx.logger.debug("Rewriting subscriptions");
  pending = rewriteSubscriptions(ctx, pending);

  // Topological sort
  ctx.logger.debug("Ordering changes by dependencies");
  const orderedChanges = topologicalSort(ctx, pending);

  // Execute (dry run or actual)
  await executeChanges(ctx, orderedChanges, changeSetId, existingSet, dryRun);
}

/**
 * Normalizes subscriptions in the working set to use SI component IDs instead of
 * working set IDs. This ensures that when we compare working set components to
 * existing components, subscriptions that point to the same component are recognized
 * as identical.
 *
 * For each component in the working set:
 * - Find its corresponding existing component (if any)
 * - For each subscription in the working set component
 * - If it references a working set ID that has an existing component
 * - Rewrite it to use the existing component's SI ID
 *
 * @param ctx - Template context with working set
 * @param existingSet - Existing components from SI with template tags
 */
function normalizeWorkingSetSubscriptions(
  ctx: TemplateContext,
  existingSet: ExistingSetComponent[],
): void {
  const workingSet = ctx.workingSet();
  if (!workingSet) {
    return;
  }

  // Build mapping from workingSet ID to SI component ID
  const wsIdToSiId = new Map<string, string>();
  for (const existing of existingSet) {
    wsIdToSiId.set(existing.templateWorkingSetId, existing.id);
  }

  // Rewrite subscriptions in each working set component
  for (const component of workingSet) {
    for (const [path, value] of Object.entries(component.attributes)) {
      if (isSubscription(value)) {
        const v = value as {
          $source: { component: string; path: string; func?: string };
        };
        const componentRef = v.$source.component;

        // Check if this references a working set ID that we have an SI ID for
        if (wsIdToSiId.has(componentRef)) {
          const siId = wsIdToSiId.get(componentRef)!;
          component.attributes[path] = {
            $source: {
              component: siId,
              path: v.$source.path,
              ...(v.$source.func && { func: v.$source.func }),
            },
          };
        }
      }
    }
  }
}

/**
 * Checks if a value represents a subscription to another component's attribute.
 */
// deno-lint-ignore no-explicit-any
function isSubscription(value: any): boolean {
  return (
    typeof value === "object" &&
    value !== null &&
    "$source" in value &&
    typeof value.$source === "object" &&
    value.$source !== null &&
    "component" in value.$source &&
    "path" in value.$source
  );
}
