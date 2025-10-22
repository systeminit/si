import { getLogger } from "./logger.ts";
import { Project } from "./project.ts";
import type { AbsoluteFilePath } from "./project.ts";

const logger = getLogger();

export async function materializeSchemaBase(
  project: Project,
  schemaName: string,
) {
  const schemaBasePath = project.schemaBasePath(schemaName);

  // Create schema base directory
  if (!(await schemaBasePath.exists())) {
    await schemaBasePath.mkdir({ recursive: true });
    logger.info("Created schema directory: {schemaBasePath}", {
      schemaBasePath: schemaBasePath.toString(),
    });
  }
}

export async function materializeSchemaActionBase(
  project: Project,
  schemaName: string,
) {
  const actionBasePath = project.actionBasePath(schemaName);

  // Create the action base directory
  if (!(await actionBasePath.exists())) {
    await actionBasePath.mkdir({ recursive: true });
    logger.info("Created actions directory: {actionBasePath}", {
      actionBasePath: actionBasePath.toString(),
    });
  }
}

export async function materializeSchemaCodegenBase(
  project: Project,
  schemaName: string,
) {
  const codegenBasePath = project.codegenBasePath(schemaName);

  // Create the codegen base directory
  if (!(await codegenBasePath.exists())) {
    await codegenBasePath.mkdir({ recursive: true });
    logger.info("Created code generators directory: {codegenBasePath}", {
      codegenBasePath: codegenBasePath.toString(),
    });
  }
}

export async function materializeSchemaManagementBase(
  project: Project,
  schemaName: string,
) {
  const managementBasePath = project.managementBasePath(schemaName);

  // Create the management base directory
  if (!(await managementBasePath.exists())) {
    await managementBasePath.mkdir({ recursive: true });
    logger.info("Created management directory: {managementBasePath}", {
      managementBasePath: managementBasePath.toString(),
    });
  }
}

export async function materializeSchemaQualificationBase(
  project: Project,
  schemaName: string,
) {
  const qualificationBasePath = project.qualificationBasePath(schemaName);

  // Create the qualification base directory
  if (!(await qualificationBasePath.exists())) {
    await qualificationBasePath.mkdir({ recursive: true });
    logger.info("Created qualifications directory: {qualificationBasePath}", {
      qualificationBasePath: qualificationBasePath.toString(),
    });
  }
}

export async function materializeSchemaFormatVersion(
  project: Project,
  schemaName: string,
  formatVersionBody: string,
): Promise<{ formatVersionPath: AbsoluteFilePath }> {
  // Create the format version file
  const formatVersionFilePath = project.schemaFormatVersionPath(schemaName);
  await ensureFileDoesNotExist(formatVersionFilePath);
  await createFileWithLogging(formatVersionFilePath, formatVersionBody);

  return {
    formatVersionPath: formatVersionFilePath,
  };
}

export async function materializeSchema(
  project: Project,
  schemaName: string,
  metadataBody: string,
  codeBody: string,
): Promise<{ metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }> {
  // Create schema metadata file
  const metadataFilePath = project.schemaMetadataPath(schemaName);
  await ensureFileDoesNotExist(metadataFilePath);
  await createFileWithLogging(metadataFilePath, metadataBody);

  // Create schema func code file
  const schemaCodeFilePath = project.schemaFuncCodePath(schemaName);
  await ensureFileDoesNotExist(schemaCodeFilePath);
  await createFileWithLogging(schemaCodeFilePath, codeBody);

  return {
    metadataPath: metadataFilePath,
    codePath: schemaCodeFilePath,
  };
}

export async function materializeSchemaAction(
  project: Project,
  schemaName: string,
  actionName: string,
  metadataBody: string,
  codeBody: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  // Create action func metadata file
  const actionMetadataFilePath = project.actionFuncMetadataPath(
    schemaName,
    actionName,
  );
  await ensureFileDoesNotExist(actionMetadataFilePath);
  await createFileWithLogging(actionMetadataFilePath, metadataBody);

  // Create action func code file
  const actionCodeFilePath = project.actionFuncCodePath(schemaName, actionName);
  await ensureFileDoesNotExist(actionCodeFilePath);
  await createFileWithLogging(actionCodeFilePath, codeBody);

  return {
    metadataPath: actionMetadataFilePath,
    codePath: actionCodeFilePath,
  };
}

export async function materializeSchemaCodegen(
  project: Project,
  schemaName: string,
  codegenName: string,
  metadataBody: string,
  codeBody: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  // Create codegen func metadata file
  const codegenMetadataFilePath = project.codegenFuncMetadataPath(
    schemaName,
    codegenName,
  );
  await ensureFileDoesNotExist(codegenMetadataFilePath);
  await createFileWithLogging(codegenMetadataFilePath, metadataBody);

  // Create codegen func code file
  const codegenCodeFilePath = project.codegenFuncCodePath(
    schemaName,
    codegenName,
  );
  await ensureFileDoesNotExist(codegenCodeFilePath);
  await createFileWithLogging(codegenCodeFilePath, codeBody);

  return {
    metadataPath: codegenMetadataFilePath,
    codePath: codegenCodeFilePath,
  };
}

export async function materializeSchemaManagement(
  project: Project,
  schemaName: string,
  managementName: string,
  metadataBody: string,
  codeBody: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  // Create management func metadata file
  const managementMetadataFilePath = project.managementFuncMetadataPath(
    schemaName,
    managementName,
  );
  await ensureFileDoesNotExist(managementMetadataFilePath);
  await createFileWithLogging(managementMetadataFilePath, metadataBody);

  // Create management func code file
  const managementCodeFilePath = project.managementFuncCodePath(
    schemaName,
    managementName,
  );
  await ensureFileDoesNotExist(managementCodeFilePath);
  await createFileWithLogging(managementCodeFilePath, codeBody);

  return {
    metadataPath: managementMetadataFilePath,
    codePath: managementCodeFilePath,
  };
}

export async function materializeSchemaQualification(
  project: Project,
  schemaName: string,
  qualificationName: string,
  metadataBody: string,
  codeBody: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  // Create qualification func metadata file
  const qualificationMetadataFilePath = project.qualificationFuncMetadataPath(
    schemaName,
    qualificationName,
  );
  await ensureFileDoesNotExist(qualificationMetadataFilePath);
  await createFileWithLogging(qualificationMetadataFilePath, metadataBody);

  // Create qualification func code file
  const qualificationCodeFilePath = project.qualificationFuncCodePath(
    schemaName,
    qualificationName,
  );
  await ensureFileDoesNotExist(qualificationCodeFilePath);
  await createFileWithLogging(qualificationCodeFilePath, codeBody);

  return {
    metadataPath: qualificationMetadataFilePath,
    codePath: qualificationCodeFilePath,
  };
}

/**
 * Ensures that a file does not exist at the given path.
 * Throws FileExistsError if the file already exists.
 */
export async function ensureFileDoesNotExist(
  filePath: AbsoluteFilePath,
): Promise<void> {
  if (await filePath.exists()) {
    logger.error("File already exists at {filePath}", {
      filePath: filePath.toString(),
    });
    throw new FileExistsError(filePath.toString());
  }
}

/**
 * Creates a file with the given content and logs the creation.
 */
export async function createFileWithLogging(
  filePath: AbsoluteFilePath,
  content: string,
) {
  await filePath.writeTextFile(content);
  logger.info("Created: {filePath}", {
    filePath: filePath.toString(),
  });
}

/**
 * Error thrown when attempting to create a file that already exists.
 */
export class FileExistsError extends Error {
  constructor(public readonly path: string) {
    super(`File already exists at: ${path}`);
    this.name = "FileExistsError";
  }
}
