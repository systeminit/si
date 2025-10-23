import { ChangeSetsApi, SchemasApi, FuncsApi } from "@systeminit/api-client";
import { AuthenticatedCliContext } from "./helpers.ts";
import { AxiosError } from "axios";
import { Context } from "../context.ts";
import { unknownValueToErrorMessage } from "../helpers.ts";
import { SCHEMA_FILE_FORMAT_VERSION } from "../config.ts";
import { AbsoluteDirectoryPath, Project } from "../project.ts";

async function parseActions(
  ctx: Context,
  schemaName: string,
  funcBasePath: AbsoluteDirectoryPath,
) {
  const logger = ctx.logger;
  const validActionKinds = ["create", "destroy", "refresh", "update"];
  const actionFiles = [];

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
    }))

  return [
    ...addKind("Qualification", schema.qualifications),
    ...addKind("Action", schema.actions),
    ...addKind("CodeGeneration", schema.codeGenerators),
    ...addKind("Management", schema.managementFuncs),
    ...addKind("Authentication", schema.authFuncs),
  ];
}

export async function pushAssets(
  cliContext: AuthenticatedCliContext,
  project: Project,
  skipConfirmation?: boolean,
) {
  const { apiConfiguration, workspace, ctx } = cliContext;
  const logger = ctx.logger;

  const { instanceUrl: workspaceUrlPrefix, id: workspaceId } = workspace;

  const schemas = [] as Schema[];

  let readSchemas = 0; // This is the total number of read schema directories, including failures
  // Read schemas from the local folder
  try {
    const schemasBasePath = project.schemasBasePath().toString();

    const entries = Deno.readDir(schemasBasePath);

    for await (const entry of entries) {
      if (!entry.isDirectory) {
        continue;
      }

      readSchemas += 1;

      const schemaDirName = entry.name;

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
          link = metadata.link;
        } catch (error) {
          const msg = error instanceof Deno.errors.NotFound
            ? `metadata.json file not found on the ${schemaDirName} directory`
            : unknownValueToErrorMessage(error);
          logger.error(msg);
          continue;
        }

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
          project.qualificationBasePath(schemaName),
        );

        const actions = await parseActions(
          ctx,
          schemaName,
          project.actionBasePath(schemaName),
        );

        const codeGenerators = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.codegenBasePath(schemaName),
        );

        const managementFuncs = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.managementBasePath(schemaName),
        );

        const authFuncs = await parseSimpleFuncDirectory(
          ctx,
          schemaName,
          project.authBasePath(schemaName),
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

        const existingNames = {} as Record<string, string>;
        allSchemaFuncsWithKind(schema).forEach((func) => {
          const { name: funcName } = func.funcData;
          const { kind } = func;
          if (existingNames[funcName]) {
            throw new Error(
              `Duplicate function name "${funcName}" found in asset "${schemaName}": ${existingNames[funcName]} and ${kind}`,
            );
          }
          existingNames[funcName] = kind;
        })

        schemas.push(schema);
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

  const failedSchemaDirectories = readSchemas - schemas.length;

  // Confirmation prompt
  if (!skipConfirmation) {
    const failureMsg = failedSchemaDirectories > 0
      ? ` Failed to read ${failedSchemaDirectories} asset description(s).`
      : "";

    const emptyMsg = schemas.length === 0 ? " Aborting." : "";

    console.log(
      `Found ${schemas.length} asset(s) to push.${failureMsg}${emptyMsg}`,
    );

    if (schemas.length === 0) {
      return;
    }

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
        schemas.forEach((schema) => console.log(`  - ${schema.name}`));
        console.log();
      } else if (input === "y") {
        confirmed = true;
      } else {
        return;
      }
    }
  }

  // TODO We should read the schema from head and see what we need to do before creating the changeset, so we can avoid changesets with no real diffs
  // Create schemas on the server
  const changeSetsApi = new ChangeSetsApi(apiConfiguration);

  const changeSetName = "Conduit Push " + new Date().toISOString();

  const createChangeSetResponse = await changeSetsApi.createChangeSet({
    workspaceId,
    createChangeSetV1Request: { changeSetName },
  });

  const changeSetId = createChangeSetResponse.data.changeSet.id;

  let pushedSchemas = 0;
  try {
    const siSchemasApi = new SchemasApi(apiConfiguration);
    const siFuncsApi = new FuncsApi(apiConfiguration);

    for (const schema of schemas) {
      const { code, name, category, description } = schema;

      let existingSchemaId = undefined;
      try {
        const response = await siSchemasApi.findSchema({
          workspaceId,
          changeSetId,
          schema: name,
        });

        existingSchemaId = response.data.schemaId;
      } catch (error) {
        // Swallow error if it's a schema not found error, as we will create the schema
        if (!(error instanceof AxiosError)) {
          throw error;
        }

        // TypeScript does not believe the typecheck above
        const axiosError = error as AxiosError;

        if (axiosError.status !== 404) {
          throw error;
        }
      }

      let schemaId;
      let schemaVariantId;

      // These two records will only be filled out if the schema already exists
      // But we create it here to create a single code path when dealing with funcs
      const funcsToUnbindById = {} as Record<string, {
        kind: string,
        name: string,
      }>;
      const existingFuncDataByName = {} as Record<string, {
        id: string,
        name: string,
        displayName?: string | null,
        description?: string | null,
        code: string,
        funcKind: string,
      }>;

      if (existingSchemaId) {
        logger.info(
          `existing schema ${name} (${existingSchemaId}), unlocking and updating...`,
        );

        const unlockSchemaResponse = await siSchemasApi.unlockSchema({
          workspaceId,
          changeSetId,
          schemaId: existingSchemaId,
        });

        schemaId = existingSchemaId;
        schemaVariantId = unlockSchemaResponse.data.unlockedVariantId;

        // TODO add optional fields (link)
        const unlockedVariantData = (await siSchemasApi.updateSchemaVariant({
          workspaceId,
          changeSetId,
          schemaId: existingSchemaId,
          schemaVariantId,
          updateSchemaVariantV1Request: {
            name,
            code,
            category,
            description,
          },
        })).data;

        // Gather all the funcs that are currently bound to this schema
        logger.debug(`gathering funcs...`);
        for (const existingFunc of unlockedVariantData.variantFuncs) {
          const funcRequest = await siFuncsApi.getFunc({
            workspaceId,
            changeSetId,
            funcId: existingFunc.id,
          });

          const loadedFuncData = funcRequest.data;
          funcsToUnbindById[existingFunc.id] = {
            kind: loadedFuncData.kind,
            name: loadedFuncData.name,
          };

          existingFuncDataByName[loadedFuncData.name] = {
            id: existingFunc.id,
            name: loadedFuncData.name,
            displayName: loadedFuncData.displayName,
            description: loadedFuncData.description,
            code: loadedFuncData.code,
            funcKind: existingFunc.funcKind.kind,
          };
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


      let createdFuncs = 0;
      let modifiedFuncs = 0;
      let skippedFuncs = 0;

      for (const filesystemFuncAndKind of allSchemaFuncsWithKind(schema)) {
        const filesystemFunc = filesystemFuncAndKind.funcData;
        const filesystemFuncKind = filesystemFuncAndKind.kind;

        const existingFunc = existingFuncDataByName[filesystemFunc.name];
        if (existingFunc) {
          logger.debug(`found ${filesystemFuncKind} ${filesystemFunc.name}...`);
          delete funcsToUnbindById[existingFunc.id];

          if (funcContentsMatch(existingFunc, filesystemFunc)) {
            logger.debug(`${existingFunc.name} is unchanged. Skipping...`);
            skippedFuncs += 1;
          } else {
            logger.debug(`${existingFunc.name} is changed. Unlocking and updating...`);
            modifiedFuncs += 1;

            const unlockFuncResult = await siFuncsApi.unlockFunc({
              workspaceId,
              changeSetId,
              funcId: existingFunc.id,
              unlockFuncV1Request: {
                schemaVariantId
              }
            })

            const unlockedFuncId = unlockFuncResult.data.unlockedFuncId;

            await siFuncsApi.updateFunc({
              workspaceId,
              changeSetId,
              funcId: unlockedFuncId,
              updateFuncV1Request: filesystemFunc,
            });
          }
        } else {
          logger.debug(`${filesystemFunc.name} is new. Creating...`);
          createdFuncs += 1;

          switch (filesystemFuncKind) {
            case "Action":
              if (!("kind" in filesystemFunc)) {
                logger.error(`Action must have a 'kind' field, ${filesystemFunc.name} is missing it`);
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
              logger.error(`Unknown func kind for func "${filesystemFunc.name}": ${filesystemFuncKind}`);
          }
        }
      }

      let detachedFuncs = 0;

      // TODO unbinding should happen before creating and updating any funcs, so we don't get conflicts with existing actions
      // loop over funcIdsToUnbind, using both the keys and the values
      for (const [funcId, { kind, name }] of Object.entries(funcsToUnbindById)) {
        logger.debug(`${name} was removed. Detaching...`);
        detachedFuncs += 1;

        const detachPayload = {
          ...baseVariantPayload,
          funcId,
        }

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
            console.error(`Unknown unknown func kind for func ${funcId}: ${kind}`);
        }
      }

      logger.info(`func summary: ${createdFuncs} created, ${modifiedFuncs} modified, ${detachedFuncs} detached, ${skippedFuncs} unchanged`);

      pushedSchemas += 1;

    }

    const changeSetUrl =
      `${workspaceUrlPrefix}/w/${workspaceId}/${changeSetId}/l/a`;

    ctx.analytics.trackEvent("push_assets", {
      pushedSchemasCount: pushedSchemas,
      pushedSchemaNames: schemas.map((schema) => schema.name),
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
