import type { AttributeDiff, SubscriptionSource } from "./converge_types.ts";

/**
 * Detects when all properties of array elements are being unset and
 * collapses them to unset the array element itself.
 *
 * For example, if both "/domain/Tags/3/Key" and "/domain/Tags/3/Value"
 * are being unset, replace them with "/domain/Tags/3".
 *
 * According to the SI API docs:
 * - If only one property is being unset → unset that property individually
 * - If all properties of an array element are being unset → unset the array element itself
 */
export function collapseArrayElementUnsets(
  unsetPaths: string[],
  existingAttrs: { [key: string]: unknown },
): string[] {
  // Match pattern: /path/to/array/N/property
  // Captures: (1) array element path, (2) property name
  const arrayElementPattern = /^(\/[^/]+(?:\/[^/]+)*\/\d+)\/(.+)$/;

  // Group paths by array element prefix
  const arrayElements = new Map<string, Set<string>>();
  const nonArrayPaths: string[] = [];

  for (const path of unsetPaths) {
    const match = path.match(arrayElementPattern);
    if (match) {
      const [, elementPath, property] = match;
      if (!arrayElements.has(elementPath)) {
        arrayElements.set(elementPath, new Set());
      }
      arrayElements.get(elementPath)!.add(property);
    } else {
      nonArrayPaths.push(path);
    }
  }

  // For each array element, check if ALL its properties are being unset
  const result = [...nonArrayPaths];

  for (const [elementPath, unsetProps] of arrayElements.entries()) {
    // Find all properties that exist for this array element in existingAttrs
    const existingProps = new Set<string>();
    const elementPrefix = elementPath + "/";

    for (const existingPath of Object.keys(existingAttrs)) {
      if (existingPath.startsWith(elementPrefix)) {
        const property = existingPath.substring(elementPrefix.length);
        // Only count direct children, not nested paths
        if (!property.includes("/")) {
          existingProps.add(property);
        }
      }
    }

    // If we're unsetting all properties of this array element,
    // unset the element itself
    if (existingProps.size > 0 && existingProps.size === unsetProps.size) {
      result.push(elementPath);
    } else {
      // Otherwise, unset individual properties
      for (const prop of unsetProps) {
        result.push(`${elementPath}/${prop}`);
      }
    }
  }

  return result;
}

/**
 * Computes the difference between working set attributes and existing attributes.
 * Returns a diff containing set operations, unset operations, and subscriptions.
 */
export function computeAttributeDiff(
  workingSetAttrs: { [key: string]: unknown },
  existingAttrs: { [key: string]: unknown },
): AttributeDiff {
  const diff: AttributeDiff = {
    set: new Map(),
    unset: [],
    subscriptions: new Map(),
  };

  // Find attributes to set or update
  for (const [path, value] of Object.entries(workingSetAttrs)) {
    // Check if it's a subscription
    if (isSubscription(value)) {
      const existingValue = existingAttrs[path];

      // Check if existing attribute is also a subscription and if they're identical
      if (isSubscription(existingValue)) {
        const workingSub = extractSubscription(value);
        const existingSub = extractSubscription(existingValue);

        // Only add to diff if subscriptions differ
        if (!deepEqual(workingSub, existingSub)) {
          diff.subscriptions.set(path, workingSub);
        }
      } else {
        // Existing is not a subscription, so this is a change
        diff.subscriptions.set(path, extractSubscription(value));
      }
      continue;
    }

    // Check if value differs from existing
    const existingValue = existingAttrs[path];
    if (!deepEqual(value, existingValue)) {
      diff.set.set(path, value);
    }
  }

  // Find attributes to unset (in existing but not in working)
  for (const path of Object.keys(existingAttrs)) {
    if (!(path in workingSetAttrs)) {
      // Don't unset template tags - we manage those
      if (!path.startsWith("/si/tags/template")) {
        diff.unset.push(path);
      }
    }
  }

  // Collapse array element unsets: if all properties of an array element
  // are being unset, unset the element itself instead of individual properties
  diff.unset = collapseArrayElementUnsets(diff.unset, existingAttrs);

  return diff;
}

/**
 * Checks if a value represents a subscription to another component's attribute.
 */
export function isSubscription(value: unknown): boolean {
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

/**
 * Extracts subscription information from a subscription value.
 */
export function extractSubscription(value: unknown): SubscriptionSource {
  const v = value as {
    $source: { component: string; path: string; func?: string };
  };
  return {
    component: v.$source.component,
    path: v.$source.path,
    func: v.$source.func,
  };
}

/**
 * Performs deep equality comparison between two values.
 */
export function deepEqual(a: unknown, b: unknown): boolean {
  // Handle primitives
  if (a === b) return true;
  if (a == null || b == null) return false;
  if (typeof a !== typeof b) return false;

  // Handle arrays
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    return a.every((val, idx) => deepEqual(val, b[idx]));
  }

  // Handle objects
  if (typeof a === "object" && typeof b === "object") {
    const objA = a as Record<string, unknown>;
    const objB = b as Record<string, unknown>;
    const keysA = Object.keys(objA).sort();
    const keysB = Object.keys(objB).sort();
    if (!deepEqual(keysA, keysB)) return false;
    return keysA.every((key) => deepEqual(objA[key], objB[key]));
  }

  return false;
}

/**
 * Converts an AttributeDiff to a payload suitable for the SI update API.
 */
export function attributeDiffToUpdatePayload(
  diff: AttributeDiff,
): { [key: string]: unknown } {
  const payload: { [key: string]: unknown } = {};

  // Set operations
  for (const [path, value] of diff.set.entries()) {
    payload[path] = value;
  }

  // Unset operations
  for (const path of diff.unset) {
    payload[path] = { "$source": null };
  }

  // Subscription operations
  for (const [path, sub] of diff.subscriptions.entries()) {
    payload[path] = {
      "$source": {
        component: sub.component,
        path: sub.path,
        ...(sub.func && { func: sub.func }),
      },
    };
  }

  return payload;
}

/**
 * Checks if an AttributeDiff contains any changes.
 */
export function isEmptyDiff(diff: AttributeDiff): boolean {
  return (
    diff.set.size === 0 &&
    diff.unset.length === 0 &&
    diff.subscriptions.size === 0
  );
}
