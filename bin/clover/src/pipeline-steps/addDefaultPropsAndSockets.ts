import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "lodash";
import {
  createObjectProp,
  createScalarProp,
  isExpandedPropSpec,
} from "../spec/props.ts";
import { getSiFuncId } from "../spec/siFuncs.ts";
import { attrFuncInputSpecFromSocket, createSocket } from "../spec/sockets.ts";

export function addDefaultPropsAndSockets(specs: PkgSpec[]): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];

  for (const spec of specs) {
    const schema = spec.schemas[0];

    if (!schema) {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: missing schema`,
      );
      continue;
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

    // Extra prop
    const extraProp = createObjectProp("extra", domain.metadata.propPath);

    // Create PropUsageMap
    {
      const propUsageMapProp = createScalarProp(
        "PropUsageMap",
        "string",
        extraProp.metadata.propPath,
      );
      const propUsageMap = {
        createOnly: [] as string[],
        updatable: [] as string[],
      };

      const queue = _.cloneDeep(domain.entries);

      while (queue.length > 0) {
        const prop = queue.pop();
        if (!prop || !isExpandedPropSpec(prop)) break;

        if (prop.metadata.createOnly) {
          propUsageMap.createOnly.push(prop.name);
        } else if (!prop.metadata.readOnly) {
          propUsageMap.updatable.push(prop.name);
        }

        if (prop.kind === "object") {
          prop.entries.forEach((p) => queue.unshift(p));
        }
      }

      propUsageMapProp.data.defaultValue = JSON.stringify(propUsageMap);
      propUsageMapProp.data.hidden = true;
      extraProp.entries.push(propUsageMapProp);
    }

    // Create Region prop and socket
    {
      const regionSocket = createSocket("Region", "input", "one");
      schemaVariant.sockets.push(regionSocket);

      const regionProp = createScalarProp(
        "Region",
        "string",
        extraProp.metadata.propPath,
      );
      regionProp.data.inputs = [
        attrFuncInputSpecFromSocket(regionSocket),
      ];
      regionProp.data.funcUniqueId = getSiFuncId("si:identity");
      extraProp.entries.push(regionProp);
    }

    // Create AwsResourceType
    {
      const typeProp = createScalarProp(
        "AwsResourceType",
        "string",
        extraProp.metadata.propPath,
      );

      typeProp.data.defaultValue = schema.name;
      typeProp.data.hidden = true;

      extraProp.entries.push(typeProp);
    }

    // Create Credential prop and socket under root/secrets
    {
      const credSocket = createSocket("AWS Credential", "input", "one");
      schemaVariant.sockets.push(credSocket);

      const credProp = createScalarProp(
        "AWS Credential",
        "string",
        extraProp.metadata.propPath,
      );
      credProp.data.widgetKind = "Secret";
      credProp.data.widgetOptions = [{
        "label": "secretKind",
        "value": "AWS Credential",
      }];
      credProp.data.inputs = [
        attrFuncInputSpecFromSocket(credSocket),
      ];
      credProp.data.funcUniqueId = getSiFuncId("si:identity");
      if (schemaVariant.secrets.kind !== "object") {
        console.log(
          `Could not generate default props and sockets for ${spec.name}: secrets is not object`,
        );
        continue;
      }

      schemaVariant.secrets.entries.push(credProp);
    }

    // Finalize
    domain.entries.push(extraProp);
    newSpecs.push(spec);
  }

  return newSpecs;
}
