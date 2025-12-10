import {
  ChangeSetsApi,
  ComponentsApi,
  SchemasApi,
  SecretsApi,
} from "@systeminit/api-client";
import { Context } from "../context.ts";
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
  promptForSecretName,
} from "./prompts.ts";
import {
  getOrCreateChangeSet,
  collectFieldValues,
  displaySecretDryRun,
  displaySecretSuccess,
} from "./shared.ts";

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
 * Main entry point for the secret create command
 */
export async function callSecretCreate(
  options: SecretCreateOptions,
): Promise<void> {
  // Get context
  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

  // Create API clients
  const changeSetsApi = new ChangeSetsApi(apiConfig);
  const secretsApi = new SecretsApi(apiConfig);
  const schemasApi = new SchemasApi(apiConfig);
  const componentsApi = new ComponentsApi(apiConfig);

  // Get or create change set
  const { changeSetId, wasCreated } = await getOrCreateChangeSet(
    ctx,
    changeSetsApi,
    workspaceId,
    options.changeSet,
    `Create ${options.secretType}`,
  );

  try {
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
    // For creates, don't allow empty field values
    const fieldValues = await collectFieldValues(ctx, definition, options, false);

    // Dry run mode - show what would be created
    if (options.dryRun) {
      displaySecretDryRun(
        ctx,
        {
          operation: "create",
          secretName,
          definition: definition.secretDefinition,
          description,
        },
        definition,
        fieldValues,
      );
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
    displaySecretSuccess(ctx, {
      operation: "created",
      secret,
      changeSetId,
      componentId,
      nextSteps: [
        "Apply the change set to make the credential available",
        "Other components can now use this credential",
      ],
    });
  } catch (error) {
    ctx.logger.error(`Failed to create secret: ${error}`);

    // Abandon the change set if we created it
    if (wasCreated) {
      ctx.logger.info("Abandoning change set due to error...");
      try {
        await changeSetsApi.abandonChangeSet({
          workspaceId,
          changeSetId,
        });
        ctx.logger.debug(`Abandoned change set: ${changeSetId}`);
      } catch (abandonError) {
        ctx.logger.warn(`Failed to abandon change set: ${abandonError}`);
      }
    }

    Deno.exit(1);
  }
}
