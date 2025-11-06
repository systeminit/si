import type { TemplateContext } from "./context.ts";
import type { TemplateComponent } from "./context.ts";
import type {
  AttributeDiff,
  PendingChanges,
  SubscriptionSource,
} from "./converge_types.ts";

/**
 * Rewrites subscriptions in pending changes from workingSet IDs to SI component IDs.
 * This ensures that subscriptions reference the correct components after they are
 * created or updated in System Initiative.
 */
export function rewriteSubscriptions(
  ctx: TemplateContext,
  pending: PendingChanges,
): PendingChanges {
  ctx.logger.trace("Rewriting subscriptions");

  // Track which workingSet IDs map to which SI component IDs
  const workingSetToSiId = new Map<string, string>();

  // Existing components already have SI IDs
  for (const [wsId, existing] of pending.existingByWorkingSetId.entries()) {
    workingSetToSiId.set(wsId, existing.id);
  }

  // Process creates
  for (const create of pending.creates) {
    const result = rewriteAttributeSubscriptions(
      create.attributes,
      workingSetToSiId,
      pending.workingSetById,
    );
    create.attributes = result.attributes;
    create.dependencies = result.dependencies;
  }

  // Process updates
  for (const update of pending.updates) {
    const result = rewriteDiffSubscriptions(
      update.attributeDiff,
      workingSetToSiId,
      pending.workingSetById,
    );
    update.attributeDiff = result.diff;
    update.dependencies = result.dependencies;
  }

  ctx.logger.trace("Subscription rewriting complete");
  return pending;
}

/**
 * Result of rewriting subscriptions in attributes
 */
interface RewriteResult {
  attributes: { [key: string]: unknown };
  dependencies: string[]; // workingSet IDs this depends on
}

/**
 * Rewrites subscriptions in a flat attributes object.
 */
function rewriteAttributeSubscriptions(
  attributes: { [key: string]: unknown },
  workingSetToSiId: Map<string, string>,
  workingSetById: Map<string, TemplateComponent>,
): RewriteResult {
  const rewritten: { [key: string]: unknown } = {};
  const dependencies = new Set<string>();

  for (const [path, value] of Object.entries(attributes)) {
    if (isSubscription(value)) {
      const v = value as {
        $source: { component: string; path: string; func?: string };
      };
      const componentRef = v.$source.component;

      // Check if it's a workingSet ID
      if (workingSetById.has(componentRef)) {
        const siId = workingSetToSiId.get(componentRef);

        if (siId) {
          // Already exists in SI - use that ID
          rewritten[path] = {
            $source: {
              component: siId,
              path: v.$source.path,
              ...(v.$source.func && { func: v.$source.func }),
            },
          };
        } else {
          // Will be created - add as dependency
          dependencies.add(componentRef);
          // Keep as workingSet ID for now - will resolve after create
          rewritten[path] = value;
        }
      } else {
        // Reference by name or already SI ID - keep as-is
        rewritten[path] = value;
      }
    } else {
      rewritten[path] = value;
    }
  }

  return {
    attributes: rewritten,
    dependencies: Array.from(dependencies),
  };
}

/**
 * Result of rewriting subscriptions in a diff
 */
interface DiffRewriteResult {
  diff: AttributeDiff;
  dependencies: string[];
}

/**
 * Rewrites subscriptions in an AttributeDiff.
 */
function rewriteDiffSubscriptions(
  diff: AttributeDiff,
  workingSetToSiId: Map<string, string>,
  workingSetById: Map<string, TemplateComponent>,
): DiffRewriteResult {
  const newSubscriptions = new Map<string, SubscriptionSource>();
  const dependencies = new Set<string>();

  for (const [path, sub] of diff.subscriptions.entries()) {
    const componentRef = sub.component;

    if (workingSetById.has(componentRef)) {
      const siId = workingSetToSiId.get(componentRef);

      if (siId) {
        newSubscriptions.set(path, {
          ...sub,
          component: siId,
        });
      } else {
        dependencies.add(componentRef);
        newSubscriptions.set(path, sub);
      }
    } else {
      newSubscriptions.set(path, sub);
    }
  }

  return {
    diff: {
      ...diff,
      subscriptions: newSubscriptions,
    },
    dependencies: Array.from(dependencies),
  };
}

/**
 * Checks if a value represents a subscription.
 */
function isSubscription(value: unknown): boolean {
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
