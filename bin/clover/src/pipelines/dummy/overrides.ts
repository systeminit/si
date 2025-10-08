import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import {
  arnProp,
  policyDocumentProp,
  propForOverride,
  stringPropForOverride,
  suggest,
} from "../generic/overrides.ts";

// Dummy provider property overrides - simple examples to test the generic system
export const DUMMY_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  "Dummy::Server": {
    region: suggest("Dummy::Region", "name"),
    serverArn: arnProp("Dummy::Server", "arn"),
  },
  
  "Dummy::Database": {
    serverId: [
      suggest("Dummy::Server", "id"),
      suggest("Dummy::Server", "name"),
    ],
    accessPolicy: policyDocumentProp,
  },

  ".*": {
    ".*Id": suggest("Dummy::Server", "id"),
    ".*Name": suggest("Dummy::Server", "name"),
    ".*Policy": policyDocumentProp,
    ".*Arn": arnProp("Dummy::Server"),
  },
};

// Dummy provider schema overrides - simple property modifications
export const DUMMY_SCHEMA_OVERRIDES = new Map<string, SchemaOverrideFn>([
  [
    "Dummy::Server",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const nameProp = propForOverride(variant.domain, "name");
      nameProp.data.widgetKind = "Text";

      const sizeProp = stringPropForOverride(variant.domain, "size");
      sizeProp.data.widgetKind = "ComboBox";
      sizeProp.data.defaultValue = "medium";
    },
  ],
  [
    "Dummy::Database", 
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const engineProp = stringPropForOverride(variant.domain, "engine");
      engineProp.data.widgetKind = "ComboBox";
      engineProp.data.inputs = [];
      engineProp.data.funcUniqueId = null;
    },
  ],
]);