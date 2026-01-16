/**
 * Change Set Review Module - Review all changes in a change set
 *
 * This module provides functionality to review all component changes in a
 * change set, showing attribute diffs and subscription sources.
 *
 * @module
 */

import { ChangeSetsApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import { resolveChangeSet } from "./utils.ts";
import type { ChangeSetReviewOptions } from "./types.ts";

export type { ChangeSetReviewOptions };

/**
 * Main entry point for the change-set review command
 */
export async function callChangeSetReview(
  options: ChangeSetReviewOptions,
): Promise<void> {
  const ctx = Context.instance();

  try {
    const apiConfig = Context.apiConfig();
    const workspaceId = Context.workspaceId();

    ctx.logger.info("Fetching change set review...");

    // Resolve change set ID from name or ID
    const changeSetId = await resolveChangeSet(
      workspaceId,
      options.changeSetIdOrName,
    );

    const changeSetsApi = new ChangeSetsApi(apiConfig);

    const response = await changeSetsApi.reviewChangeSet({
      workspaceId,
      changeSetId,
      includeResourceDiff: options.includeResourceDiff || false,
    });

    const reviewData = response.data;

    // Handle building response (202) - check for status field
    if ("status" in reviewData && reviewData.status === "building") {
      const buildingData = reviewData as unknown as {
        status: string;
        message: string;
        retryAfterSeconds: number;
        estimatedCompletionSeconds: number;
      };

      ctx.logger.warn(
        "Change set review data is still being generated. Please retry in a few seconds.",
      );
      ctx.logger.info(
        `Retry in ${buildingData.retryAfterSeconds} seconds (estimated completion: ${buildingData.estimatedCompletionSeconds}s)`,
      );
      Deno.exit(0);
    }

    // Type guard for successful response
    if (!("components" in reviewData)) {
      ctx.logger.error("Unexpected response format from API");
      Deno.exit(1);
    }

    const { components, summary } = reviewData;

    // Display summary
    if (components.length === 0) {
      ctx.logger.info("No components have changes in this change set");
      return;
    }

    ctx.logger.info(
      `Found ${summary.totalComponents} component(s) with changes: ${summary.added} added, ${summary.modified} modified, ${summary.removed} removed`,
    );
    ctx.logger.info("");

    // Display each component's changes
    for (const component of components) {
      ctx.logger.info(
        `Component: ${component.componentName} (${component.schemaName})`,
      );
      ctx.logger.info(`Status: ${component.diffStatus}`);

      const attributeDiffs = Object.entries(component.attributeDiffs || {});

      if (component.diffStatus === "Added") {
        ctx.logger.info("All attributes are new:");
      }

      for (const [path, diff] of attributeDiffs) {
        if (diff.changeType === "added") {
          ctx.logger.info(`  + "${path}"`);
          displayAttributeValue("      ", diff, "new");
        } else if (diff.changeType === "modified") {
          ctx.logger.info(`  ~ "${path}"`);
          ctx.logger.info("      Old:");
          displayAttributeValue("        ", diff, "old");
          ctx.logger.info("      New:");
          displayAttributeValue("        ", diff, "new");
        } else if (diff.changeType === "removed") {
          ctx.logger.info(`  - "${path}"`);
          displayAttributeValue("      ", diff, "old");
        }
      }

      ctx.logger.info("");
    }

    // Display summary again at the end
    ctx.logger.info(
      `Summary: ${summary.added} added, ${summary.modified} modified, ${summary.removed} removed`,
    );

    ctx.analytics.trackEvent("change set review", {
      changeSetId,
      componentsChanged: summary.totalComponents,
      added: summary.added,
      modified: summary.modified,
      removed: summary.removed,
    });
  } catch (error: unknown) {
    // Handle 400 error for HEAD change set
    if (
      error &&
      typeof error === "object" &&
      "response" in error &&
      error.response &&
      typeof error.response === "object" &&
      "status" in error.response &&
      error.response.status === 400
    ) {
      ctx.logger.error(
        "Cannot review HEAD change set - HEAD has no diffs to review",
      );
      Deno.exit(1);
    }

    ctx.logger.error(`Failed to review change set: ${error}`);
    Deno.exit(1);
  }
}

/**
 * Display a single attribute value with proper formatting
 */
function displayAttributeValue(
  indent: string,
  // deno-lint-ignore no-explicit-any
  diff: any,
  type: "old" | "new",
): void {
  const ctx = Context.instance();

  const value = type === "new" ? diff.newValue : diff.oldValue;
  const sourceType = type === "new" ? diff.newSourceType : diff.oldSourceType;

  if (sourceType === "subscription") {
    const componentName = type === "new"
      ? diff.newSourceComponentName
      : diff.oldSourceComponentName;
    const sourcePath = type === "new" ? diff.newSourcePath : diff.oldSourcePath;

    if (componentName) {
      ctx.logger.info(
        `${indent}Value: "$source: ${componentName} -> ${sourcePath}"`,
      );
    } else {
      ctx.logger.info(`${indent}Value: "$source: -> ${sourcePath}"`);
    }
  } else if (sourceType === "prototype") {
    const prototype = type === "new"
      ? diff.newSourcePrototype
      : diff.oldSourcePrototype;
    ctx.logger.info(`${indent}Value: "$source: ${prototype}"`);
  } else {
    // Static value
    if (typeof value === "string") {
      ctx.logger.info(`${indent}Value: "${value}"`);
    } else if (value === null || value === undefined) {
      ctx.logger.info(`${indent}Value: ${value}`);
    } else {
      ctx.logger.info(`${indent}Value: ${JSON.stringify(value)}`);
    }
  }
}
