/**
 * Change Set List Module - List all change sets
 *
 * This module provides functionality to list all change sets in the
 * System Initiative workspace.
 *
 * @module
 */

import { ChangeSetsApi, type ChangeSetViewV1 } from "@systeminit/api-client";
import { stringify as stringifyYaml } from "@std/yaml";
import { Context } from "../context.ts";
import type { ChangeSetListOptions } from "./types.ts";

export type { ChangeSetListOptions };

/**
 * Main entry point for the change-set list command
 */
export async function callChangeSetList(
  options: ChangeSetListOptions,
): Promise<void> {
  const ctx = Context.instance();

  try {
    const apiConfig = Context.apiConfig();
    const workspaceId = Context.workspaceId();

    // Determine output format early to control logging
    const outputFormat = options.output || "info";

    // Only log when in info mode to keep json/yaml output clean
    if (outputFormat === "info") {
      ctx.logger.info("Listing change sets...");
    }

    const changeSetsApi = new ChangeSetsApi(apiConfig);

    const response = await changeSetsApi.listChangeSets({
      workspaceId,
    });

    const changeSets = response.data.changeSets as ChangeSetViewV1[];
    const count = changeSets.length;

    ctx.logger.debug(`Found {count} change sets`, { count });

    // Handle empty results
    if (count === 0) {
      if (outputFormat === "info") {
        ctx.logger.warn("No change sets found");
      } else if (outputFormat === "json") {
        console.log(JSON.stringify({ count: 0, changeSets: [] }, null, 2));
      } else if (outputFormat === "yaml") {
        console.log(stringifyYaml({ count: 0, changeSets: [] }));
      }
      return;
    }

    // Build output structure
    const output = {
      count,
      changeSets: changeSets.map((cs) => ({
        id: cs.id,
        name: cs.name,
        status: cs.status,
        isHead: cs.isHead,
      })),
    };

    // Handle output based on format
    if (outputFormat === "json") {
      console.log(JSON.stringify(output, null, 2));
    } else if (outputFormat === "yaml") {
      console.log(stringifyYaml(output));
    } else {
      // Info mode: display results
      ctx.logger.info("Found {count} change sets", { count });

      for (let i = 0; i < changeSets.length; i++) {
        const cs = changeSets[i];
        const index = i + 1;

        ctx.logger.info("Change Set {index}/{total}: {name}", {
          index,
          total: count,
          name: cs.name,
        });
        ctx.logger.info("  ID: {id}", { id: cs.id });
        ctx.logger.info("  Status: {status}", { status: cs.status });
        if (cs.isHead) {
          ctx.logger.info("  HEAD: {head}", { head: true });
        }
      }
    }
    
    ctx.analytics.trackEvent("change set list", {
      changeSets: changeSets.length,
    });
    
  } catch (error) {
    ctx.logger.error(`Failed to list change sets: ${error}`);
    Deno.exit(1);
  }
}
