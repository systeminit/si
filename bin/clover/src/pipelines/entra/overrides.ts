import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";

// Property-level overrides for specific Microsoft Entra resources
export const ENTRA_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  // Add property overrides here as needed
  // Example:
  // "Microsoft.Graph/users": {
  //   "displayName": widget("text"),
  // },
};

// Schema-level overrides for specific Microsoft Entra resources
export const ENTRA_SCHEMA_OVERRIDES = new Map<
  string,
  SchemaOverrideFn | SchemaOverrideFn[]
>(
  Object.entries({
    // Add schema overrides here as needed
    // Example:
    // "Microsoft.Graph/users": [
    //   addScalarProp("/domain/userPrincipalName", "string", true),
    // ],
  }),
);
