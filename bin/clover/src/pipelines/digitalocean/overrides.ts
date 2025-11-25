import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";

// Property-level overrides for specific DigitalOcean resources
export const DIGITALOCEAN_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  // Add property overrides here as needed
  // Example:
  // "DigitalOcean/droplets": {
  //   "name": widget("text"),
  // },
};

// Schema-level overrides for specific DigitalOcean resources
export const DIGITALOCEAN_SCHEMA_OVERRIDES = new Map<
  string,
  SchemaOverrideFn | SchemaOverrideFn[]
>(
  Object.entries({
    // Add schema overrides here as needed
    // Example:
    // "DigitalOcean/droplets": [
    //   addScalarProp("/domain/region", "string", true),
    // ],
  }),
);
