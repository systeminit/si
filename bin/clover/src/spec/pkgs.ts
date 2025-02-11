import { SchemaSpec } from "../bindings/SchemaSpec.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { ExpandedPropSpec, ExpandedPropSpecFor } from "./props.ts";
import { ExpandedSocketSpec } from "./sockets.ts";
import { Extend } from "../extend.ts";
import { SchemaVariantSpecData } from "../bindings/SchemaVariantSpecData.ts";

export type ExpandedPkgSpec = Extend<PkgSpec, {
  schemas: [ExpandedSchemaSpec]; // Array of exactly one schema
}>;

export type ExpandedSchemaSpec = Extend<SchemaSpec, {
  variants: [ExpandedSchemaVariantSpec]; // Exactly one schema variant
}>;

export type ExpandedSchemaVariantSpec = Extend<SchemaVariantSpec, {
  data: NonNullable<SchemaVariantSpecData>;
  sockets: ExpandedSocketSpec[];
  domain: ExpandedPropSpecFor["object"];
  secrets: ExpandedPropSpec;
  secretDefinition: ExpandedPropSpec | null;
  resourceValue: ExpandedPropSpecFor["object"];
}>;
