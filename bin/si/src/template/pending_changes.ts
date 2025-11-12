import type { TemplateComponent, TemplateContext } from "./context.ts";
import type {
  CreateChange,
  ExistingSetComponent,
  PendingChanges,
  UpdateChange,
} from "./converge_types.ts";
import {
  computeAttributeDiff,
  isEmptyDiff,
  isSubscription,
} from "./attribute_diff.ts";

/**
 * Normalizes subscriptions in a component's attributes by translating
 * working set IDs to SI IDs using the existingByWorkingSetId mapping.
 * This allows subscriptions to be compared correctly during diff computation.
 */
function normalizeSubscriptions(
  component: TemplateComponent,
  existingByWorkingSetId: Map<string, ExistingSetComponent>,
): { [key: string]: unknown } {
  const normalized: { [key: string]: unknown } = {};

  for (const [path, value] of Object.entries(component.attributes)) {
    if (isSubscription(value)) {
      // Extract the subscription and translate the component ID
      const sub = value as {
        $source: { component: string; path: string; func?: string };
      };
      const workingSetId = sub.$source.component;
      const existing = existingByWorkingSetId.get(workingSetId);

      if (existing) {
        // Translate working set ID to SI ID
        normalized[path] = {
          $source: {
            component: existing.id,
            path: sub.$source.path,
            ...(sub.$source.func ? { func: sub.$source.func } : {}),
          },
        };
      } else {
        // No translation available, keep as-is
        normalized[path] = value;
      }
    } else {
      // Not a subscription, keep as-is
      normalized[path] = value;
    }
  }

  return normalized;
}

/**
 * Compares the working set against the existing set to determine what
 * changes need to be made (creates, updates, deletes).
 *
 * This function performs a three-way analysis:
 * 1. **Creates**: Working set components without matching existing components
 * 2. **Updates**: Matching components with attribute or name differences
 * 3. **Deletes**: Existing components not present in working set
 *
 * Components are matched using two strategies:
 * - Primary: `/si/tags/templateWorkingSetId` tag (stable component ID)
 * - Fallback: `/si/tags/templateDynamicName` tag (for copied components)
 *
 * @param ctx - Template context with working set and logging
 * @param existingSet - Components currently in System Initiative created by this template
 * @returns PendingChanges object with categorized changes and lookup maps
 */
export function buildPendingChanges(
  ctx: TemplateContext,
  existingSet: ExistingSetComponent[],
): PendingChanges {
  const workingSet = ctx.workingSet() || [];

  // Build lookup maps
  const workingSetById = new Map();
  for (const comp of workingSet) {
    workingSetById.set(comp.id, comp);
  }

  const existingByWorkingSetId = new Map();
  const existingByDynamicName = new Map();
  for (const comp of existingSet) {
    existingByWorkingSetId.set(comp.templateWorkingSetId, comp);

    // Build dynamic name lookup for fallback matching
    const dynamicName = comp.attributes["/si/tags/templateDynamicName"];
    if (dynamicName && typeof dynamicName === "string") {
      existingByDynamicName.set(dynamicName, comp);
    }
  }

  const pending: PendingChanges = {
    creates: [],
    updates: [],
    deletes: [],
    workingSetById,
    existingByWorkingSetId,
    existingByDynamicName,
  };

  // Track which existing components have been matched (by ID or dynamic name)
  // to avoid incorrectly marking them for deletion
  const matchedExisting = new Set<string>();

  ctx.logger.debug("Building pending changes");

  // Determine creates and updates
  for (const wsComp of workingSet) {
    // Two-stage matching: try primary ID lookup, then fallback to dynamic name
    let existing = existingByWorkingSetId.get(wsComp.id);

    // Fallback: check if this is a dynamically created component
    if (!existing) {
      const dynamicName = wsComp.attributes["/si/tags/templateDynamicName"];
      if (dynamicName && typeof dynamicName === "string") {
        existing = existingByDynamicName.get(dynamicName);
        if (existing) {
          // Found via dynamic name - add mapping without mutating component ID
          ctx.logger.debug(
            `Matched dynamic component "{name}" by name (existing ID: {existingId})`,
            { name: wsComp.name, existingId: existing.id },
          );
          // Add to existingByWorkingSetId so subscription rewriting can find it
          // DO NOT mutate wsComp.id - subscriptions may already reference it!
          // The subscription rewriter will translate working set IDs to SI IDs
          existingByWorkingSetId.set(wsComp.id, existing);
          // Mark as matched to prevent deletion
          matchedExisting.add(existing.id);
        }
      }
    } else {
      // Matched by primary ID lookup
      matchedExisting.add(existing.id);
    }

    if (!existing) {
      // Component doesn't exist - CREATE
      ctx.logger.debug(`Create: {name}`, { name: wsComp.name });
      const createChange: CreateChange = {
        type: "create",
        workingSetComponent: wsComp,
        attributes: wsComp.attributes,
        dependencies: [], // Will be filled by subscription rewriter
      };
      pending.creates.push(createChange);
    } else {
      // Component exists - check for UPDATE
      // Normalize subscriptions in working set before comparing
      const normalizedAttrs = normalizeSubscriptions(
        wsComp,
        existingByWorkingSetId,
      );
      const attributeDiff = computeAttributeDiff(
        normalizedAttrs,
        existing.attributes,
      );

      const nameChange = wsComp.name !== existing.name
        ? { from: existing.name, to: wsComp.name }
        : undefined;

      // Only create update if there are actual changes
      if (!isEmptyDiff(attributeDiff) || nameChange) {
        ctx.logger.debug(`Update: {name}`, { name: wsComp.name });
        const updateChange: UpdateChange = {
          type: "update",
          existingComponent: existing,
          workingSetComponent: wsComp,
          attributeDiff,
          nameChange,
          dependencies: [],
        };
        pending.updates.push(updateChange);
      } else {
        ctx.logger.debug(`No changes: {name}`, { name: wsComp.name });
      }
    }
  }

  // Determine deletes
  // Only delete components that were NOT matched (by either ID or dynamic name)
  for (const existing of existingSet) {
    if (!matchedExisting.has(existing.id)) {
      ctx.logger.debug(`Delete: {name}`, { name: existing.name });
      pending.deletes.push({
        type: "delete",
        existingComponent: existing,
      });
    }
  }

  ctx.logger.info(
    `Pending changes: {creates} creates, {updates} updates, {deletes} deletes`,
    {
      creates: pending.creates.length,
      updates: pending.updates.length,
      deletes: pending.deletes.length,
    },
  );

  return pending;
}
