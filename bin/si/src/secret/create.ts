import {
  ChangeSetsApi,
  ComponentsApi,
  SchemasApi,
  SecretsApi,
  type SecretV1,
} from "@systeminit/api-client";
import { Context } from "../context.ts";
import type { SecretDefinitionV1, SecretFieldValues } from "./types.ts";
import type { SecretCreateOptions } from "./types.ts";

export type { SecretCreateOptions };
import {
  getSecretDefinitions,
  listAvailableSecretTypes,
  matchSecretType,
  suggestSimilarSecretTypes,
} from "./definitions.ts";
import {
  promptForDescription,
  promptForFields,
  promptForSecretName,
} from "./prompts.ts";
import { discoverCredentials } from "./discovery.ts";

/**
 * JWT payload structure for System Initiative API tokens.
 */
interface SIJwtPayload {
  workspaceId: string;
  userId: string;
  [key: string]: unknown;
}

/**
 * Attempt to install a missing secret schema
 */
async function tryInstallSecretSchema(
  ctx: Context,
  schemasApi: SchemasApi,
  changeSetsApi: ChangeSetsApi,
  workspaceId: string,
  secretType: string,
): Promise<boolean> {
  try {
    ctx.logger.info(
      `Secret type "${secretType}" not found. Attempting to install...`,
    );

    // Create a temporary changeset for schema installation
    const installChangeSetName = `Install ${secretType} schema - ${Date.now()}`;
    const { data: createResponse } = await changeSetsApi.createChangeSet({
      workspaceId,
      createChangeSetV1Request: {
        changeSetName: installChangeSetName,
      },
    });

    const installChangeSetId = createResponse.changeSet.id;

    ctx.logger.debug(`Created install changeset: ${installChangeSetId}`);

    // Try to find the schema
    try {
      const { data: schema } = await schemasApi.findSchema({
        workspaceId,
        changeSetId: installChangeSetId,
        schema: secretType,
      });

      if (schema && schema.schemaId) {
        ctx.logger.info(`Found schema "${secretType}", installing...`);

        // Install the schema - this makes it available immediately
        // No need to apply the changeset since schema installation is idempotent
        // and the schema becomes available for use right away
        await schemasApi.installSchema({
          workspaceId,
          changeSetId: installChangeSetId,
          schemaId: schema.schemaId,
        });

        ctx.logger.info(`✓ Successfully installed "${secretType}" schema`);
        return true;
      } else {
        ctx.logger.warn(
          `Schema "${secretType}" not found in available schemas`,
        );
        return false;
      }
    } catch (error) {
      ctx.logger.debug(`Failed to find or install schema: ${error}`);
      return false;
    }
  } catch (error) {
    ctx.logger.debug(`Failed to install secret schema: ${error}`);
    return false;
  }
}

/**
 * Get or create a change set for the secret operation
 */
async function getOrCreateChangeSet(
  ctx: Context,
  changeSetsApi: ChangeSetsApi,
  workspaceId: string,
  options: SecretCreateOptions,
): Promise<string> {
  // If change set specified, use it
  if (options.changeSet) {
    // TODO: Validate that the change set exists
    // For now, just return it
    return options.changeSet;
  }

  // Create a new change set
  const changeSetName = `Create ${options.secretType} - ${Date.now()}`;

  ctx.logger.info(`Creating change set: ${changeSetName}`);

  const response = await changeSetsApi.createChangeSet({
    workspaceId,
    createChangeSetV1Request: {
      changeSetName,
    },
  });

  const changeSetId = response.data.changeSet.id;

  ctx.logger.debug(`Created change set: ${changeSetId}`);

  return changeSetId;
}

/**
 * Collect field values from various sources
 */
async function collectFieldValues(
  ctx: Context,
  definition: SecretDefinitionV1,
  options: SecretCreateOptions,
): Promise<SecretFieldValues> {
  // Start with any field values provided via CLI arguments
  let fieldValues: SecretFieldValues = options.fields || {};

  // If --use-local-profile, try to discover credentials
  if (options.useLocalProfile) {
    ctx.logger.info("Discovering credentials from local environment...");

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

  return fieldValues;
}

/**
 * Display the secret that would be created (dry run mode)
 */
function displayDryRun(
  ctx: Context,
  secretName: string,
  description: string,
  definition: SecretDefinitionV1,
  fieldValues: SecretFieldValues,
): void {
  ctx.logger.info("");
  ctx.logger.info("[DRY RUN] Would create secret with:");
  ctx.logger.info(`  Name: ${secretName}`);
  ctx.logger.info(`  Definition: ${definition.secretDefinition}`);

  if (description) {
    ctx.logger.info(`  Description: ${description}`);
  }

  ctx.logger.info("  Fields:");

  for (const field of definition.formData) {
    const value = fieldValues[field.name];

    if (value) {
      // Mask sensitive values
      const displayValue =
        field.kind === "password"
          ? "********"
          : value.length > 20
            ? `${value.substring(0, 20)}... (${value.length} characters)`
            : value;

      ctx.logger.info(`    - ${field.name}: ${displayValue}`);
    } else {
      ctx.logger.info(`    - ${field.name}: (not set)`);
    }
  }

  ctx.logger.info("");
  ctx.logger.info("No changes made. Remove --dry-run to create the secret.");
}

/**
 * Display success message after secret creation
 */
function displaySuccess(
  ctx: Context,
  secret: SecretV1,
  changeSetId: string,
  componentId: string,
): void {
  ctx.logger.info("");
  ctx.logger.info("✓ Credential created successfully!");
  ctx.logger.info(`  Component ID: ${componentId}`);
  ctx.logger.info(`  Secret ID: ${secret.id}`);
  ctx.logger.info(`  Secret Name: ${secret.name}`);
  ctx.logger.info(`  Change Set ID: ${changeSetId}`);

  if (secret.description) {
    ctx.logger.info(`  Description: ${secret.description}`);
  }

  ctx.logger.info("");
  ctx.logger.info("Next steps:");
  ctx.logger.info(`  1. Apply the change set to make the credential available`);
  ctx.logger.info(`  2. Other components can now use this credential`);
}

/**
 * Main entry point for the secret create command
 */
export async function callSecretCreate(
  options: SecretCreateOptions,
): Promise<void> {
  // Get context
  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

  try {
    // Create API clients
    const changeSetsApi = new ChangeSetsApi(apiConfig);
    const secretsApi = new SecretsApi(apiConfig);
    const schemasApi = new SchemasApi(apiConfig);
    const componentsApi = new ComponentsApi(apiConfig);

    // Get or create change set
    const changeSetId = await getOrCreateChangeSet(
      ctx,
      changeSetsApi,
      workspaceId,
      options,
    );

    // Query available secret definitions
    ctx.logger.debug("Querying available secret definitions...");

    let definitions = await getSecretDefinitions(
      secretsApi,
      workspaceId,
      changeSetId,
    );

    if (definitions.length === 0) {
      throw new Error("No secret definitions available in this workspace");
    }

    // Match the secret type
    let definition = matchSecretType(options.secretType, definitions, ctx);

    if (!definition) {
      // Try to install the schema if it's not found
      const installed = await tryInstallSecretSchema(
        ctx,
        schemasApi,
        changeSetsApi,
        workspaceId,
        options.secretType,
      );

      if (installed) {
        // Re-query definitions after installation
        ctx.logger.info("Re-querying secret definitions...");
        definitions = await getSecretDefinitions(
          secretsApi,
          workspaceId,
          changeSetId,
        );

        // Try matching again
        definition = matchSecretType(options.secretType, definitions, ctx);
      }

      if (!definition) {
        // Still no match - provide helpful error
        ctx.logger.error(`Unknown secret type: "${options.secretType}"`);
        ctx.logger.info("");

        const suggestions = suggestSimilarSecretTypes(
          options.secretType,
          definitions,
        );

        if (suggestions.length > 0) {
          ctx.logger.info("Did you mean one of these?");
          for (const suggestion of suggestions) {
            ctx.logger.info(`  - ${suggestion}`);
          }
          ctx.logger.info("");
        }

        ctx.logger.info(listAvailableSecretTypes(definitions));
        Deno.exit(1);
      }
    }

    // Get secret name
    let secretName = options.name;

    if (!secretName && !options.interactive) {
      throw new Error(
        "Secret name is required. Use --name flag or --interactive mode.",
      );
    }

    if (!secretName) {
      secretName = await promptForSecretName();
    }

    // Get description
    let description = options.description || "";

    if (options.interactive && !options.description) {
      description = await promptForDescription();
    }

    // Collect field values
    const fieldValues = await collectFieldValues(ctx, definition, options);

    // Check if we have at least some values to create a secret
    // We don't validate all fields as required since some may be optional
    if (Object.keys(fieldValues).length === 0) {
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
      Deno.exit(1);
    }

    // Dry run mode - show what would be created
    if (options.dryRun) {
      displayDryRun(ctx, secretName, description, definition, fieldValues);
      return;
    }

    // Step 1: Create the credential component
    ctx.logger.info("");
    ctx.logger.info(
      `Creating ${definition.secretDefinition} component "${secretName}"...`,
    );

    const componentResponse = await componentsApi.createComponent({
      workspaceId,
      changeSetId,
      createComponentV1Request: {
        schemaName: definition.secretDefinition,
        name: secretName,
      },
    });

    const componentId = componentResponse.data.component.id;
    ctx.logger.info(`✓ Component created with ID: ${componentId}`);

    // Step 2: Create the secret
    ctx.logger.info(`Creating secret data...`);

    const secretResponse = await secretsApi.createSecret({
      workspaceId,
      changeSetId,
      createSecretV1Request: {
        name: secretName,
        definitionName: definition.secretDefinition,
        description: description || undefined,
        rawData: fieldValues,
      },
    });

    const secret = secretResponse.data.secret;
    ctx.logger.info(`✓ Secret created with ID: ${secret.id}`);

    // Step 3: Update the component to attach the secret
    ctx.logger.info(`Attaching secret to component...`);

    await componentsApi.updateComponent({
      workspaceId,
      changeSetId,
      componentId,
      updateComponentV1Request: {
        secrets: {
          [definition.secretDefinition]: secretName,
        },
      },
    });

    ctx.logger.info(`✓ Secret attached to component`);

    // Display success
    displaySuccess(ctx, secret, changeSetId, componentId);
  } catch (error) {
    ctx.logger.error(`Failed to create secret: ${error}`);
    Deno.exit(1);
  }
}
