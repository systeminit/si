import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";
import {
  copyPropWithNewIds,
  createDefaultProp,
  ExpandedPropSpec,
  generatePropHash,
  isExpandedPropSpec,
} from "../spec/props.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { attrFuncInputSpecFromSocket } from "../spec/sockets.ts";
import { createSocket } from "../spec/sockets.ts";
import { attrFuncInputSpecFromProp } from "../spec/sockets.ts";
import { getSiFuncId } from "../spec/siFuncs.ts";

export function generateSubAssets(incomingSpecs: PkgSpec[]): PkgSpec[] {
  const outgoingSpecs = [] as PkgSpec[];
  const newSpecsByHash = {} as Record<
    string,
    { spec: PkgSpec; names: string[] }
  >;

  for (const spec of incomingSpecs) {
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

        const newDomainWithOldIds = _.cloneDeep(domain);
        newDomainWithOldIds.entries = prop.typeProp.entries;

        // recreate ["root", "domain", etc.]
        fixPropPath(
          newDomainWithOldIds.entries,
          newDomainWithOldIds.metadata.propPath,
        );

        const newDomain = copyPropWithNewIds(newDomainWithOldIds);

        const hash = generatePropHash(newDomain);

        const maybeExistingSubAsset = newSpecsByHash[hash];
        if (maybeExistingSubAsset) {
          maybeExistingSubAsset.names.push(name);
          continue;
        }

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

        // Push the generated asset into the original array so we can extract subAssets from it too
        incomingSpecs.push(newSpec);
        newSpecsByHash[hash] = {
          spec: newSpec,
          names: [name],
        };
      }
    }

    outgoingSpecs.push(spec);
  }

  // Select best name and category for each subAsset
  for (
    const { spec, names } of _.values(newSpecsByHash) as {
      spec: PkgSpec;
      names: string[];
    }[]
  ) {
    let finalObjName: string | null | undefined = undefined;
    let finalAwsCategory: string | null | undefined = undefined;
    let finalParent: string | null | undefined = undefined;

    for (const name of names) {
      const nameTokens = name.split("::");
      if (nameTokens.length !== 4) {
        throw new Error(`Could not parse subAsset name: ${name}`);
      }

      const [_aws, awsCategory, parent, objName] = nameTokens;
      finalObjName = objName;

      // For categories and parents, set to null if not all of them are the same
      if (finalAwsCategory === undefined) {
        finalAwsCategory = awsCategory;
      } else if (
        finalAwsCategory !== null && finalAwsCategory !== awsCategory
      ) {
        finalAwsCategory = null;
        // Category being null also short circuits the parent to null
        finalParent = null;
        break;
      }

      if (finalParent === undefined) {
        finalParent = parent;
      } else if (finalParent !== null && parent !== finalParent) {
        finalParent = null;
      }
    }

    let finalName: string;
    let finalSiCategory: string | undefined;

    if (finalParent) {
      finalName = `${finalParent} ${finalObjName}`;
    } else if (finalAwsCategory) {
      finalName = `${finalAwsCategory} ${finalObjName}`;
    } else {
      finalName = `AWS ${finalObjName}`;
      finalSiCategory = "AWS Structural Assets";
    }

    const schema = spec.schemas[0];
    if (!schema || !schema.data) {
      throw new Error(`Could not parse schema for subAsset: ${name}`);
    }
    const schemaVariant = schema.variants[0];

    if (!schemaVariant || !schemaVariant.data) {
      throw new Error(`Could not get variant for subAsset: ${name}`);
    }

    spec.name = finalName;
    schema.name = finalName;
    schema.data.name = finalName;
    if (finalSiCategory) {
      schema.data.category = finalSiCategory;
    }
    schemaVariant.data.displayName = finalName;
  }

  return outgoingSpecs;
}

function fixPropPath(props: ExpandedPropSpec[], parentPath: string[]) {
  for (const prop of props) {
    prop.metadata.propPath = [...parentPath, prop.name];
    if (prop.kind === "object") {
      fixPropPath(prop.entries as ExpandedPropSpec[], prop.metadata.propPath);
    }
  }
}
