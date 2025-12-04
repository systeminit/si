import _ from "lodash";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createObjectProp,
  createScalarProp,
  ExpandedPropSpec,
} from "../../../spec/props.ts";
import { DigitalOceanSchema } from "../schema.ts";

export interface PropUsageMap {
  createOnly: string[];
  updatable: string[];
}

export function addDefaultProps(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const { domain } = schemaVariant;

    // Extra prop
    const extraProp = createObjectProp(
      "extra",
      domain.metadata.propPath,
      undefined,
      true,
    );

    extraProp.data.hidden = true;

    // Create Endpoint prop
    {
      const endpointProp = createScalarProp(
        "endpoint",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      endpointProp.data.hidden = true;


      // Get endpoint from the DigitalOceanSchema stored in the variant's superSchema
      const doSchema = schemaVariant.superSchema as DigitalOceanSchema;
      endpointProp.data.defaultValue = doSchema?.endpoint || schema.name;

      extraProp.entries.push(endpointProp);
    }

    // Create IdentifierField prop
    {
      const identifierFieldProp = createScalarProp(
        "IdentifierField",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      const doSchema = schemaVariant.superSchema as DigitalOceanSchema;
      identifierFieldProp.data.defaultValue = doSchema?.identifierField || "id";
      identifierFieldProp.data.hidden = true;

      extraProp.entries.push(identifierFieldProp);
    }

    // Create UpdateMethod prop
    {
      const updateMethodProp = createScalarProp(
        "UpdateMethod",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      const doSchema = schemaVariant.superSchema as DigitalOceanSchema;
      updateMethodProp.data.defaultValue = doSchema?.updateMethod || "PUT";
      updateMethodProp.data.hidden = true;

      extraProp.entries.push(updateMethodProp);
    }

    // Create RequiredQueryParams prop
    {
      const queryParamsProp = createScalarProp(
        "RequiredQueryParams",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      const doSchema = schemaVariant.superSchema as DigitalOceanSchema;
      queryParamsProp.data.defaultValue = JSON.stringify(doSchema?.requiredQueryParams || []);
      queryParamsProp.data.hidden = true;

      extraProp.entries.push(queryParamsProp);
    }

    // Create DigitalOceanResourceType prop
    // TODO: Check if this is really needed
    {
      const resourceTypeProp = createScalarProp(
        "DigitalOceanResourceType",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      resourceTypeProp.data.defaultValue = schema.name;
      resourceTypeProp.data.hidden = true;

      extraProp.entries.push(resourceTypeProp);
    }

    // Create PropUsageMap
    {
      const propUsageMapProp = createScalarProp(
        "PropUsageMap",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      const propUsageMap: PropUsageMap = {
        createOnly: [],
        updatable: [],
      };

      const queue: ExpandedPropSpec[] = _.cloneDeep(domain.entries);

      while (queue.length > 0) {
        const prop = queue.pop();
        if (!prop) break;

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

    // Create ScalarPropertyMap - tracks root-level domain properties that are scalars
    // Used by discover/import functions to know which properties to normalize from objects to scalars
    // TODO: Check if this is really needed
    {
      const scalarPropertyMapProp = createScalarProp(
        "ScalarPropertyMap",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      const scalarProperties: string[] = [];

      // Only check root-level domain properties
      for (const prop of domain.entries) {
        if (
          prop.kind === "string" || prop.kind === "number"
        ) {
          scalarProperties.push(prop.name);
        }
      }

      scalarPropertyMapProp.data.defaultValue = JSON.stringify(
        scalarProperties,
      );
      scalarPropertyMapProp.data.hidden = true;
      extraProp.entries.push(scalarPropertyMapProp);
    }

    {
      const credProp = createScalarProp(
        "DigitalOcean Credential",
        "string",
        extraProp.metadata.propPath,
        true,
      );
      credProp.data.widgetKind = "Secret";
      credProp.data.widgetOptions = [
        {
          label: "secretKind",
          value: "DigitalOcean Credential",
        },
      ];

      if (schemaVariant.secrets.kind !== "object") {
        console.log(
          `Could not generate default props for ${spec.name}: secrets is not object`,
        );
        continue;
      }

      addPropSuggestSource(credProp, {
        schema: "DigitalOcean Credential",
        prop: "/secrets/DigitalOcean Credential",
      });

      schemaVariant.secrets.entries.push(credProp);
    }

    // Finalize
    domain.entries.push(extraProp);
    newSpecs.push(spec);
  }

  return newSpecs;
}
