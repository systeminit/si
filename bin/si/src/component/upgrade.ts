/**
 * Component Upgrade Module - Upgrade components to newer schema versions
 *
 * This module provides functionality to upgrade components to the latest version
 * of their schema. Components can only be upgraded in a change set (not HEAD),
 * and must be marked as upgradable. The command gracefully handles various error
 * conditions with clear messaging.
 *
 * @module
 */

import {
  ChangeSetsApi,
  ComponentsApi,
  SearchApi,
  SchemasApi,
} from "@systeminit/api-client";
import { Context } from "../context.ts";
import { resolveChangeSet } from "./change_set.ts";

/**
 * Result of getting or creating a change set
 */
interface ChangeSetResult {
  changeSetId: string;
  wasCreated: boolean;
}

/**
 * Options for the component upgrade command
 */
export interface ComponentUpgradeOptions {
  /** Change set ID or name (optional - will create new change set if not provided) */
  changeSet?: string;
  /** Optional schema category filter (e.g., "AWS::EC2") */
  schemaCategory?: string;
  /** Upgrade all upgradable components */
  all?: boolean;
  /** Dry-run mode - preview upgrades without applying */
  dryRun?: boolean;
}

/**
 * Get or create a change set for the upgrade operation
 *
 * @returns Object containing the changeSetId and whether it was newly created
 */
async function getOrCreateChangeSet(
  ctx: Context,
  changeSetsApi: ChangeSetsApi,
  workspaceId: string,
  changeSetIdOrName: string | undefined,
  operationName: string,
): Promise<ChangeSetResult> {
  // If change set specified, resolve name to ID if needed
  if (changeSetIdOrName) {
    ctx.logger.debug(`Resolving change set: ${changeSetIdOrName}`);
    const changeSetId = await resolveChangeSet(workspaceId, changeSetIdOrName);
    ctx.logger.debug(`Resolved to change set ID: ${changeSetId}`);
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
 * Check if a component can be upgraded
 * Returns true if upgradable, false if already at latest version
 * Exits on 404
 */
async function checkComponentUpgradable(
  ctx: Context,
  componentsApi: ComponentsApi,
  workspaceId: string,
  componentIdOrName: string,
): Promise<boolean> {
  ctx.logger.debug(`Checking if component can be upgraded: {component}`, {
    component: componentIdOrName,
  });

  const isUlid = /^[0-9A-HJKMNP-TV-Z]{26}$/i.test(componentIdOrName);

  try {
    const componentResponse = await componentsApi.findComponent({
      workspaceId,
      changeSetId: "HEAD",
      ...(isUlid
        ? { componentId: componentIdOrName }
        : { component: componentIdOrName }),
    });

    const component = componentResponse.data.component;

    if (!component.canBeUpgraded) {
      ctx.logger.info(
        `Component {name} ({id}) is already at the latest schema version`,
        { name: component.name, id: component.id },
      );
      return false;
    }

    return true;
  } catch (error) {
    // deno-lint-ignore no-explicit-any
    const statusCode = (error as any)?.response?.status || (error as any)?.status;

    if (statusCode === 404) {
      ctx.logger.error(`Component '{component}' not found`, {
        component: componentIdOrName,
      });
      Deno.exit(1);
    }
    throw error;
  }
}

/**
 * Check if there are any upgradable components
 * Returns true if there are upgradable components, false otherwise
 */
async function checkAnyUpgradable(
  ctx: Context,
  searchApi: SearchApi,
  workspaceId: string,
  schemaCategory?: string,
): Promise<boolean> {
  ctx.logger.debug("Checking for upgradable components...");

  const searchParts = ["isUpgradable:true"];
  if (schemaCategory) {
    searchParts.push(`category:${schemaCategory}`);
  }
  const searchString = searchParts.join(" ");

  try {
    const searchResponse = await searchApi.search({
      workspaceId,
      changeSetId: "HEAD",
      q: searchString,
    });

    const upgradableComponents = searchResponse.data.components || [];

    if (upgradableComponents.length === 0) {
      const filterMsg = schemaCategory
        ? ` matching category '${schemaCategory}'`
        : "";
      ctx.logger.info(`No upgradable components found${filterMsg}`);
      return false;
    }

    ctx.logger.debug(`Found {count} upgradable component(s)`, {
      count: upgradableComponents.length,
    });
    return true;
  } catch (error) {
    // deno-lint-ignore no-explicit-any
    const errorMessage = (error as any)?.response?.data?.message ||
      // deno-lint-ignore no-explicit-any
      (error as any)?.message ||
      "Unknown error";

    ctx.logger.error(`Failed to search for upgradable components: {error}`, {
      error: errorMessage,
    });
    throw error;
  }
}

/**
 * Upgrades one or more components to the latest schema version.
 *
 * This function:
 * 1. Creates or uses existing change set
 * 2. Validates that either a component or --all flag is provided
 * 3. For single component: finds and upgrades the specified component
 * 4. For --all flag: searches for upgradable components (optionally filtered by schema category)
 * 5. In dry-run mode, lists what would be upgraded without applying changes
 * 6. On error, abandons auto-created change sets
 *
 * @param componentIdOrName - Optional component ID (ULID) or name. Must provide this OR options.all
 * @param options - Command options including changeSet, all, schemaCategory, and dryRun
 * @throws Error if neither component nor --all provided, or if both provided, or if upgrade fails
 */
export async function callComponentUpgrade(
  componentIdOrName: string | undefined,
  options: ComponentUpgradeOptions,
): Promise<void> {
  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

  // Validate that either a component or --all flag is provided
  if (!componentIdOrName && !options.all) {
    ctx.logger.error(
      "Either a component name/ID or --all flag must be specified",
    );
    ctx.logger.info("");
    ctx.logger.info("Usage:");
    ctx.logger.info("  si component upgrade <component>    # Upgrade specific component");
    ctx.logger.info("  si component upgrade --all          # Upgrade all upgradable components");
    ctx.logger.info("");
    Deno.exit(1);
  }

  // Validate that both component and --all are not provided
  if (componentIdOrName && options.all) {
    ctx.logger.error(
      "Cannot specify both a component name/ID and --all flag",
    );
    Deno.exit(1);
  }

  // Create API clients
  const changeSetsApi = new ChangeSetsApi(apiConfig);
  const componentsApi = new ComponentsApi(apiConfig);
  const searchApi = new SearchApi(apiConfig);
  const schemasApi = new SchemasApi(apiConfig);

  // Check if there's anything to upgrade BEFORE creating a change set
  const hasWork = componentIdOrName
    ? await checkComponentUpgradable(ctx, componentsApi, workspaceId, componentIdOrName)
    : await checkAnyUpgradable(ctx, searchApi, workspaceId, options.schemaCategory);

  if (!hasWork) {
    return; // Nothing to do, exit cleanly
  }

  // Now we know there's something to upgrade - create/get change set
  // Determine operation name for change set
  let operationName = "Upgrade components";
  if (componentIdOrName) {
    operationName = `Upgrade ${componentIdOrName}`;
  } else if (options.schemaCategory) {
    operationName = `Upgrade ${options.schemaCategory} components`;
  }

  // Get or create change set
  const { changeSetId, wasCreated } = await getOrCreateChangeSet(
    ctx,
    changeSetsApi,
    workspaceId,
    options.changeSet,
    operationName,
  );

  if (wasCreated) {
    ctx.logger.info(`Created change set: {changeSetId}`, { changeSetId });
  } else {
    ctx.logger.debug(`Using change set: {changeSetId}`, { changeSetId });
  }

  try {
    // If a specific component is provided, upgrade just that one
    if (componentIdOrName) {
      ctx.logger.info(`Upgrading component: {component}`, {
        component: componentIdOrName,
      });

      // We already checked this component exists and can be upgraded
      // Now find it in the change set (not HEAD) to get the ID for upgrade
      const isUlid = /^[0-9A-HJKMNP-TV-Z]{26}$/i.test(componentIdOrName);

      const componentResponse = await componentsApi.findComponent({
        workspaceId,
        changeSetId: changeSetId,
        ...(isUlid
          ? { componentId: componentIdOrName }
          : { component: componentIdOrName }),
      });

      const component = componentResponse.data.component;

      ctx.logger.debug(`Found component: {id} ({name})`, {
        id: component.id,
        name: component.name,
      });

    // Dry-run mode: log what would be upgraded
    if (options.dryRun) {
      ctx.logger.info(`Would upgrade component: {name} ({id})`, {
        name: component.name,
        id: component.id,
      });
      ctx.logger.info("Dry-run mode - no changes applied");
      return;
    }

    // Upgrade the component
    try {
      const upgradeResponse = await componentsApi.upgradeComponent({
        workspaceId,
        changeSetId: changeSetId,
        componentId: component.id,
      });

      ctx.logger.info(
        `Successfully upgraded component: {name} ({id})`,
        {
          name: upgradeResponse.data.component.name,
          id: upgradeResponse.data.component.id,
        },
      );

      // Track component upgrade
      const schemaResponse = await schemasApi.getSchema({
        workspaceId,
        changeSetId,
        schemaId: component.schemaId,
      });

      ctx.analytics.trackEvent("component upgrade", {
        schemaName: schemaResponse.data.name,
        dryRun: options.dryRun ?? false,
      });
    } catch (error) {
      // Handle specific error messages from the API
      // deno-lint-ignore no-explicit-any
      const errorMessage = (error as any)?.response?.data?.message ||
        // deno-lint-ignore no-explicit-any
        (error as any)?.message ||
        "Unknown error";

      // Check for common upgrade errors
      if (errorMessage.includes("cannot be upgraded")) {
        ctx.logger.error(
          `Component {name} cannot be upgraded: {error}`,
          { name: component.name, error: errorMessage },
        );
      } else if (errorMessage.includes("no newer version")) {
        ctx.logger.info(
          `Component {name} is already at the latest version`,
          { name: component.name },
        );
      } else if (errorMessage.includes("HEAD")) {
        ctx.logger.error(
          "Cannot upgrade components on HEAD change set",
        );
      } else {
        ctx.logger.error(
          `Failed to upgrade component {name}: {error}`,
          { name: component.name, error: errorMessage },
        );
      }

      throw error;
    }

    return;
  }

  // No specific component provided - upgrade all upgradable components
  ctx.logger.info("Searching for upgradable components...");

  // Build search query
  const searchParts = ["isUpgradable:true"];
  if (options.schemaCategory) {
    searchParts.push(`category:${options.schemaCategory}`);
  }
  const searchString = searchParts.join(" ");

  ctx.logger.debug(`Search query: {query}`, { query: searchString });

  let upgradableComponents;
  try {
    const searchResponse = await searchApi.search({
      workspaceId,
      changeSetId: changeSetId,
      q: searchString,
    });

    upgradableComponents = searchResponse.data.components || [];
  } catch (error) {
    // deno-lint-ignore no-explicit-any
    const errorMessage = (error as any)?.response?.data?.message ||
      // deno-lint-ignore no-explicit-any
      (error as any)?.message ||
      "Unknown error";

    ctx.logger.error(
      `Failed to search for upgradable components: {error}`,
      { error: errorMessage },
    );
    throw error;
  }

  if (upgradableComponents.length === 0) {
    const filterMsg = options.schemaCategory
      ? ` matching category '${options.schemaCategory}'`
      : "";
    ctx.logger.info(`No upgradable components found${filterMsg}`);
    return;
  }

  ctx.logger.info(
    `Found {count} upgradable component(s)`,
    { count: upgradableComponents.length },
  );

  // Dry-run mode: list what would be upgraded
  if (options.dryRun) {
    ctx.logger.info("Components that would be upgraded:");
    for (const comp of upgradableComponents) {
      ctx.logger.info(`  - {name} ({id})`, {
        name: comp.name,
        id: comp.id,
      });
    }
    ctx.logger.info("Dry-run mode - no changes applied");
    return;
  }

  // Upgrade each component
  let successCount = 0;
  let failureCount = 0;

  for (const comp of upgradableComponents) {
    try {
      ctx.logger.info(`Upgrading: {name} ({id})`, {
        name: comp.name,
        id: comp.id,
      });

      await componentsApi.upgradeComponent({
        workspaceId,
        changeSetId: changeSetId,
        componentId: comp.id,
      });

      ctx.logger.info(`  ✓ Successfully upgraded {name}`, {
        name: comp.name,
      });
      successCount++;

      // Track individual upgrade
      ctx.analytics.trackEvent("component upgrade", {
        schemaName: comp.schema.name,
        dryRun: false,
        bulk: true,
      });
    } catch (error) {
      // deno-lint-ignore no-explicit-any
      const errorMessage = (error as any)?.response?.data?.message ||
        // deno-lint-ignore no-explicit-any
        (error as any)?.message ||
        "Unknown error";

      ctx.logger.error(`  ✗ Failed to upgrade {name}: {error}`, {
        name: comp.name,
        error: errorMessage,
      });
      failureCount++;
    }
  }

  // Summary
  ctx.logger.info(
    `Upgrade complete: {success} succeeded, {failure} failed`,
    { success: successCount, failure: failureCount },
  );

    if (failureCount > 0) {
      throw new Error(
        `${failureCount} component(s) failed to upgrade. See errors above for details.`,
      );
    }
  } catch (error) {
    ctx.logger.error(`Failed to upgrade component(s): ${error}`);

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

    throw error;
  }
}
