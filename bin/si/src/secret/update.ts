import {
  ChangeSetsApi,
  ComponentsApi,
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
  componentNameOrId: string;
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

  const componentNameOrId = options.componentNameOrId;

  // Create API clients
  const changeSetsApi = new ChangeSetsApi(apiConfig);
  const secretsApi = new SecretsApi(apiConfig);
  const componentsApi = new ComponentsApi(apiConfig);

  // Get or create change set
  const { changeSetId, wasCreated } = await getOrCreateChangeSet(
    ctx,
    changeSetsApi,
    workspaceId,
    options.changeSet,
    "Update secret",
  );

  try {
    // Find the component by name or ID
    ctx.logger.info(`Looking for component: ${componentNameOrId}`);

    // Detect if it's a ULID (26 alphanumeric characters)
    const isUlid = /^[0-9A-HJKMNP-TV-Z]{26}$/i.test(componentNameOrId);

    // First, find the component to get its ID
    const findResponse = await componentsApi.findComponent({
      workspaceId,
      changeSetId,
      // Use componentId parameter for ULIDs, component parameter for names
      ...(isUlid
        ? { componentId: componentNameOrId }
        : { component: componentNameOrId }),
    });

    const componentId = findResponse.data.component.id;
    ctx.logger.info(`Found component: ${findResponse.data.component.name} (${componentId})`);

    // Get the full component details which includes secretId
    const componentResponse = await componentsApi.getComponent({
      workspaceId,
      changeSetId,
      componentId,
    });

    const component = componentResponse.data.component;

    // Check if the component has a secretId
    // The secretId is available in the getComponent response
    const secretId = (component as { secretId?: string }).secretId;
    if (!secretId) {
      throw new Error(
        `Component "${componentNameOrId}" does not have an associated secret. Only components with secrets can be updated.`,
      );
    }

    ctx.logger.info(`Found secretId in component: ${secretId}`);

    // Find the secret using the secretId
    const { secret: existingSecret, definition } = await findSecret(
      ctx,
      secretsApi,
      workspaceId,
      changeSetId,
      secretId,
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
