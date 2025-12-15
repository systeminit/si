import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createObjectProp,
  createScalarProp,
} from "../../../spec/props.ts";
import { GcpSchema } from "../schema.ts";

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
      insertProp.data.defaultValue = JSON.stringify({
        path: gcpSchema.methods.insert.path,
        parameterOrder: gcpSchema.methods.insert.parameterOrder,
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
      patchProp.data.defaultValue = JSON.stringify({
        path: gcpSchema.methods.patch.path,
        httpMethod: gcpSchema.methods.patch.httpMethod,
        parameterOrder: gcpSchema.methods.patch.parameterOrder,
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
      });
      extraProp.entries.push(deleteProp);
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
