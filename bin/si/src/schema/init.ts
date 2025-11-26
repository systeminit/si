/**
 * Project initialization commands for SI.
 *
 * This module provides functionality to initialize new SI projects by
 * creating the necessary project directory structure and marker files.
 *
 * @module
 */

import type { Context } from "../context.ts";
import { getLogger } from "../logger.ts";
import { type AbsoluteDirectoryPath, Project } from "./project.ts";

const logger = getLogger();

export async function callProjectInit(
  ctx: Context,
  rootPath: AbsoluteDirectoryPath,
) {
  logger.info("Initializing SI project");
  logger.info("---");
  logger.info("");

  if (await rootPath.exists()) {
    logger.info("  - Exists:  {rootPath}", {
      rootPath: rootPath.toString(),
    });
  } else {
    await rootPath.mkdir({ recursive: true });
    logger.info("  - Created: {rootPath}", {
      rootPath: rootPath.toString(),
    });
  }

  const projectMarkerPath = Project.projectMarkerPath(rootPath);
  if (await projectMarkerPath.exists()) {
    logger.info("  - Exists:  {projectMarkerPath}", {
      projectMarkerPath: projectMarkerPath.toString(),
    });
  } else {
    await projectMarkerPath.create();
    logger.info("  - Created: {projectMarkerPath}", {
      projectMarkerPath: projectMarkerPath.toString(),
    });
  }

  ctx.analytics.trackEvent("project_init");

  logger.info("");
  logger.info("---");
  logger.info("SI project successfully initialized");
  logger.info(`   ${rootPath.toString()}`);
}
