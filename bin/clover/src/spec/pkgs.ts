import { SchemaSpec } from "../bindings/SchemaSpec.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { ExpandedPropSpec, ExpandedPropSpecFor } from "./props.ts";
import { Extend } from "../extend.ts";
import { SchemaVariantSpecData } from "../bindings/SchemaVariantSpecData.ts";
import { CfSchema } from "../cfDb.ts";

export type ExpandedPkgSpec = Extend<
  PkgSpec,
  {
    schemas: [ExpandedSchemaSpec]; // Array of exactly one schema
  }
>;

export type ExpandedSchemaSpec = Extend<
  SchemaSpec,
  {
    variants: [ExpandedSchemaVariantSpec]; // Exactly one schema variant
  }
>;

export type ExpandedSchemaVariantSpec = Omit<
  Extend<
    SchemaVariantSpec,
    {
      data: NonNullable<SchemaVariantSpecData>;
      domain: ExpandedPropSpecFor["object"];
      secrets: ExpandedPropSpecFor["object"];
      secretDefinition: ExpandedPropSpec | null;
      resourceValue: ExpandedPropSpecFor["object"];
      cfSchema: CfSchema;
    }
  >,
  "sockets"
>;
