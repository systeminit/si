import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createObjectProp,
  createScalarProp,
  ExpandedPropSpec,
  toPropPath,
} from "../../../spec/props.ts";
import { GcpSchema } from "../schema.ts";
import _ from "lodash";

export interface PropUsageMap {
  createOnly: string[];
  updatable: string[];
  excluded: string[];
  pathParameters: string[];
}

export function addDefaultProps(specs: ExpandedPkgSpec[]): ExpandedPkgSpec[] {
  const gcpSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const { domain } = schemaVariant;

    // Extra prop (for future metadata if needed)
    const extraProp = createObjectProp(
      "extra",
      domain.metadata.propPath,
      undefined,
      true,
    );

    extraProp.data.hidden = true;

    // Get GCP schema to access method metadata
    const gcpSchema = schemaVariant.superSchema as GcpSchema;

    // Store method metadata for runtime use
    // Base URL
    {
      const baseUrlProp = createScalarProp(
        "baseUrl",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      baseUrlProp.data.hidden = true;
      baseUrlProp.data.defaultValue = gcpSchema.baseUrl;
      extraProp.entries.push(baseUrlProp);
    }

    // Store API endpoint metadata as JSON
    // Only store what's actually used: path, parameterOrder, and httpMethod (for update/patch only)
    if (gcpSchema.methods.get) {
      const getProp = createScalarProp(
        "getApiPath",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      getProp.data.hidden = true;
      getProp.data.defaultValue = JSON.stringify({
        path: gcpSchema.methods.get.path,
        parameterOrder: gcpSchema.methods.get.parameterOrder,
      });
      extraProp.entries.push(getProp);
    }

    if (gcpSchema.methods.insert) {
      const insertProp = createScalarProp(
        "insertApiPath",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      insertProp.data.hidden = true;

      // Collect query parameters for the insert method
      // These are parameters with location: "query" that need to be added to the URL
      const queryParams: string[] = [];
      if (gcpSchema.methods.insert.parameters) {
        for (
          const [paramName, paramDef] of Object.entries(
            gcpSchema.methods.insert.parameters,
          )
        ) {
          if (paramDef.location === "query") {
            queryParams.push(paramName);
          }
        }
      }

      insertProp.data.defaultValue = JSON.stringify({
        path: gcpSchema.methods.insert.path,
        httpMethod: gcpSchema.methods.insert.httpMethod,
        parameterOrder: gcpSchema.methods.insert.parameterOrder,
        ...(queryParams.length > 0 ? { queryParams } : {}),
      });
      extraProp.entries.push(insertProp);
    }

    if (gcpSchema.methods.update) {
      const updateProp = createScalarProp(
        "updateApiPath",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      updateProp.data.hidden = true;
      updateProp.data.defaultValue = JSON.stringify({
        path: gcpSchema.methods.update.path,
        httpMethod: gcpSchema.methods.update.httpMethod,
        parameterOrder: gcpSchema.methods.update.parameterOrder,
      });
      extraProp.entries.push(updateProp);
    }

    if (gcpSchema.methods.patch) {
      const patchProp = createScalarProp(
        "patchApiPath",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      patchProp.data.hidden = true;

      // Detect if the patch request uses a wrapper type (e.g., UpdateTopicRequest)
      // Wrapper types have a field that contains the resource (e.g., "topic") and an "updateMask" field
      // If detected, store the wrapper field name so the update action can wrap the payload
      let requestWrapperField: string | undefined;
      const patchRequest = gcpSchema.methods.patch.request;
      if (patchRequest?.properties) {
        const props = Object.keys(patchRequest.properties);
        // A wrapper type has exactly 2 properties: the resource wrapper field and updateMask
        if (props.length === 2 && props.includes("updateMask")) {
          requestWrapperField = props.find((p) => p !== "updateMask");
        }
      }

      patchProp.data.defaultValue = JSON.stringify({
        path: gcpSchema.methods.patch.path,
        httpMethod: gcpSchema.methods.patch.httpMethod,
        parameterOrder: gcpSchema.methods.patch.parameterOrder,
        ...(requestWrapperField ? { requestWrapperField } : {}),
      });
      extraProp.entries.push(patchProp);
    }

    if (gcpSchema.methods.delete) {
      const deleteProp = createScalarProp(
        "deleteApiPath",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      deleteProp.data.hidden = true;
      deleteProp.data.defaultValue = JSON.stringify({
        path: gcpSchema.methods.delete.path,
        parameterOrder: gcpSchema.methods.delete.parameterOrder,
        // Include httpMethod since some APIs use POST for delete (e.g., deleteConnection)
        httpMethod: gcpSchema.methods.delete.httpMethod,
      });
      extraProp.entries.push(deleteProp);
    }

    if (gcpSchema.methods.list) {
      // Extract query parameters from the list method
      const queryParams: string[] = [];
      if (gcpSchema.methods.list.parameters) {
        for (
          const [paramName, paramDef] of Object.entries(
            gcpSchema.methods.list.parameters,
          )
        ) {
          if (paramDef.location === "query") {
            queryParams.push(paramName);
          }
        }
      }

      const listProp = createScalarProp(
        "listApiPath",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      listProp.data.hidden = true;
      listProp.data.defaultValue = JSON.stringify({
        path: gcpSchema.methods.list.path,
        parameterOrder: gcpSchema.methods.list.parameterOrder,
        queryParams: queryParams, // Store valid query parameters
      });
      extraProp.entries.push(listProp);
    }

    // Store the GCP resource type for management functions
    {
      const resourceTypeProp = createScalarProp(
        "GcpResourceType",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      resourceTypeProp.data.hidden = true;
      resourceTypeProp.data.defaultValue = gcpSchema.typeName;
      extraProp.entries.push(resourceTypeProp);
    }

    // Store availableScopes for scope handling
    if (gcpSchema.availableScopes && gcpSchema.availableScopes.length > 0) {
      const scopesProp = createScalarProp(
        "availableScopes",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      scopesProp.data.hidden = true;
      scopesProp.data.defaultValue = JSON.stringify(gcpSchema.availableScopes);
      extraProp.entries.push(scopesProp);
    }

    // Flag for list-only resources (no get method)
    // Refresh action uses list filtering when this is true
    if (!gcpSchema.methods.get && gcpSchema.methods.list) {
      const listOnlyProp = createScalarProp(
        "listOnly",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      listOnlyProp.data.hidden = true;
      listOnlyProp.data.defaultValue = "true";
      extraProp.entries.push(listOnlyProp);
    }

    // For global-only resources, set the location prop to default "global"
    // This allows auto-construction of parent path without user input
    if (gcpSchema.isGlobalOnly) {
      const locationProp = domain.entries.find((p: ExpandedPropSpec) =>
        p.name === "location"
      );
      if (locationProp) {
        locationProp.data.hidden = true;
        locationProp.data.defaultValue = "global";
      }
    }

    // Create PropUsageMap for filtering createOnly properties in updates
    {
      const propUsageMapProp = createScalarProp(
        "PropUsageMap",
        "string",
        extraProp.metadata.propPath,
        false,
      );

      // Collect path parameters from all API methods that send request bodies
      const pathParamsSet = new Set<string>();
      if (gcpSchema.methods.insert?.parameterOrder) {
        gcpSchema.methods.insert.parameterOrder.forEach((p: string) =>
          pathParamsSet.add(p)
        );
      }
      if (gcpSchema.methods.update?.parameterOrder) {
        gcpSchema.methods.update.parameterOrder.forEach((p: string) =>
          pathParamsSet.add(p)
        );
      }
      if (gcpSchema.methods.patch?.parameterOrder) {
        gcpSchema.methods.patch.parameterOrder.forEach((p: string) =>
          pathParamsSet.add(p)
        );
      }

      // Add 'location' to pathParameters for resources that use it in URL construction
      // (e.g., projects/{project}/locations/{location}/keys)
      // This prevents 'location' from being sent in the request body
      if (
        gcpSchema.isGlobalOnly ||
        domain.entries.some((p: ExpandedPropSpec) => p.name === "location")
      ) {
        pathParamsSet.add("location");
      }

      // Add query parameters from insert method to pathParameters
      // These are URL params that should NOT be included in the request body
      // (e.g., serviceId for Cloud Run services)
      if (gcpSchema.methods.insert?.parameters) {
        for (
          const [paramName, param] of Object.entries(
            gcpSchema.methods.insert.parameters,
          )
        ) {
          if (param.location === "query") {
            pathParamsSet.add(paramName);
          }
        }
      }

      const propUsageMap: PropUsageMap = {
        createOnly: [],
        updatable: [],
        excluded: [],
        pathParameters: Array.from(pathParamsSet),
      };

      // Walk the domain properties and categorize them
      const queue: ExpandedPropSpec[] = _.cloneDeep(domain.entries);

      while (queue.length > 0) {
        const prop = queue.pop();
        if (!prop) break;

        const fullPath = toPropPath(prop.metadata.propPath);

        if (prop.metadata.createOnly) {
          propUsageMap.createOnly.push(fullPath);
        } else if (!prop.metadata.readOnly) {
          propUsageMap.updatable.push(fullPath);
        }

        if (prop.kind === "object") {
          prop.entries.forEach((p: ExpandedPropSpec) => queue.unshift(p));
        }
      }

      propUsageMapProp.data.defaultValue = JSON.stringify(propUsageMap);
      propUsageMapProp.data.hidden = true;
      extraProp.entries.push(propUsageMapProp);
    }

    // Add Google Cloud Credential to secrets
    {
      const credProp = createScalarProp(
        "Google Cloud Credential",
        "string",
        extraProp.metadata.propPath,
        true,
      );
      credProp.data.widgetKind = "Secret";
      credProp.data.widgetOptions = [
        {
          label: "secretKind",
          value: "Google Cloud Credential",
        },
      ];

      if (schemaVariant.secrets.kind !== "object") {
        console.log(
          `Could not generate default props for ${spec.name}: secrets is not object`,
        );
        continue;
      }

      addPropSuggestSource(credProp, {
        schema: "Google Cloud Credential",
        prop: "/secrets/Google Cloud Credential",
      });

      schemaVariant.secrets.entries.push(credProp);
    }

    domain.entries.push(extraProp);
    gcpSpecs.push(spec);
  }

  return gcpSpecs;
}
