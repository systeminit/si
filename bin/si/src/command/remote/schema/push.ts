import {
  ChangeSetsApi,
  type FindSchemaV1Response,
  FuncsApi,
  type GetFuncV1Response,
  SchemasApi,
} from "@systeminit/api-client";
import type { Context } from "../../../context.ts";
import type { AuthenticatedCliContext } from "../../../cli/helpers.ts";
import { unknownValueToErrorMessage } from "../../../helpers.ts";
import { SCHEMA_FILE_FORMAT_VERSION } from "../../../config.ts";
import {
  type AbsoluteDirectoryPath,
  FunctionKind,
  normalizeFsName,
  type Project,
} from "../../../project.ts";
import type { Logger } from "@logtape/logtape";
import { wrapInChangeSet } from "../../../change_set_utils.ts";

async function parseActions(
  ctx: Context,
  schemaName: string,
  funcBasePath: AbsoluteDirectoryPath,
) {
  const logger = ctx.logger;
  const validActionKinds = ["create", "destroy", "refresh", "update"];
  const actionFiles = [];

  logger.debug(`reading ${funcBasePath.toString()}...`);

  // Read the file list from the actions folder
  try {
    const actionEntries = Deno.readDir(funcBasePath.toString());

    for await (const actionEntry of actionEntries) {
      if (!actionEntry.isFile || !actionEntry.name.endsWith(".ts")) {
        continue;
      }

      const actionFileName = actionEntry.name;
      const strippedFileName = actionFileName.replace(/\.ts$/, "");
      const kind = validActionKinds.includes(strippedFileName)
        ? strippedFileName.charAt(0).toUpperCase() + strippedFileName.slice(1) // Uppercase first letter
        : "Manual";

      actionFiles.push({
        kind,
        strippedFileName,
      });
    }
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw new Error(
        `Error reading actions directory for asset "${schemaName}": ${
          unknownValueToErrorMessage(
            error,
          )
        }`,
      );
    }
    // If actions folder doesn't exist, that's ok - just continue
    return [];
  }

  const actions = [];

  // For each listed file, get the contents and the required metadata file ({kind}.json)
  for (const { kind, strippedFileName } of actionFiles) {
    const actionTsPath = `${funcBasePath.toString()}/${strippedFileName}.ts`;
    const actionJsonPath =
      `${funcBasePath.toString()}/${strippedFileName}.metadata.json`;

    try {
      // Read the TypeScript file
      const code = await Deno.readTextFile(actionTsPath);

      if (!code || code.trim() === "") {
        logger.error(
          `Empty code in action file "${actionTsPath}" for asset "${schemaName}", skipping...`,
        );
        continue;
      }

      // Read the matching JSON file
      let actionJsonContent;
      try {
        actionJsonContent = await Deno.readTextFile(actionJsonPath);
      } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
          logger.error(
            `No matching .json file found for action "${actionTsPath}" in asset "${schemaName}", skipping...`,
          );
          continue;
        }
        throw error;
      }

      const actionMetadata = JSON.parse(actionJsonContent);

      let name = actionMetadata.name;
      // Validate required name field
      if (!name || name.trim() === "") {
        if (kind === "Manual") {
          name = strippedFileName;
        } else {
          logger.error(
            `Missing required 'name' field in "${actionJsonPath}" for non manual asset "${schemaName}", skipping...`,
          );
          continue;
        }
      }

      // Build action object
      const actionObject = {
        name,
        displayName: actionMetadata.displayName,
        description: actionMetadata.description,
        kind,
        code: code,
      };

      actions.push(actionObject);
    } catch (error) {
      logger.error(
        `Error processing action "${strippedFileName}" for asset "${schemaName}": ${
          unknownValueToErrorMessage(
            error,
          )
        }, skipping...`,
      );
    }
  }

  return actions;
}

type ActionArray = Awaited<ReturnType<typeof parseActions>>;
type Action = ActionArray[number];

async function parseSimpleFuncDirectory(
  ctx: Context,
  schemaName: string,
  funcBasePath: AbsoluteDirectoryPath,
) {
  const logger = ctx.logger;
  const files = [] as string[];

  logger.debug(`reading ${funcBasePath.toString()}...`);

  // Read the file list from the actions folder
  try {
    const entries = Deno.readDir(funcBasePath.toString());

    for await (const entry of entries) {
      if (!entry.isFile || !entry.name.endsWith(".ts")) {
        continue;
      }

      const fileName = entry.name;
      const strippedFileName = fileName.replace(/\.ts$/, "");

      files.push(strippedFileName);
    }
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw new Error(
        `Error reading funcs directory "${funcBasePath}" for asset "${schemaName}": ${
          unknownValueToErrorMessage(
            error,
          )
        }`,
      );
    }
    // If folder doesn't exist, skip by returning early
    return [];
  }

  const funcs = [];

  for (const strippedFileName of files) {
    const funcTsPath = `${funcBasePath.toString()}/${strippedFileName}.ts`;
    const funcJsonPath =
      `${funcBasePath.toString()}/${strippedFileName}.metadata.json`;

    try {
      // Read the TypeScript file
      const code = await Deno.readTextFile(funcTsPath);

      if (!code || code.trim() === "") {
        logger.error(
          `Empty code in file "${funcTsPath}" for asset "${schemaName}", skipping...`,
        );
        continue;
      }

      // Read the matching JSON file
      let metadata = {} as Record<string, string>;
      try {
        const jsonContent = await Deno.readTextFile(funcJsonPath);
        metadata = JSON.parse(jsonContent);
      } catch (error) {
        // Not finding the metadata file is ok - just continue
        if (!(error instanceof Deno.errors.NotFound)) throw error;
      }

      const name = metadata.name ?? strippedFileName;

      // Build action object
      const funcObject = {
        name,
        displayName: metadata.displayName ?? name,
        description: metadata.description ?? null,
        code: code,
      };

      funcs.push(funcObject);
    } catch (error) {
      logger.error(
        `Error processing "${strippedFileName}" for asset "${schemaName}": ${
          unknownValueToErrorMessage(
            error,
          )
        }, skipping...`,
      );
    }
  }

  return funcs;
}

type SimpleFuncArray = Awaited<ReturnType<typeof parseSimpleFuncDirectory>>;
type SimpleFunc = SimpleFuncArray[number];

interface Schema {
  name: string;
  code: string;
  category: string;
  description: string;
  link?: string;
  color: string;
  actions: ActionArray;
  qualifications: SimpleFuncArray;
  codeGenerators: SimpleFuncArray;
  managementFuncs: SimpleFuncArray;
  authFuncs: SimpleFuncArray;
}

function allSchemaFuncsWithKind(schema: Schema) {
  const addKind = (kind: string, funcs: (Action | SimpleFunc)[]) =>
    funcs.map((func) => ({
      kind,
      funcData: func,
    }));

  return [
    ...addKind("Qualification", schema.qualifications),
    ...addKind("Action", schema.actions),
    ...addKind("CodeGeneration", schema.codeGenerators),
    ...addKind("Management", schema.managementFuncs),
    ...addKind("Authentication", schema.authFuncs),
  ];
}

export async function callRemoteSchemaPush(
  cliContext: AuthenticatedCliContext,
  project: Project,
  filterSchemaNames: string[],
  skipConfirmation?: boolean,
  updateBuiltins: boolean,
) {
  const { apiConfiguration, workspace, ctx } = cliContext;
  const logger = ctx.logger;

  const { instanceUrl: workspaceUrlPrefix, id: workspaceId } = workspace;

  const siFuncsApi = new FuncsApi(apiConfiguration);
  const changeSetsApi = new ChangeSetsApi(apiConfiguration);
  const siSchemasApi = new SchemasApi(apiConfiguration);

  const schemasFromFilesystem = [] as Schema[];

  // ==================================
  // Read schemas from the local folder
  // ==================================
  logger.info("reading schemas from filesystem...");
  let readSchemas = 0;
  let skippedSchemas = 0;
  try {
    const schemasBasePath = project.schemas.moduleBasePath().toString();

    const entries = Deno.readDir(schemasBasePath);

    const existingLocalSchemaNames = {} as Record<string, string>;

    for await (const entry of entries) {
      if (!entry.isDirectory) {
        continue;
      }

      const schemaDirName = entry.name;

      const normalizedDirName = normalizeFsName(entry.name);
      if (schemaDirName !== normalizedDirName) {
        logger.error(
          `Error reading ${schemaDirName}: Unsupported characters in the directory name. It should be ${normalizedDirName}. Skipping...`,
        );
        continue;
      }

      readSchemas += 1;

      const versionFilePath = project.schemas.formatVersionPath(schemaDirName);

      let thisFormatVersion: number;
      try {
        const formatContent = await versionFilePath.readTextFile();
        thisFormatVersion = parseInt(formatContent);
      } catch (error) {
        const msg = error instanceof Deno.errors.NotFound
          ? `.format-version file not found on the ${schemaDirName} directory`
          : unknownValueToErrorMessage(error);
        logger.error(msg);
        continue;
      }

      if (
        isNaN(thisFormatVersion) ||
        thisFormatVersion !== SCHEMA_FILE_FORMAT_VERSION
      ) {
        const msg =
          `Unsupported format version ${thisFormatVersion} on the asset ${schemaDirName}. ` +
          `Supported version is ${SCHEMA_FILE_FORMAT_VERSION}. ` +
          `You can run a new scaffold command and port your schema over to push it with this executable`;

        logger.error(msg);
        continue;
      }
      logger.debug(
        `found valid version file in ${schemaDirName}. reading schema...`,
      );

      const schemaCodePath = project.schemas.schemaFuncCodePath(schemaDirName);
      try {
        const code = await schemaCodePath.readTextFile();

        let schemaName: string;
        let category: string;
        let description: string;
        let link: string;
        let color: string;

        const schemaMetadataPath = project.schemas.schemaMetadataPath(
          schemaDirName,
        );
        try {
          const metadataContent = await schemaMetadataPath.readTextFile();
          const metadata = JSON.parse(metadataContent);

          schemaName = metadata.name;

          if (!schemaName || schemaName.trim() === "") {
            throw new Error("Missing required 'name' field in metadata.json");
          }

          if (
            filterSchemaNames.length > 0 &&
            !filterSchemaNames.includes(schemaName)
          ) {
            skippedSchemas += 1;
            logger.debug("skipping filtered schema: {schemaName}", {
              schemaName,
            });
            continue;
          }

          category = metadata.category || "";
          description = metadata.description || "";
          link = metadata.documentation ?? null;
          color = metadata.color ?? "#000000";
        } catch (error) {
          const msg = error instanceof Deno.errors.NotFound
            ? `metadata.json file not found on the ${schemaDirName} directory`
            : unknownValueToErrorMessage(error);
          logger.error(msg);
          continue;
        }

        const lowerCaseSchemaName = schemaName.toLowerCase();
        const existingLocalSchema =
          existingLocalSchemaNames[lowerCaseSchemaName];
        if (existingLocalSchema) {
          throw new Error(
            `Duplicate schema name "${schemaName}" found in asset "${schemaDirName}" and "${existingLocalSchema}" (names are case insensitive).`,
          );
        }
        existingLocalSchemaNames[lowerCaseSchemaName] = schemaDirName;
        logger.info(`reading schema ${schemaName}`);

        if (category === "") {
          // Try to get the category from the asset name
          const categoryMatch = schemaName.match(/^([^\s:]+::[^\s:]+)::(.+)$/);
          if (categoryMatch && categoryMatch[1] !== undefined) {
            category = categoryMatch[1];
          }
        }

        const qualifications = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.schemas.funcBasePath(
            schemaDirName,
            FunctionKind.Qualification,
          ),
        );

        logger.debug(
          `loaded ${qualifications.length} qualifications for ${schemaName}`,
        );

        const actions = await parseActions(
          ctx,
          schemaName,
          project.schemas.funcBasePath(schemaDirName, FunctionKind.Action),
        );

        logger.debug(`loaded ${actions.length} actions for ${schemaName}`);

        const codeGenerators = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.schemas.funcBasePath(schemaDirName, FunctionKind.Codegen),
        );

        logger.debug(
          `loaded ${codeGenerators.length} code generators for ${schemaName}`,
        );

        const managementFuncs = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.schemas.funcBasePath(schemaDirName, FunctionKind.Management),
        );

        logger.debug(
          `loaded ${managementFuncs.length} management funcs for ${schemaName}`,
        );

        const authFuncs = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.schemas.funcBasePath(schemaDirName, FunctionKind.Auth),
        );

        logger.debug(
          `authFuncs ${codeGenerators.length} auth funcs for ${schemaName}`,
        );

        const schema = {
          name: schemaName,
          code,
          category,
          description,
          link,
          color,
          qualifications,
          actions,
          codeGenerators,
          managementFuncs,
          authFuncs,
        };

        const existingFuncNames = {} as Record<string, string>;
        allSchemaFuncsWithKind(schema).forEach((func) => {
          const { name: funcName } = func.funcData;
          const { kind } = func;
          if (existingFuncNames[funcName]) {
            throw new Error(
              `Duplicate function name "${funcName}" found in asset "${schemaName}": ${
                existingFuncNames[funcName]
              } and ${kind}`,
            );
          }
          existingFuncNames[funcName] = kind;
        });

        schemasFromFilesystem.push(schema);
      } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
          logger.error(
            `No index.ts file found for asset "${schemaDirName}", skipping...\n`,
          );
          continue;
        }
        throw error;
      }
    }
  } catch (error) {
    throw new Error(
      `Error reading assets directory: ${unknownValueToErrorMessage(error)}`,
    );
  }
  const failedSchemaDirectories = readSchemas - schemasFromFilesystem.length - skippedSchemas;

  // ==================================
  // Pre check: Compare schema and funcs to head
  // ==================================
  const changeSets = (await changeSetsApi.listChangeSets({ workspaceId }))
    .data.changeSets as { id: string; isHead: boolean }[]; // Mocking this type since the client lib does not have it right now

  const headChangesetId = changeSets.find((cs) => cs.isHead)?.id;

  if (!headChangesetId) {
    throw new Error("No head change set found");
  }

  logger.info(`comparing filesystem assets to HEAD change set...`);

  const schemasToPush = [] as {
    schemaPayload: Schema;
    existingSchemaData?: {
      schemaId: string;
      variantDataChanged: boolean;
      funcsToUnbind: {
        id: string;
        kind: string;
        name: string;
      }[];
      updatableFuncIdsByName: Record<string, string>;
    };
  }[];

  for (const filesystemSchema of schemasFromFilesystem) {
    const existingSchema = await tryFindSchema(
      siSchemasApi,
      workspaceId,
      headChangesetId,
      filesystemSchema.name,
    );

    if (!existingSchema) {
      logger.info(
        `${filesystemSchema.name}: schema is new. enqueueing for push...`,
      );
      schemasToPush.push({ schemaPayload: filesystemSchema });
      continue;
    }

    logger.debug(
      `schema ${filesystemSchema.name} already exists. checking if it needs an update...`,
    );

    const existingVariant = (await siSchemasApi.getDefaultVariant({
      workspaceId,
      changeSetId: headChangesetId,
      schemaId: existingSchema.schemaId,
    })).data;

    if (!updateBuiltins && existingVariant.installedFromUpstream) {
      logger.warn(
        "{schemaName}: schema is a builtin. You can only push overlay funcs for it. skipping...",
        { schemaName: filesystemSchema.name },
      );
      continue;
    }

    const existingVariantCode = (await siFuncsApi.getFunc({
      workspaceId,
      changeSetId: headChangesetId,
      funcId: existingVariant.assetFuncId,
    })).data.code;

    const variantDataChanged = !schemaContentsMatch(filesystemSchema, {
      name: existingSchema.schemaName,
      category: existingVariant.category,
      description: existingVariant.description,
      link: existingVariant.link,
      color: existingVariant.color,
      code: existingVariantCode,
    });

    logger.info(
      `${filesystemSchema.name}: schema fields ${
        variantDataChanged ? "" : "un"
      }changed. Checking funcs... `,
    );

    // Go through the functions to see what needs to be done
    const existingFuncByName = {} as Record<string, {
      id: string;
      data: GetFuncV1Response;
    }>;
    const funcsToUnbindByName = {} as Record<string, {
      id: string;
      name: string;
      kind: string;
    }>;
    for (const { id: existingFuncId } of existingVariant.variantFuncs) {
      const existingFunc = (await siFuncsApi.getFunc({
        workspaceId,
        changeSetId: headChangesetId,
        funcId: existingFuncId,
      })).data;

      existingFuncByName[existingFunc.name] = {
        id: existingFuncId,
        data: existingFunc,
      };

      funcsToUnbindByName[existingFunc.name] = {
        id: existingFuncId,
        name: existingFunc.name,
        kind: existingFunc.kind,
      };
    }

    logger.trace(
      `- found ${Object.keys(existingFuncByName).length} existing funcs`,
    );

    // Compare existing functions to the filesystem data
    const actions = [] as ActionArray;
    const qualifications = [] as SimpleFuncArray;
    const codeGenerators = [] as SimpleFuncArray;
    const managementFuncs = [] as SimpleFuncArray;
    const authFuncs = [] as SimpleFuncArray;

    const updatableFuncIdsByName = {} as Record<string, string>;

    let createdFuncs = 0;
    let modifiedFuncs = 0;
    let skippedFuncs = 0;

    for (
      const filesystemFuncAndKind of allSchemaFuncsWithKind(filesystemSchema)
    ) {
      const filesystemFunc = filesystemFuncAndKind.funcData;
      const filesystemFuncKind = filesystemFuncAndKind.kind;

      const existingFunc = existingFuncByName[filesystemFunc.name];

      if (existingFunc && filesystemFuncKind === existingFunc.data.kind) {
        // If we'll update or skip, don't unbind
        delete funcsToUnbindByName[filesystemFunc.name];

        if (funcContentsMatch(filesystemFunc, existingFunc.data)) {
          skippedFuncs++;
          logger.trace(
            `- skipping unchanged ${filesystemFuncKind} function: ${filesystemFunc.name}`,
          );
          continue;
        }

        modifiedFuncs++;
        logger.debug(
          `- updating ${filesystemFuncKind} function: ${filesystemFunc.name}`,
        );
        updatableFuncIdsByName[filesystemFunc.name] = existingFunc.id;
      } else {
        createdFuncs++;
        logger.debug(
          `- creating ${filesystemFuncKind} function: ${filesystemFunc.name}`,
        );
      }

      switch (filesystemFuncKind) {
        case "Qualification":
          qualifications.push(filesystemFunc);
          break;
        case "Action":
          if (!("kind" in filesystemFunc)) {
            logger.error(
              `Action must have a 'kind' field, ${filesystemFunc.name} is missing it`,
            );
            continue;
          }
          actions.push(filesystemFunc);
          break;
        case "CodeGeneration":
          codeGenerators.push(filesystemFunc);
          break;
        case "Management":
          managementFuncs.push(filesystemFunc);
          break;
        case "Authentication":
          authFuncs.push(filesystemFunc);
          break;
      }
    }

    let detachedFuncs = 0;
    for (const [funcName, { kind }] of Object.entries(funcsToUnbindByName)) {
      detachedFuncs++;
      logger.debug(`- removing ${kind} function: ${funcName}`);
    }

    const functionsModified = createdFuncs > 0 || modifiedFuncs > 0 ||
      detachedFuncs > 0;

    if (!variantDataChanged && !functionsModified) {
      logger.info(`- no changes to push. skipping...`);
      continue;
    }

    logger.info(
      `- func summary: ${createdFuncs} created, ${modifiedFuncs} modified, ${detachedFuncs} detached, ${skippedFuncs} unchanged`,
    );

    const schemaToPush = {
      schemaPayload: {
        ...filesystemSchema,
        actions,
        qualifications,
        codeGenerators,
        managementFuncs,
        authFuncs,
      },
      existingSchemaData: {
        schemaId: existingSchema.schemaId,
        variantDataChanged,
        funcsToUnbind: Object.values(funcsToUnbindByName),
        updatableFuncIdsByName,
      },
    };

    schemasToPush.push(schemaToPush);
  }

  // ==================================
  // Confirmation prompt (and dry run logic, eventually)
  // ==================================
  if (schemasToPush.length === 0) {
    console.log("No assets to push. terminating.");
    return;
  }

  if (!skipConfirmation) {
    const failureMsg = failedSchemaDirectories > 0
      ? ` Failed to read ${failedSchemaDirectories} asset description(s).`
      : "";

    const skipsMsg = skippedSchemas > 0
      ? ` ${skippedSchemas} asset(s) filtered out.`
      : "";

    console.log(
      `Found ${schemasToPush.length} asset(s) to push.${failureMsg}${skipsMsg}`,
    );

    let confirmed = false;
    while (!confirmed) {
      console.log(
        "Do you want to continue? (y = yes, l = list assets, any other key = cancel)",
      );

      const input = await readRawInput();

      if (input === "l") {
        console.log("\nAssets to be pushed:");
        schemasToPush.forEach((schema) =>
          console.log(`  - ${schema.schemaPayload.name}`)
        );
        console.log();
      } else if (input === "y") {
        confirmed = true;
      } else {
        return;
      }
    }
  }

  // ==================================
  // Create schemas on the server
  // ==================================
  logger.info("creating change set and pushing schema changes...");
  await wrapInChangeSet(
    changeSetsApi,
    logger,
    workspaceId,
    "SI Schema Push",
    async (changeSetId) => {
      let pushedSchemas = 0;

      for (const schemaData of schemasToPush) {
        const { code, name, category, description, link, color } =
          schemaData.schemaPayload;

        const existingSchemaData = schemaData.existingSchemaData;

        let schemaId;
        let schemaVariantId;

        const updatableFuncIdsByName =
          existingSchemaData?.updatableFuncIdsByName ??
            {} as Record<string, string>;
        const funcsToUnbind = existingSchemaData?.funcsToUnbind ?? [];

        if (existingSchemaData) {
          logger.info(
            `existing schema ${name} (${existingSchemaData.schemaId}), unlocking and updating...`,
          );

          const unlockSchemaResponse = await siSchemasApi.unlockSchema({
            workspaceId,
            changeSetId,
            schemaId: existingSchemaData.schemaId,
          });

          schemaId = existingSchemaData.schemaId;
          schemaVariantId = unlockSchemaResponse.data.unlockedVariantId;

          if (existingSchemaData.variantDataChanged) {
            await siSchemasApi.updateSchemaVariant({
              workspaceId,
              changeSetId,
              schemaId,
              schemaVariantId,
              updateSchemaVariantV1Request: {
                name,
                code,
                category,
                description,
                link,
                color,
              },
            });
          }
        } else {
          logger.info(`creating schema ${name}...`);

          const createSchemaResponse = await siSchemasApi.createSchema({
            workspaceId,
            changeSetId,
            createSchemaV1Request: {
              name,
              code,
              category,
              color,
              description,
              link,
            },
          });

          schemaId = createSchemaResponse.data.schemaId;
          schemaVariantId = createSchemaResponse.data.defaultVariantId;
        }

        const baseVariantPayload = {
          workspaceId,
          changeSetId,
          schemaId,
          schemaVariantId,
        };

        for (const { id: funcId, kind, name: funcName } of funcsToUnbind) {
          logger.trace(`${kind} "${funcName}": detaching`);
          await unbindFunc(
            siSchemasApi,
            baseVariantPayload,
            kind,
            funcId,
          );
        }

        for (
          const filesystemFuncAndKind of allSchemaFuncsWithKind(
            schemaData.schemaPayload,
          )
        ) {
          const filesystemFunc = filesystemFuncAndKind.funcData;
          const filesystemFuncKind = filesystemFuncAndKind.kind;
          const funcName = filesystemFunc.name;

          const existingFuncId = updatableFuncIdsByName[funcName];
          if (existingFuncId) {
            logger.trace(
              `${filesystemFuncKind} "${funcName}": unlocking and updating`,
            );

            const unlockedFuncId = (await siFuncsApi.unlockFunc({
              workspaceId,
              changeSetId,
              funcId: existingFuncId,
              unlockFuncV1Request: {
                schemaVariantId,
              },
            })).data.unlockedFuncId;

            await siFuncsApi.updateFunc({
              workspaceId,
              changeSetId,
              funcId: unlockedFuncId,
              updateFuncV1Request: filesystemFunc,
            });
          } else {
            logger.trace(
              `${filesystemFuncKind} "${filesystemFunc.name}": creating`,
            );

            await createFunc(
              siSchemasApi,
              logger,
              baseVariantPayload,
              filesystemFuncKind,
              filesystemFunc,
              updateBuiltins,
            );
          }
        }

        pushedSchemas += 1;
      }

      const changeSetUrl =
        `${workspaceUrlPrefix}/w/${workspaceId}/${changeSetId}/l/a`;

      ctx.analytics.trackEvent("push_assets", {
        pushedSchemasCount: pushedSchemas,
        pushedSchemaNames: schemasFromFilesystem.map((schema) => schema.name),
        workspaceId,
        changeSetId,
        changeSetUrl,
      });

      console.log(
        `${pushedSchemas} schemas pushed. To see them, go to: ${changeSetUrl}`,
      );
    },
  );
}

export async function callRemoteSchemaOverlaysPush(
  cliContext: AuthenticatedCliContext,
  project: Project,
  skipConfirmation?: boolean,
) {
  const { apiConfiguration, workspace, ctx } = cliContext;
  const logger = ctx.logger;
  const { instanceUrl: workspaceUrlPrefix, id: workspaceId } = workspace;

  const siSchemasApi = new SchemasApi(apiConfiguration);
  const siFuncsApi = new FuncsApi(apiConfiguration);
  const changeSetsApi = new ChangeSetsApi(apiConfiguration);

  logger.info("Processing schema overlays for push...");
  logger.info("---");
  logger.info("");

  // Get HEAD change set
  const changeSets = (await changeSetsApi.listChangeSets({ workspaceId }))
    .data.changeSets as { id: string; isHead: boolean }[];

  const headChangesetId = changeSets.find((cs) => cs.isHead)?.id;

  if (!headChangesetId) {
    throw new Error("No head change set found");
  }

  // Get all assets that have overlays (folders in project.overlays)
  const overlaySchemaNames = await project.overlays.currentSchemaDirNames();

  logger.info(`Found ${overlaySchemaNames.length} schemas with overlays`);

  const schemasToPush = [] as {
    schemaName: string;
    schemaId: string;
    schemaVariantId: string;
    funcsToUnbind: {
      id: string;
      kind: string;
      name: string;
    }[];
    funcsToCreate: {
      kind: string;
      data: Action | SimpleFunc;
    }[];
    funcsToUpdate: {
      id: string;
      kind: string;
      data: Action | SimpleFunc;
    }[];
  }[];

  for (const schemaName of overlaySchemaNames) {
    logger.info(`Processing schema: ${schemaName}`);

    // Get schema from remote
    const existingSchema = await tryFindSchema(
      siSchemasApi,
      workspaceId,
      headChangesetId,
      schemaName,
    );

    if (!existingSchema) {
      logger.warn(`  Schema ${schemaName} not found remotely, skipping`);
      continue;
    }

    // Get variant data
    const existingVariant = (await siSchemasApi.getDefaultVariant({
      workspaceId,
      changeSetId: headChangesetId,
      schemaId: existingSchema.schemaId,
    })).data;

    // Check if asset is a builtin
    if (!existingVariant.installedFromUpstream) {
      logger.warn(`  Schema ${schemaName} is not a builtin, skipping`);
      continue;
    }

    // Get existing overlay functions from remote
    const existingOverlayFuncByName = {} as Record<string, {
      id: string;
      data: GetFuncV1Response;
    }>;

    for (const { id: funcId, isOverlay } of existingVariant.variantFuncs) {
      if (!isOverlay) {
        continue;
      }
      const func = (await siFuncsApi.getFunc({
        workspaceId,
        changeSetId: headChangesetId,
        funcId,
      })).data;

      existingOverlayFuncByName[func.name] = {
        id: funcId,
        data: func,
      };
    }

    // Get local overlay functions from filesystem
    const localOverlayActions = await parseActions(
      ctx,
      schemaName,
      project.overlays.funcBasePath(schemaName, FunctionKind.Action),
    );
    const localOverlayAuth = await parseSimpleFuncDirectory(
      ctx,
      schemaName,
      project.overlays.funcBasePath(schemaName, FunctionKind.Auth),
    );
    if (localOverlayAuth.length > 0) {
      logger.warn(
        `${localOverlayAuth.length} auth functions found in overlays, but authentication funcs are not supported. Skipping...`,
      );
    }
    const localOverlayCodegen = await parseSimpleFuncDirectory(
      ctx,
      schemaName,
      project.overlays.funcBasePath(schemaName, FunctionKind.Codegen),
    );
    const localOverlayManagement = await parseSimpleFuncDirectory(
      ctx,
      schemaName,
      project.overlays.funcBasePath(schemaName, FunctionKind.Management),
    );
    const localOverlayQualification = await parseSimpleFuncDirectory(
      ctx,
      schemaName,
      project.overlays.funcBasePath(schemaName, FunctionKind.Qualification),
    );

    const localOverlayFuncs = [
      ...localOverlayActions.map((f) => ({ kind: "Action", funcData: f })),
      ...localOverlayCodegen.map((f) => ({
        kind: "CodeGeneration",
        funcData: f,
      })),
      ...localOverlayManagement.map((f) => ({
        kind: "Management",
        funcData: f,
      })),
      ...localOverlayQualification.map((f) => ({
        kind: "Qualification",
        funcData: f,
      })),
    ];

    // Compare and categorize functions
    const funcsToCreate = [];
    const funcsToUpdate = [];
    const funcsToUnbindByName = {} as Record<
      string,
      { id: string; name: string; kind: string }
    >;

    // Track all remote functions for unbinding
    for (
      const [name, { id, data }] of Object.entries(existingOverlayFuncByName)
    ) {
      funcsToUnbindByName[name] = {
        id,
        name,
        kind: data.kind,
      };
    }

    // Check local functions
    for (const { kind, funcData } of localOverlayFuncs) {
      const funcName = funcData.name;
      const existingFunc = existingOverlayFuncByName[funcName];

      if (existingFunc && existingFunc.data.kind === kind) {
        // Function exists remotely, don't unbind
        delete funcsToUnbindByName[funcName];

        // Check if it needs updating
        if (!funcContentsMatch(funcData, existingFunc.data)) {
          funcsToUpdate.push({
            id: existingFunc.id,
            kind,
            data: funcData,
            existingFuncId: existingFunc.id,
          });
          logger.info(`  ${kind} "${funcName}": will be updated`);
        } else {
          logger.debug(`  ${kind} "${funcName}": unchanged`);
        }
      } else {
        // Function doesn't exist remotely or kind changed
        funcsToCreate.push({
          kind,
          data: funcData,
        });
        logger.info(`  ${kind} "${funcName}": will be created`);
      }
    }

    // Log functions to unbind
    for (const { name, kind } of Object.values(funcsToUnbindByName)) {
      logger.info(`  ${kind} "${name}": will be unbound`);
    }

    logger.info("");
    logger.info(
      `  Summary: ${funcsToCreate.length} to create, ${funcsToUpdate.length} to update, ${
        Object.keys(funcsToUnbindByName).length
      } to unbind`,
    );

    if (
      funcsToCreate.length === 0 && funcsToUpdate.length === 0 &&
      Object.keys(funcsToUnbindByName).length === 0
    ) {
      logger.trace("  No functions to push, skipping...");
      continue;
    }
    schemasToPush.push({
      schemaName,
      schemaId: existingSchema.schemaId,
      schemaVariantId: existingVariant.variantId,
      funcsToUnbind: Object.values(funcsToUnbindByName),
      funcsToCreate,
      funcsToUpdate,
    });
  }

  if (schemasToPush.length === 0) {
    console.log("No assets with overlays to push. terminating.");
    return;
  }

  if (!skipConfirmation) {
    console.log(
      `Found ${schemasToPush.length} schema(s) with overlays to push.`,
    );

    let confirmed = false;
    while (!confirmed) {
      console.log(
        "Do you want to continue? (y = yes, l = list schemas, any other key = cancel)",
      );

      const input = await readRawInput();

      if (input === "l") {
        console.log("\nSchemas with overlays to be pushed:");
        schemasToPush.forEach((schema) =>
          console.log(`  - ${schema.schemaName}`)
        );
        console.log();
      } else if (input === "y") {
        confirmed = true;
      } else {
        return;
      }
    }
  }

  logger.info("creating change set and pushing overlay changes...");

  await wrapInChangeSet(
    changeSetsApi,
    logger,
    workspaceId,
    "SI Overlays Push",
    async (changeSetId) => {
      for (const schema of schemasToPush) {
        logger.debug(`Pushing schema: ${schema.schemaName}`);

        const baseVariantPayload = {
          workspaceId,
          changeSetId,
          schemaId: schema.schemaId,
          schemaVariantId: schema.schemaVariantId,
        };

        for (
          const { name: funcName, id: funcId, kind } of schema.funcsToUnbind
        ) {
          logger.trace(`${kind} "${funcName}": detaching`);

          await unbindFunc(
            siSchemasApi,
            baseVariantPayload,
            kind,
            funcId,
          );
        }

        for (const { kind, data: funcData } of schema.funcsToCreate) {
          const funcName = funcData.name;
          logger.trace(`${kind} "${funcName}": creating`);
          await createFunc(
            siSchemasApi,
            logger,
            baseVariantPayload,
            kind,
            funcData,
            false,
          );
        }

        for (
          const { id: funcId, data: funcData, kind } of schema.funcsToUpdate
        ) {
          const funcName = funcData.name;
          logger.trace(`${kind} "${funcName}": updating`);

          await siFuncsApi.updateFunc({
            workspaceId,
            changeSetId,
            funcId,
            updateFuncV1Request: funcData,
          });
        }
      }

      const changeSetUrl =
        `${workspaceUrlPrefix}/w/${workspaceId}/${changeSetId}/l/a`;

      const pushedSchemasCount = schemasToPush.length;
      ctx.analytics.trackEvent("push_overlays", {
        pushedSchemasCount,
        workspaceId,
        changeSetId,
        changeSetUrl,
      });

      console.log(
        `pushed overlays to ${pushedSchemasCount} schema(s). To see them, go to: ${changeSetUrl}`,
      );
    },
  );
}

interface FuncData {
  name: string;
  displayName?: string | null;
  code: string;
  description?: string | null;
}
const funcContentsMatch = (funcA: FuncData, funcB: FuncData) =>
  funcA.name === funcB.name &&
  funcA.displayName === funcB.displayName &&
  funcA.code === funcB.code &&
  funcA.description === funcB.description;

/// Hides away the ugliness caused by find schema returning a 404 error when the schema doesn't exist
async function tryFindSchema(
  siSchemasApi: SchemasApi,
  workspaceId: string,
  changeSetId: string,
  schemaName: string,
): Promise<FindSchemaV1Response | undefined> {
  try {
    const response = await siSchemasApi.findSchema({
      workspaceId,
      changeSetId,
      schema: schemaName,
    });

    return response.data;
  } catch (error) {
    if (
      typeof error === 'object' &&
      error !== null &&
      'status' in error &&
      error.status === 404
    ) {
      // Swallow error if it's a schema not found error
      return undefined;
    } else {
      throw error;
    }
  }
}

interface SchemaData {
  name: string;
  category: string;
  description?: string | null;
  link?: string | null;
  color: string;
  code: string;
}
const schemaContentsMatch = (schemaA: SchemaData, schemaB: SchemaData) =>
  schemaA.name === schemaB.name &&
  schemaA.category === schemaB.category &&
  // Since the API allows these to be undefined or null, we normalize both to undefined
  (schemaA.description ?? undefined) === (schemaB.description ?? undefined) &&
  (schemaA.link ?? undefined) === (schemaB.link ?? undefined) &&
  schemaA.color === schemaB.color &&
  schemaA.code === schemaB.code;

interface BaseVariantPayload {
  workspaceId: string;
  changeSetId: string;
  schemaId: string;
  schemaVariantId: string;
}

async function createFunc(
  api: SchemasApi,
  logger: Logger,
  baseVariantPayload: BaseVariantPayload,
  kind: string,
  funcData: Action | SimpleFunc,
  updateBuiltins: boolean,
) {
  const payload = {
    ...funcData,
    skipOverlay: updateBuiltins,
  }

  switch (kind) {
    case "Action":
      if (!("kind" in payload)) {
        logger.error(
          `Action must have a 'kind' field, ${payload.name} is missing it`,
        );
        break;
      }

      await api.createVariantAction({
        ...baseVariantPayload,
        createVariantActionFuncV1Request: payload,
      });
      break;
    case "Qualification":
      await api.createVariantQualification({
        ...baseVariantPayload,
        createVariantQualificationFuncV1Request: payload,
      });
      break;
    case "CodeGeneration":
      await api.createVariantCodegen({
        ...baseVariantPayload,
        createVariantCodegenFuncV1Request: payload,
      });
      break;
    case "Management":
      await api.createVariantManagement({
        ...baseVariantPayload,
        createVariantManagementFuncV1Request: payload,
      });
      break;
    case "Authentication":
      await api.createVariantAuthentication({
        ...baseVariantPayload,
        createVariantAuthenticationFuncV1Request: payload,
      });
      break;
    default:
      throw new Error(
        `Unknown func kind for func "${payload.name}": ${kind}`,
      );
  }
}
async function unbindFunc(
  api: SchemasApi,
  baseVariantPayload: BaseVariantPayload,
  kind: string,
  funcId: string,
) {
  const detachPayload = {
    ...baseVariantPayload,
    funcId,
  };

  switch (kind) {
    case "Action":
      await api.detachActionFuncBinding(detachPayload);
      break;
    case "Qualification":
      await api.detachQualificationFuncBinding(detachPayload);
      break;
    case "CodeGeneration":
      await api.detachCodegenFuncBinding(detachPayload);
      break;
    case "Management":
      await api.detachManagementFuncBinding(detachPayload);
      break;
    case "Authentication":
      await api.detachAuthenticationFuncBinding(detachPayload);
      break;
    default:
      throw new Error(`Unknown func kind for func ${funcId}: ${kind}`);
  }
}

// Read input from the first keyboard press. We need to make sure we are no longer in raw mode
// after we capture the input.
async function readRawInput() {
  Deno.stdin.setRaw(true);
  let input: string | undefined;
  try {
    const buf = new Uint8Array(1);
    await Deno.stdin.read(buf);
    input = new TextDecoder().decode(buf).toLowerCase();
    console.log(input);
  } finally {
    Deno.stdin.setRaw(false);
  }
  return input;
}
