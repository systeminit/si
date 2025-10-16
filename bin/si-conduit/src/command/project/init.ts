/**
 * Project initialization commands for SI Conduit.
 *
 * This module provides functionality to initialize new SI Conduit projects by
 * creating the necessary project directory structure and marker files.
 *
 * @module
 */

import { Context } from "../../context.ts";
import { AbsoluteDirectoryPath, Project } from "../../project.ts";

export async function callProjectInit(
  ctx: Context,
  rootPath: AbsoluteDirectoryPath,
) {
  const logger = ctx.logger;

  logger.info("Initializing Conduit project");
  logger.info("----------------------------");
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

  logger.info("");
  logger.info("Conduit project initialized");
  logger.info("");
  logger.info(`   ${rootPath.toString()}`);
}
