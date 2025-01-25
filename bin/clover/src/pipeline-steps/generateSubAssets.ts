import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";
import {
  createDefaultProp,
  ExpandedPropSpec,
  isExpandedPropSpec,
} from "../spec/props.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { attrFuncInputSpecFromSocket } from "../spec/sockets.ts";
import { createSocket } from "../spec/sockets.ts";
import { attrFuncInputSpecFromProp } from "../spec/sockets.ts";
import { getSiFuncId } from "../spec/siFuncs.ts";

export function generateSubAssets(specs: PkgSpec[]): PkgSpec[] {
  for (const spec of specs) {
    const schema = spec.schemas[0];
    if (!schema) {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: missing schema`,
      );
    }
    const schemaVariant = schema.variants[0];

    if (!schemaVariant) {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: missing variant`,
      );
      continue;
    }

    const domain = schemaVariant.domain;
    if (!isExpandedPropSpec(domain)) {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: domain has no metadata`,
      );
      continue;
    }
    if (domain.kind !== "object") {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: domain is not object`,
      );
      continue;
    }

    for (const prop of domain.entries) {
      if (prop.kind === "array" && prop.typeProp.kind === "object") {
        const objName = prop.name.endsWith("s")
          ? prop.name.slice(0, -1)
          : prop.name;
        const name = `${spec.name}::${objName}`;
        const variantId = ulid();

        const newDomain = _.cloneDeep(domain);

        // recreate ["root", "domain", etc.]
        fixPropPath(newDomain.entries, newDomain.metadata.propPath);

        // set the parent prop to have an input socket for this new asset
        const propInputSocket = createSocket(objName, "input", "many");
        if (prop.data) {
          prop.data.inputs = [
            attrFuncInputSpecFromSocket(propInputSocket, "value"),
          ];
          prop.data.funcUniqueId = getSiFuncId("si:normalizeToArray");
          schemaVariant.sockets.push(propInputSocket);
        }

        // output the domain of this new spec
        const newSpecOutputSocket = createSocket(objName, "output", "one");
        if (newSpecOutputSocket.data) {
          newSpecOutputSocket.data.funcUniqueId = getSiFuncId("si:identity");
          newSpecOutputSocket.inputs.push(attrFuncInputSpecFromProp(newDomain));
        }

        const variantData = _.cloneDeep(schemaVariant.data);
        const variant: SchemaVariantSpec = {
          ...schemaVariant,
          data: {
            ...variantData,
            displayName: name,
            funcUniqueId: ulid(),
            description: prop.typeProp.data?.documentation ?? "",
          },
          domain: newDomain,
          actionFuncs: [],
          leafFunctions: [],
          managementFuncs: [],
          resourceValue: createDefaultProp("resource_value"),
          sockets: [newSpecOutputSocket],
          secrets: createDefaultProp("secrets"),
          uniqueId: variantId,
        };

        const schemaData = _.cloneDeep(schema.data);
        const newSpec: PkgSpec = {
          ...spec,
          name,
          description: prop.typeProp.data?.documentation ?? "",
          funcs: [],
          schemas: [{
            ...schema,
            name,
            data: {
              ...schemaData,
              name,
              defaultSchemaVariant: variantId,
            },
            uniqueId: ulid(),
            variants: [variant],
          }],
        };

        specs.push(newSpec);
      }
    }
  }
  return specs;
}

function fixPropPath(props: ExpandedPropSpec[], parentPath: string[]) {
  for (const prop of props) {
    prop.metadata.propPath = [...parentPath, prop.name];
    if (prop.kind === "object") {
      fixPropPath(prop.entries as ExpandedPropSpec[], prop.metadata.propPath);
    }
  }
}
