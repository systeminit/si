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
 *
 * This function performs a three-way comparison to identify:
 * 1. **Set operations**: Attributes that are new or have changed values
 * 2. **Unset operations**: Attributes present in existing but not in working set (will be removed)
 * 3. **Subscription operations**: Subscription references that are new or changed
 *
 * The function automatically collapses array element unsets - if all properties of an
 * array element are being removed, it unsets the element itself instead of individual properties.
 *
 * Template tags (`/si/tags/template*`) are never unset to preserve template metadata.
 *
 * @param workingSetAttrs - Desired attributes from the working set component
 * @param existingAttrs - Current attributes on the existing component
 * @returns AttributeDiff containing sets, unsets, and subscription changes
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
 *
 * Subscriptions in System Initiative use the `$source` structure to reference
 * another component's attribute path.
 *
 * @param value - Value to check
 * @returns true if the value is a subscription object with `$source.component` and `$source.path`
 *
 * @example
 * ```ts
 * const sub = { $source: { component: "abc123", path: "/domain/name" } };
 * isSubscription(sub); // true
 *
 * isSubscription("plain string"); // false
 * ```
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
 *
 * Converts a subscription attribute value (with `$source` structure) into
 * a normalized SubscriptionSource object.
 *
 * @param value - Subscription value to extract (must be a valid subscription)
 * @returns SubscriptionSource with component ID, path, and optional func
 *
 * @example
 * ```ts
 * const subValue = {
 *   $source: {
 *     component: "abc123",
 *     path: "/domain/connectionString",
 *     func: "si:normalizeToArray"
 *   }
 * };
 *
 * const source = extractSubscription(subValue);
 * // { component: "abc123", path: "/domain/connectionString", func: "si:normalizeToArray" }
 * ```
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
 *
 * Recursively compares primitives, arrays, and objects for structural equality.
 * Used to determine if attribute values have actually changed between working set
 * and existing components.
 *
 * @param a - First value to compare
 * @param b - Second value to compare
 * @returns true if values are deeply equal, false otherwise
 *
 * @example
 * ```ts
 * deepEqual({ a: 1, b: [2, 3] }, { a: 1, b: [2, 3] }); // true
 * deepEqual({ a: 1 }, { a: 2 }); // false
 * deepEqual([1, 2, 3], [1, 2, 3]); // true
 * deepEqual([1, 2], [1, 2, 3]); // false
 * ```
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
 * Converts an AttributeDiff to a payload suitable for the System Initiative update API.
 *
 * Transforms the structured AttributeDiff into the flat key-value format expected
 * by the SI API's component update endpoint:
 * - Set operations: Include value directly
 * - Unset operations: Use `{ "$source": null }` to clear the attribute
 * - Subscription operations: Use `{ "$source": { component, path, func? } }` format
 *
 * @param diff - AttributeDiff to convert
 * @returns Flat attributes object ready for API submission
 *
 * @example
 * ```ts
 * const diff: AttributeDiff = {
 *   set: new Map([["/domain/region", "us-west-2"]]),
 *   unset: ["/domain/oldConfig"],
 *   subscriptions: new Map([["/domain/db", { component: "abc", path: "/domain/conn" }]])
 * };
 *
 * const payload = attributeDiffToUpdatePayload(diff);
 * // {
 * //   "/domain/region": "us-west-2",
 * //   "/domain/oldConfig": { "$source": null },
 * //   "/domain/db": { "$source": { component: "abc", path: "/domain/conn" } }
 * // }
 * ```
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
 *
 * An empty diff means the working set component is identical to the existing
 * component and no update is needed.
 *
 * @param diff - AttributeDiff to check
 * @returns true if the diff has no sets, unsets, or subscription changes
 *
 * @example
 * ```ts
 * const emptyDiff = { set: new Map(), unset: [], subscriptions: new Map() };
 * isEmptyDiff(emptyDiff); // true
 *
 * const nonEmptyDiff = { set: new Map([["/domain/a", "b"]]), unset: [], subscriptions: new Map() };
 * isEmptyDiff(nonEmptyDiff); // false
 * ```
 */
export function isEmptyDiff(diff: AttributeDiff): boolean {
  return (
    diff.set.size === 0 &&
    diff.unset.length === 0 &&
    diff.subscriptions.size === 0
  );
}
