import _ from "npm:lodash";
import {
  copyPropWithNewIds,
  createDefaultProp,
  ExpandedPropSpec,
  generatePropHash,
} from "../spec/props.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import {
  createInputSocketFromProp,
  createOutputSocketFromProp,
} from "../spec/sockets.ts";
import { attrFuncInputSpecFromProp } from "../spec/sockets.ts";
import _logger from "../logger.ts";
import { ExpandedPkgSpecWithSockets, ExpandedSchemaVariantSpecWithSockets } from "../spec/pkgs.ts";

const logger = _logger.ns("subAssets").seal();

export function generateSubAssets(
  incomingSpecs: ExpandedPkgSpecWithSockets[],
): ExpandedPkgSpecWithSockets[] {
  const outgoingSpecs = [] as ExpandedPkgSpecWithSockets[];
  const newSpecsByHash = {} as Record<
    string,
    { spec: ExpandedPkgSpecWithSockets; names: string[] }
  >;

  for (const spec of incomingSpecs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const { domain } = schemaVariant;

    for (const prop of domain.entries) {
      if (prop.kind === "array" && prop.typeProp.kind === "object") {
        const objName = prop.name;

        logger.debug(`Generating subasset ${objName} for ${spec.name}`);

        const name = `${spec.name}::${objName}`;
        const variantId = ulid();

        const newDomainWithOldIds: typeof domain = _.cloneDeep(domain);
        newDomainWithOldIds.entries = prop.typeProp.entries;

        // recreate ["root", "domain", etc.]
        fixPropPath(
          newDomainWithOldIds.entries,
          newDomainWithOldIds.metadata.propPath,
        );

        const newDomain = copyPropWithNewIds(newDomainWithOldIds);

        const hash = generatePropHash(newDomain);

        // set the parent prop to have an input socket for this new asset
        const propInputSocket = createInputSocketFromProp(prop);
        schemaVariant.sockets.push(propInputSocket);

        // reuse existing assets so we don't recreate the same asset over and
        // over again
        const maybeExistingSubAsset = newSpecsByHash[hash];
        if (maybeExistingSubAsset) {
          logger.debug(`Existing subasset found: ${name}`);
          maybeExistingSubAsset.names.push(name);
          continue;
        }

        // output the domain of this new spec
        const newSpecOutputSocket = createOutputSocketFromProp(prop, objName);
        newSpecOutputSocket.inputs = [attrFuncInputSpecFromProp(newDomain)];

        const variantData = _.cloneDeep(schemaVariant.data);
        const variant: ExpandedSchemaVariantSpecWithSockets = {
          ...schemaVariant,
          data: {
            ...variantData,
            displayName: name,
            funcUniqueId: ulid(),
            link: prop.typeProp.data?.docLink,
            description: prop.typeProp.data?.documentation ?? "",
          },
          domain: newDomain,
          actionFuncs: [],
          leafFunctions: [],
          managementFuncs: [],
          resourceValue: createDefaultProp("resource_value", undefined, false),
          sockets: [newSpecOutputSocket],
          secrets: createDefaultProp("secrets", undefined, false),
          uniqueId: variantId,
        };

        const schemaData = _.cloneDeep(schema.data);

        const newSpec: ExpandedPkgSpecWithSockets = {
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
  for (const { spec, names } of _.values(newSpecsByHash)) {
    let finalObjName: string | null | undefined = undefined;
    let finalAwsCategory: string | null | undefined = undefined;
    let finalParent: string | null | undefined = undefined;

    for (const name of names) {
      const nameTokens = name.split("::");
      // TODO check naming for sub sub assets
      if (nameTokens.length < 4) {
        throw new Error(`Could not parse subAsset name: ${name}`);
      }

      const awsCategory = nameTokens[1];
      const parent = nameTokens[nameTokens.length - 2];
      finalObjName = nameTokens[nameTokens.length - 1];

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
      finalSiCategory = "AWS";
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
      fixPropPath(prop.entries, prop.metadata.propPath);
    }
  }
}
