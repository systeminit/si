import {
  ChangeSetsApi,
  FindSchemaV1Response,
  FuncsApi,
  GetFuncV1Response,
  SchemasApi,
} from "@systeminit/api-client";
import { AxiosError } from "axios";
import { Context } from "../../../context.ts";
import { AuthenticatedCliContext } from "../../../cli/helpers.ts";
import { unknownValueToErrorMessage } from "../../../helpers.ts";
import { SCHEMA_FILE_FORMAT_VERSION } from "../../../config.ts";
import {
  AbsoluteDirectoryPath,
  normalizeFsName,
  Project,
} from "../../../project.ts";

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
  skipConfirmation?: boolean,
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
  let readSchemas = 0; // This is the total number of read schema directories, including failures
  try {
    const schemasBasePath = project.schemasBasePath().toString();

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

      const versionFilePath = project.schemaFormatVersionPath(schemaDirName);

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

      const schemaCodePath = project.schemaFuncCodePath(schemaDirName);
      try {
        const code = await schemaCodePath.readTextFile();

        let schemaName: string;
        let category: string;
        let description: string;
        let link: string;

        const schemaMetadataPath = project.schemaMetadataPath(schemaDirName);
        try {
          const metadataContent = await schemaMetadataPath.readTextFile();
          const metadata = JSON.parse(metadataContent);

          schemaName = metadata.name;

          if (!schemaName || schemaName.trim() === "") {
            throw new Error("Missing required 'name' field in metadata.json");
          }

          category = metadata.category || "";
          description = metadata.description || "";
          link = metadata.documentation ?? null;
        } catch (error) {
          const msg = error instanceof Deno.errors.NotFound
            ? `metadata.json file not found on the ${schemaDirName} directory`
            : unknownValueToErrorMessage(error);
          logger.error(msg);
          continue;
        }

        if (existingLocalSchemaNames[schemaName]) {
          throw new Error(
            `Duplicate schema name "${schemaName}" found in asset "${schemaDirName}" and "${
              existingLocalSchemaNames[schemaName]
            }".`,
          );
        }
        existingLocalSchemaNames[schemaName] = schemaDirName;
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
          project.qualificationBasePath(schemaDirName),
        );

        logger.debug(
          `loaded ${qualifications.length} qualifications for ${schemaName}`,
        );

        const actions = await parseActions(
          ctx,
          schemaName,
          project.actionBasePath(schemaDirName),
        );

        logger.debug(`loaded ${actions.length} actions for ${schemaName}`);

        const codeGenerators = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.codegenBasePath(schemaDirName),
        );

        logger.debug(
          `loaded ${codeGenerators.length} code generators for ${schemaName}`,
        );

        const managementFuncs = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.managementBasePath(schemaDirName),
        );

        logger.debug(
          `loaded ${managementFuncs.length} management funcs for ${schemaName}`,
        );

        const authFuncs = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.authBasePath(schemaDirName),
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
  const failedSchemaDirectories = readSchemas - schemasFromFilesystem.length;

  // ==================================
  // Pre check: Compare schema and funcs to head
  // ==================================
  const changeSets = (await changeSetsApi.listChangeSets({ workspaceId }))
    .data.changeSets as { id: string; isHead: boolean }[]; // Mocking this type since the client lib does not have it right now

  const headChangesetId = changeSets.find((cs) => cs.isHead)?.id;

  if (!headChangesetId) {
    throw new Error("No head changeset found");
  }

  logger.info(`comparing filesystem assets to HEAD changeset...`);

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
    for (let { id: existingFuncId } of existingVariant.variantFuncs) {
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

    console.log(
      `Found ${schemasToPush.length} asset(s) to push.${failureMsg}`,
    );

    let confirmed = false;
    while (!confirmed) {
      console.log(
        "Do you want to continue? (y = yes, l = list assets, any other key = cancel)",
      );

      const buf = new Uint8Array(1024);
      const n = await Deno.stdin.read(buf);
      const line = new TextDecoder().decode(buf.subarray(0, n ?? 0)).trim();
      const input = line.charAt(0).toLowerCase();

      if (input === "l") {
        console.log("\nAssets to be pushed:");
        schemasToPush.forEach((schema) => console.log(`  - ${schema.name}`));
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
  logger.info("creating changeset and pushing schema changes...");
  const changeSetName = "Conduit Push " + new Date().toISOString();

  const createChangeSetResponse = await changeSetsApi.createChangeSet({
    workspaceId,
    createChangeSetV1Request: { changeSetName },
  });

  const changeSetId = createChangeSetResponse.data.changeSet.id;

  let pushedSchemas = 0;
  try {
    for (const schemaData of schemasToPush) {
      const { code, name, category, description, link } =
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
            color: "#000000",
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

      for (const { id: funcId, kind, name } of funcsToUnbind) {
        logger.trace(`${kind} "${name}": detaching`);

        const detachPayload = {
          ...baseVariantPayload,
          funcId,
        };

        switch (kind) {
          case "Action":
            await siSchemasApi.detachActionFuncBinding(detachPayload);
            break;
          case "Qualification":
            await siSchemasApi.detachQualificationFuncBinding(detachPayload);
            break;
          case "CodeGeneration":
            await siSchemasApi.detachCodegenFuncBinding(detachPayload);
            break;
          case "Management":
            await siSchemasApi.detachManagementFuncBinding(detachPayload);
            break;
          case "Authentication":
            await siSchemasApi.detachAuthenticationFuncBinding(detachPayload);
            break;
          default:
            console.error(`Unknown func kind for func ${funcId}: ${kind}`);
        }
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

          switch (filesystemFuncKind) {
            case "Action":
              if (!("kind" in filesystemFunc)) {
                logger.error(
                  `Action must have a 'kind' field, ${filesystemFunc.name} is missing it`,
                );
                break;
              }

              await siSchemasApi.createVariantAction({
                ...baseVariantPayload,
                createVariantActionFuncV1Request: filesystemFunc,
              });
              break;
            case "Qualification":
              await siSchemasApi.createVariantQualification({
                ...baseVariantPayload,
                createVariantQualificationFuncV1Request: filesystemFunc,
              });
              break;
            case "CodeGeneration":
              await siSchemasApi.createVariantCodegen({
                ...baseVariantPayload,
                createVariantCodegenFuncV1Request: filesystemFunc,
              });
              break;
            case "Management":
              await siSchemasApi.createVariantManagement({
                ...baseVariantPayload,
                createVariantManagementFuncV1Request: filesystemFunc,
              });
              break;
            case "Authentication":
              await siSchemasApi.createVariantAuthentication({
                ...baseVariantPayload,
                createVariantAuthenticationFuncV1Request: filesystemFunc,
              });
              break;
            default:
              logger.error(
                `Unknown func kind for func "${filesystemFunc.name}": ${filesystemFuncKind}`,
              );
          }
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
  } catch (error) {
    if (error instanceof AxiosError) {
      logger.error(
        `API error creating schemas: (${error.status}) ${error.response?.data.message}`,
      );
      logger.error(`Request: ${error.request.method} ${error.request.path}`);
    } else {
      logger.error(
        `Error creating schemas: ${unknownValueToErrorMessage(error)}`,
      );
    }
    logger.info("Deleting changeset...");
    changeSetsApi.abandonChangeSet({
      workspaceId,
      changeSetId,
    });
  }
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
    // Swallow error if it's a schema not found error
    if (!(error instanceof AxiosError)) {
      throw error;
    }

    // TypeScript does not believe the typecheck above
    const axiosError = error as AxiosError;

    if (axiosError.status !== 404) {
      throw error;
    }
  }

  // If we got here, the error is an axios 404
  return undefined;
}

interface SchemaData {
  name: string;
  category: string;
  description?: string | null;
  link?: string | null;
  code: string;
}
const schemaContentsMatch = (schemaA: SchemaData, schemaB: SchemaData) =>
  schemaA.name === schemaB.name &&
  schemaA.category === schemaB.category &&
  // Since the API allows these to be undefined or null, we normalize both to undefined
  (schemaA.description ?? undefined) === (schemaB.description ?? undefined) &&
  (schemaA.link ?? undefined) === (schemaB.link ?? undefined) &&
  schemaA.code === schemaB.code;
