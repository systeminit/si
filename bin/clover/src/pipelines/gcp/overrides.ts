import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";

export const GCP_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {};

export const GCP_SCHEMA_OVERRIDES: Map<
  string,
  SchemaOverrideFn | SchemaOverrideFn[]
> = new Map();
