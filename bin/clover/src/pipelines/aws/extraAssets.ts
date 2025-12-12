import { SuperSchema } from "../types.ts";

/**
 * Manually-managed assets for AWS that are custom-defined and not auto-generated.
 *
 * Each asset is defined in its own directory under manually-managed-assets/{ResourceType}/
 * with a schema.ts file that exports both the schema and complete configuration.
 *
 * This keeps everything about a custom resource in one place.
 */

// Import resource definitions
import ec2Ami from "./manually-managed-assets/AWS::EC2::AMI/schema.ts";

/**
 * Registry of all manually-managed assets with their schemas and configuration
 */
export const AWS_MANUALLY_MANAGED_ASSETS = {
  "AWS::EC2::AMI": ec2Ami,
};

/**
 * Load all manually-managed asset schemas
 * Used by the extraAssets.loadSchemas config
 */
export function loadAwsExtraAssets(): SuperSchema[] {
  return Object.values(AWS_MANUALLY_MANAGED_ASSETS).map((asset) => asset.schema);
}

/**
 * Get custom configuration (functions, bindings, metadata, etc.) for all manually-managed assets
 * Used by the extraAssets.customFuncs config
 */
export function getAwsExtraAssetFuncs() {
  const customFuncs: Record<string, any> = {};
  for (const [typeName, asset] of Object.entries(AWS_MANUALLY_MANAGED_ASSETS)) {
    customFuncs[typeName] = asset.config;
  }
  return customFuncs;
}
