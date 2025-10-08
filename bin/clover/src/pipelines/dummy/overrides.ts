import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";

// Dummy provider property overrides (empty for testing)
export const DUMMY_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  // No overrides needed for the dummy provider
};

// Dummy provider schema overrides (empty for testing)
export const DUMMY_SCHEMA_OVERRIDES = new Map<string, SchemaOverrideFn>([
  // No overrides needed for the dummy provider
]);
