// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ActionFuncSpec } from "./ActionFuncSpec";
import type { AuthenticationFuncSpec } from "./AuthenticationFuncSpec";
import type { LeafFunctionSpec } from "./LeafFunctionSpec";
import type { ManagementFuncSpec } from "./ManagementFuncSpec";
import type { PropSpec } from "./PropSpec";
import type { RootPropFuncSpec } from "./RootPropFuncSpec";
import type { SchemaVariantSpecData } from "./SchemaVariantSpecData";
import type { SiPropFuncSpec } from "./SiPropFuncSpec";
import type { SocketSpec } from "./SocketSpec";

export type SchemaVariantSpec = {
  version: string;
  data: SchemaVariantSpecData | null;
  uniqueId: string | null;
  deleted: boolean;
  isBuiltin: boolean;
  actionFuncs: Array<ActionFuncSpec>;
  authFuncs: Array<AuthenticationFuncSpec>;
  leafFunctions: Array<LeafFunctionSpec>;
  sockets: Array<SocketSpec>;
  siPropFuncs: Array<SiPropFuncSpec>;
  managementFuncs: Array<ManagementFuncSpec>;
  domain: PropSpec;
  secrets: PropSpec;
  secretDefinition: PropSpec | null;
  resourceValue: PropSpec;
  rootPropFuncs: Array<RootPropFuncSpec>;
};
