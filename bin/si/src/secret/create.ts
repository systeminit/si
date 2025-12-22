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
import { promptForDescription, promptForSecretName } from "./prompts.ts";
import {
  collectFieldValues,
  displaySecretDryRun,
  displaySecretSuccess,
  getOrCreateChangeSet,
} from "./shared.ts";

/**
 * Attempt to install a missing secret schema
 */
async function tryInstallSecretSchema(
  ctx: Context,
  schemasApi: SchemasApi,
  workspaceId: string,
  changeSetId: string,
  secretType: string,
): Promise<boolean> {
  try {
    ctx.logger.info(
      `Secret type "${secretType}" not found. Attempting to install...`,
    );

    // Try to find the schema in the current changeset
    try {
      const { data: schema } = await schemasApi.findSchema({
        workspaceId,
        changeSetId,
        schema: secretType,
      });

      if (schema && schema.schemaId) {
        ctx.logger.info(`Found schema "${secretType}", installing...`);

        // Install the schema in the current changeset
        await schemasApi.installSchema({
          workspaceId,
          changeSetId,
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

    // Match the secret type
    let definition = matchSecretType(options.secretType, definitions, ctx);

    let installedSecretType = false;

    if (!definition) {
      // Try to install the schema if it's not found
      const installed = await tryInstallSecretSchema(
        ctx,
        schemasApi,
        workspaceId,
        changeSetId,
        options.secretType,
      );

      if (installed) {
        installedSecretType = true;

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

        if (definitions.length > 0) {
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
        } else {
          ctx.logger.info(
            "No secret definitions available. The schema could not be found or installed.",
          );
        }

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
    const fieldValues = await collectFieldValues(
      ctx,
      definition,
      options,
      false,
    );

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

    ctx.analytics.trackEvent("secret create", {
      secretType: definition.secretDefinition,
      secretName,
      useLocalProfile: options.useLocalProfile,
      interactive: options.interactive,
      installedSecretType,
      dryRun: options.dryRun ?? false,
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
