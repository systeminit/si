import { SchemaSpec } from "../bindings/SchemaSpec.ts";
import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { ExpandedPropSpec } from "./props.ts";
import { ExpandedSocketSpec } from "./sockets.ts";

export type ExpandedPkgSpec = Omit<PkgSpec, "schemas"> & {
  schemas: Array<ExpandedSchemaSpec>;
};

export type ExpandedSchemaSpec = Omit<SchemaSpec, "variants"> & {
  variants: Array<ExpandedSchemaVariantSpec>;
};

export type ExpandedSchemaVariantSpec =
  & Omit<
    SchemaVariantSpec,
    "sockets" | "domain" | "secrets" | "secretDefinition" | "resourceValue"
  >
  & {
    sockets: ExpandedSocketSpec[];
    domain: ExpandedPropSpec;
    secrets: ExpandedPropSpec;
    secretDefinition: ExpandedPropSpec | null;
    resourceValue: ExpandedPropSpec;
  };
