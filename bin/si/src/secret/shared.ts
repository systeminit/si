/**
 * Shared utilities for secret create and update commands
 */

import type { ChangeSetsApi, SecretsApi, SecretV1 } from "@systeminit/api-client";
import type { Context } from "../context.ts";
import type { SecretDefinitionV1, SecretFieldValues } from "./types.ts";
import type { GlobalOptions } from "../cli.ts";

/**
 * JWT payload structure for System Initiative API tokens.
 */
export interface SIJwtPayload {
  workspaceId: string;
  userId: string;
  [key: string]: unknown;
}

/**
 * Options that are common between create and update commands
 */
export interface SecretCommandOptions extends GlobalOptions {
  changeSet?: string;
  useLocalProfile?: boolean;
  interactive?: boolean;
  dryRun?: boolean;
  fields?: Record<string, string>;
}

/**
 * Result of getting or creating a change set
 */
export interface ChangeSetResult {
  changeSetId: string;
  wasCreated: boolean;
}

/**
 * Get or create a change set for the secret operation
 *
 * @returns Object containing the changeSetId and whether it was newly created
 */
export async function getOrCreateChangeSet(
  ctx: Context,
  changeSetsApi: ChangeSetsApi,
  workspaceId: string,
  changeSetId: string | undefined,
  operationName: string,
): Promise<ChangeSetResult> {
  // If change set specified, use it
  if (changeSetId) {
    // TODO: Validate that the change set exists
    // For now, just return it
    return {
      changeSetId,
      wasCreated: false,
    };
  }

  // Create a new change set
  const changeSetName = `${operationName} - ${Date.now()}`;

  ctx.logger.info(`Creating change set: ${changeSetName}`);

  const response = await changeSetsApi.createChangeSet({
    workspaceId,
    createChangeSetV1Request: {
      changeSetName,
    },
  });

  const newChangeSetId = response.data.changeSet.id;

  ctx.logger.debug(`Created change set: ${newChangeSetId}`);

  return {
    changeSetId: newChangeSetId,
    wasCreated: true,
  };
}

/**
 * Find a secret by name or ID
 */
export async function findSecret(
  ctx: Context,
  secretsApi: SecretsApi,
  workspaceId: string,
  changeSetId: string,
  secretIdOrName: string,
): Promise<{ id: string; secret: SecretV1; definition: SecretDefinitionV1 }> {
  // Get all secrets
  const response = await secretsApi.getSecrets({
    workspaceId,
    changeSetId,
  });

  const matches: Array<{ id: string; secret: SecretV1; definition: SecretDefinitionV1 }> = [];

  // Search through the grouped secrets
  for (const [_key, value] of Object.entries(response.data)) {
    if (value.secrets) {
      for (const secret of value.secrets) {
        // Match by ID (exact match only)
        if (secret.id === secretIdOrName) {
          if (!value.definition) {
            throw new Error(`Secret found but definition missing: ${secret.id}`);
          }
          // ID match is unique, return immediately
          return {
            id: secret.id,
            secret,
            definition: value.definition,
          };
        }

        // Match by name
        if (secret.name === secretIdOrName) {
          if (!value.definition) {
            throw new Error(`Secret found but definition missing: ${secret.id}`);
          }
          matches.push({
            id: secret.id,
            secret,
            definition: value.definition,
          });
        }
      }
    }
  }

  // Check results
  if (matches.length === 0) {
    throw new Error(
      `Secret not found: "${secretIdOrName}". Use either the secret ID or exact name.`,
    );
  }

  if (matches.length > 1) {
    ctx.logger.error(`Multiple secrets found with name "${secretIdOrName}":`);
    for (const match of matches) {
      ctx.logger.error(`  - ID: ${match.id}, Definition: ${match.secret.definition}`);
    }
    ctx.logger.error("");
    ctx.logger.error(`Please specify the secret by ID instead:`);
    ctx.logger.error(`  si secret update <SECRET_ID>`);
    ctx.logger.error("");
    ctx.logger.error(`For example:`);
    ctx.logger.error(`  si secret update ${matches[0].id}`);
    throw new Error(
      `Found ${matches.length} secrets with name "${secretIdOrName}". Use the exact secret ID to update a specific secret.`,
    );
  }

  return matches[0];
}

/**
 * Collect field values from various sources
 *
 * @param allowEmpty - If true, returns undefined when no fields are provided.
 *                     If false, throws an error when no fields are provided.
 */
export async function collectFieldValues(
  ctx: Context,
  definition: SecretDefinitionV1,
  options: SecretCommandOptions,
  allowEmpty: boolean = false,
): Promise<SecretFieldValues | undefined> {
  // Start with any field values provided via CLI arguments
  let fieldValues: SecretFieldValues = options.fields || {};

  // If --use-local-profile, try to discover credentials
  if (options.useLocalProfile) {
    ctx.logger.info("Discovering credentials from local environment...");

    const { discoverCredentials } = await import("./discovery.ts");
    const discovered = discoverCredentials(definition.formData);

    if (discovered) {
      ctx.logger.info(
        `✓ Found ${
          Object.keys(discovered).length
        } credential(s) in local environment`,
      );

      // Merge discovered values with any CLI-provided values (CLI takes precedence)
      fieldValues = { ...discovered, ...fieldValues };
    } else {
      ctx.logger.warn("Could not discover credentials from local environment");
      ctx.logger.info(
        "You can set environment variables or use --interactive mode",
      );
    }
  }

  // Check if we need to prompt for any values
  // Only prompt if interactive mode is explicitly requested
  if (options.interactive) {
    ctx.logger.info("");
    const { promptForFields } = await import("./prompts.ts");
    const promptedValues = await promptForFields(
      definition.formData,
      fieldValues,
    );

    // Merge prompted values (only non-empty ones)
    for (const [key, value] of Object.entries(promptedValues)) {
      if (value && value.trim().length > 0) {
        fieldValues[key] = value;
      }
    }
  }

  // If no field values provided at all
  if (Object.keys(fieldValues).length === 0) {
    if (allowEmpty) {
      return undefined;
    }

    // For create operations, we need at least some values
    ctx.logger.error("No credential values provided.");
    ctx.logger.info("");
    ctx.logger.info("Available options:");
    ctx.logger.info(
      "  --use-local-profile : Discover credentials from environment",
    );
    ctx.logger.info("  --interactive       : Prompt for values");
    ctx.logger.info("  --field-<name>      : Provide specific field values");
    ctx.logger.info("");
    ctx.logger.info("Available fields for this secret type:");
    for (const field of definition.formData) {
      ctx.logger.info(`  - ${field.name}`);
    }
    throw new Error("No credential values provided");
  }

  return fieldValues;
}

/**
 * Display field values in a dry-run or preview context
 */
export function displayFieldValues(
  ctx: Context,
  definition: SecretDefinitionV1,
  fieldValues: SecretFieldValues | undefined,
  prefix: string = "  ",
): void {
  if (!fieldValues || Object.keys(fieldValues).length === 0) {
    ctx.logger.info(`${prefix}Fields: (no changes)`);
    return;
  }

  ctx.logger.info(`${prefix}Fields:`);

  for (const field of definition.formData) {
    const value = fieldValues[field.name];

    if (value) {
      // Mask sensitive values
      const displayValue = field.kind === "password"
        ? "********"
        : value.length > 20
        ? `${value.substring(0, 20)}... (${value.length} characters)`
        : value;

      ctx.logger.info(`${prefix}  - ${field.name}: ${displayValue}`);
    } else {
      ctx.logger.info(`${prefix}  - ${field.name}: (not set)`);
    }
  }
}

/**
 * Options for displaying a dry run message
 */
export interface DryRunDisplayOptions {
  operation: "create" | "update";
  secretName: string;
  definition?: string;
  description?: string;
  existingName?: string;
  existingDescription?: string | null;
  componentId?: string;
}

/**
 * Display a dry run message for secret operations
 */
export function displaySecretDryRun(
  ctx: Context,
  options: DryRunDisplayOptions,
  definition: SecretDefinitionV1,
  fieldValues: SecretFieldValues | undefined,
): void {
  ctx.logger.info("");

  if (options.operation === "create") {
    ctx.logger.info("[DRY RUN] Would create secret with:");
    ctx.logger.info(`  Name: ${options.secretName}`);
    if (options.definition) {
      ctx.logger.info(`  Definition: ${options.definition}`);
    }
    if (options.description) {
      ctx.logger.info(`  Description: ${options.description}`);
    }
    if (options.componentId) {
      ctx.logger.info(`  Component ID: ${options.componentId}`);
    }
  } else {
    ctx.logger.info("[DRY RUN] Would update secret:");
    ctx.logger.info(`  Current Name: ${options.existingName}`);

    if (options.secretName !== options.existingName) {
      ctx.logger.info(`  New Name: ${options.secretName}`);
    }

    const existingDesc = options.existingDescription ?? undefined;
    if (options.description !== undefined && options.description !== existingDesc) {
      ctx.logger.info(
        `  Description: ${
          options.existingDescription || "(none)"
        } → ${options.description}`,
      );
    }
  }

  displayFieldValues(ctx, definition, fieldValues, "  ");

  ctx.logger.info("");
  ctx.logger.info(
    `No changes made. Remove --dry-run to ${options.operation} the secret.`,
  );
}

/**
 * Options for displaying a success message
 */
export interface SuccessDisplayOptions {
  operation: "created" | "updated";
  secret: SecretV1;
  changeSetId: string;
  componentId?: string;
  nextSteps?: string[];
}

/**
 * Display a success message for secret operations
 */
export function displaySecretSuccess(
  ctx: Context,
  options: SuccessDisplayOptions,
): void {
  ctx.logger.info("");

  const operationText = options.operation === "created"
    ? "Credential created successfully!"
    : "Secret updated successfully!";

  ctx.logger.info(`✓ ${operationText}`);

  if (options.componentId) {
    ctx.logger.info(`  Component ID: ${options.componentId}`);
  }

  ctx.logger.info(`  Secret ID: ${options.secret.id}`);
  ctx.logger.info(`  Secret Name: ${options.secret.name}`);
  ctx.logger.info(`  Change Set ID: ${options.changeSetId}`);

  if (options.secret.description) {
    ctx.logger.info(`  Description: ${options.secret.description}`);
  }

  if (options.nextSteps && options.nextSteps.length > 0) {
    ctx.logger.info("");
    ctx.logger.info("Next steps:");
    options.nextSteps.forEach((step, index) => {
      ctx.logger.info(`  ${index + 1}. ${step}`);
    });
  }
}
