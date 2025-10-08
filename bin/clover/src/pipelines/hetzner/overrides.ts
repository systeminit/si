import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";

// Hetzner-specific property overrides (empty for now)
export const HETZNER_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  // Add Hetzner-specific property overrides here as needed
  // Example:
  // "Hetzner::Cloud::Server": {
  //   LocationsProp: makeDropdownOverride,
  // },
};

// Hetzner-specific schema overrides (empty for now)
export const HETZNER_SCHEMA_OVERRIDES = new Map<string, SchemaOverrideFn>([
  // Add Hetzner-specific schema overrides here as needed
  // Example:
  // ["Hetzner::Cloud::Server", serverSchemaOverride],
]);