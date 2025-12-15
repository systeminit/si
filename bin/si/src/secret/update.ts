import {
  ChangeSetsApi,
  SecretsApi,
} from "@systeminit/api-client";
import { Context } from "../context.ts";
import {
  type SecretCommandOptions,
  getOrCreateChangeSet,
  collectFieldValues,
  displaySecretDryRun,
  displaySecretSuccess,
  findSecret,
} from "./shared.ts";

export interface SecretUpdateOptions extends SecretCommandOptions {
  secretId?: string;
  secretName?: string;
  name?: string;
  description?: string;
}

/**
 * Main entry point for the secret update command
 */
export async function callSecretUpdate(
  options: SecretUpdateOptions,
): Promise<void> {
  // Get context
  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

  // Validate that either secretId or secretName is provided
  if (!options.secretId && !options.secretName) {
    throw new Error(
      "Either --secret-id or --secret-name is required to identify the secret to update.",
    );
  }

  const secretIdOrName = options.secretId || options.secretName!;

  // Create API clients
  const changeSetsApi = new ChangeSetsApi(apiConfig);
  const secretsApi = new SecretsApi(apiConfig);

  // Get or create change set
  const { changeSetId, wasCreated } = await getOrCreateChangeSet(
    ctx,
    changeSetsApi,
    workspaceId,
    options.changeSet,
    "Update secret",
  );

  try {
    // Find the secret
    ctx.logger.info(`Looking for secret: ${secretIdOrName}`);
    const { id: secretId, secret: existingSecret, definition } = await findSecret(
      ctx,
      secretsApi,
      workspaceId,
      changeSetId,
      secretIdOrName,
    );

    ctx.logger.info(`Found secret: ${existingSecret.name} (${secretId})`);

    // Determine new name (use existing if not provided)
    const newName = options.name || existingSecret.name;

    // Determine new description (use existing if not explicitly set)
    const newDescription = options.description !== undefined
      ? options.description
      : (existingSecret.description ?? undefined);

    // Collect field values if any are being updated
    // For updates, allow empty field values (updates only metadata)
    const fieldValues = await collectFieldValues(ctx, definition, options, true);

    // Dry run mode - show what would be updated
    if (options.dryRun) {
      displaySecretDryRun(
        ctx,
        {
          operation: "update",
          secretName: newName,
          description: newDescription,
          existingName: existingSecret.name,
          existingDescription: existingSecret.description,
        },
        definition,
        fieldValues,
      );
      return;
    }

    // Perform the update
    ctx.logger.info("");
    ctx.logger.info(`Updating secret "${existingSecret.name}"...`);

    const updateResponse = await secretsApi.updateSecret({
      workspaceId,
      changeSetId,
      secretId,
      updateSecretV1Request: {
        id: secretId,
        name: newName,
        description: newDescription || null,
        rawData: fieldValues,
      },
    });

    const updatedSecret = updateResponse.data.secret;
    ctx.logger.info(`✓ Secret updated: ${updatedSecret.id}`);

    // Display success
    displaySecretSuccess(ctx, {
      operation: "updated",
      secret: updatedSecret,
      changeSetId,
      nextSteps: [
        "Apply the change set to make the changes available",
        "Components using this secret will use the updated values",
      ],
    });
    
    ctx.analytics.trackEvent("secret update", {
      secretType: definition.secretDefinition,
      secretName:existingSecret.name,
      useLocalProfile: options.useLocalProfile,
      interactive: options.interactive,
      dryRun: options.dryRun ?? false,
    });
  } catch (error) {
    ctx.logger.error(`Failed to update secret: ${error}`);

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
