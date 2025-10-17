import { ChangeSetsApi, SchemasApi } from "@systeminit/api-client";
import { AuthenticatedCliContext } from "../cli.ts";
import { AxiosError } from "axios";
import { Log } from "../log.ts";
import { unknownValueToErrorMessage } from "../helpers.ts";
import { SCHEMA_FILE_FORMAT_VERSION } from "../config.ts";

async function parseActions(actionsPath: string, assetName: string, log: Log) {
  const validActionKinds = ["create", "destroy", "refresh", "update"];
  const actionFiles = [];

  // Read the file list from the actions folder
  try {
    const actionEntries = Deno.readDirSync(actionsPath);

    for (const actionEntry of actionEntries) {
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
        `Error reading actions directory for asset "${assetName}": ${
          unknownValueToErrorMessage(error)
        }`,
      );
    }
    // If actions folder doesn't exist, that's ok - just continue
    return [];
  }

  const actions = [];

  // For each listed file, get the contents and the required metadata file ({kind}.json)
  for (const { kind, strippedFileName } of actionFiles) {
    const actionTsPath = `${actionsPath}/${strippedFileName}.ts`;
    const actionJsonPath = `${actionsPath}/${strippedFileName}.json`;

    try {
      // Read the TypeScript file
      const code = await Deno.readTextFile(actionTsPath);

      if (!code || code.trim() === "") {
        log.error(
          `Empty code in action file "${actionTsPath}" for asset "${assetName}", skipping...`,
        );
        continue;
      }

      // Read the matching JSON file
      let actionJsonContent;
      try {
        actionJsonContent = await Deno.readTextFile(actionJsonPath);
      } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
          log.error(
            `No matching .json file found for action "${actionTsPath}" in asset "${assetName}", skipping...`,
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
          log.error(
            `Missing required 'name' field in "${actionJsonPath}" for non manual asset "${assetName}", skipping...`,
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
      log.error(
        `Error processing action "${strippedFileName}" for asset "${assetName}": ${
          unknownValueToErrorMessage(error)
        }, skipping...`,
      );
    }
  }

  return actions;
}

type ActionArray = Awaited<ReturnType<typeof parseActions>>;

async function parseSimpleFuncDirectory(
  directoryPath: string,
  assetName: string,
  log: Log,
) {
  const files = [] as string[];

  // Read the file list from the actions folder
  try {
    const entries = Deno.readDirSync(directoryPath);

    for (const entry of entries) {
      if (
        !entry.isFile || !entry.name.endsWith(".ts")
      ) {
        continue;
      }

      const fileName = entry.name;
      const strippedFileName = fileName.replace(/\.ts$/, "");

      files.push(strippedFileName);
    }
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw new Error(
        `Error reading funcs directory "${directoryPath}" for asset "${assetName}": ${
          unknownValueToErrorMessage(error)
        }`,
      );
    }
    // If folder doesn't exist, skip by returning early
    return [];
  }

  const funcs = [];

  for (const strippedFileName of files) {
    const funcTsPath = `${directoryPath}/${strippedFileName}.ts`;
    const funcJsonPath = `${directoryPath}/${strippedFileName}.json`;

    try {
      // Read the TypeScript file
      const code = await Deno.readTextFile(funcTsPath);

      if (!code || code.trim() === "") {
        log.error(
          `Empty code in file "${funcTsPath}" for asset "${assetName}", skipping...`,
        );
        continue;
      }

      // Read the matching JSON file
      let metadata = {} as Record<string, string>;
      try {
        const jsonContent = await Deno.readTextFile(
          funcJsonPath,
        );
        metadata = JSON.parse(jsonContent);
      } catch (error) {
        // Not finding the metadata file is ok - just continue
        if (!(error instanceof Deno.errors.NotFound)) throw error;
      }

      let name = metadata.name ?? strippedFileName;

      // Build action object
      const funcObject = {
        name,
        displayName: metadata.displayName ?? name,
        description: metadata.description ?? null,
        code: code,
      };

      funcs.push(funcObject);
    } catch (error) {
      log.error(
        `Error processing "${strippedFileName}" for asset "${assetName}": ${
          unknownValueToErrorMessage(error)
        }, skipping...`,
      );
    }
  }

  return funcs;
}

type SimpleFuncArray = Awaited<ReturnType<typeof parseSimpleFuncDirectory>>;

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
}

export async function pushAssets(
  context: AuthenticatedCliContext,
  assetsPath: string,
  skipConfirmation?: boolean,
) {
  const { apiConfiguration, log, workspace, analytics } = context;

  const {
    instanceUrl: workspaceUrlPrefix,
    id: workspaceId,
  } = workspace;

  const schemas = [] as Schema[];

  let readSchemas = 0; // This is the total number of read schema directories, including failures
  // Read schemas from the local folder
  try {
    const entries = Deno.readDirSync(assetsPath);

    for (const entry of entries) {
      if (!entry.isDirectory) {
        continue;
      }

      readSchemas += 1;

      const assetName = entry.name;

      const versionFilePath = `${assetsPath}/${assetName}/.format-version`;

      let thisFormatVersion: number;
      try {
        const formatContent = await Deno.readTextFile(versionFilePath);
        thisFormatVersion = parseInt(formatContent);
      } catch (error) {
        const msg = error instanceof Deno.errors.NotFound
          ? `.format-version file not found on the ${assetName} directory`
          : unknownValueToErrorMessage(error);
        log.error(msg);
        continue;
      }

      if (
        isNaN(thisFormatVersion) ||
        thisFormatVersion !== SCHEMA_FILE_FORMAT_VERSION
      ) {
        let msg =
          `Unsupported format version ${thisFormatVersion} on the asset ${assetName}. ` +
          `Supported version is ${SCHEMA_FILE_FORMAT_VERSION}. ` +
          `You can run a new scaffold command and port your schema over to push it with this executable`;

        log.error(msg);
        continue;
      }

      const indexPath = `${assetsPath}/${assetName}/index.ts`;
      try {
        const code = await Deno.readTextFile(indexPath);

        let name: string;
        let category: string;
        let description: string;
        let link: string;

        const metadataPath = `${assetsPath}/${assetName}/metadata.json`;
        try {
          const metadataContent = await Deno.readTextFile(metadataPath);
          const metadata = JSON.parse(metadataContent);

          name = metadata.name;

          if (!name || name.trim() === "") {
            throw new Error("Missing required 'name' field in metadata.json");
          }

          category = metadata.category || "";
          description = metadata.description || "";
          link = metadata.link;
        } catch (error) {
          const msg = error instanceof Deno.errors.NotFound
            ? `metadata.json file not found on the ${assetName} directory`
            : unknownValueToErrorMessage(error);
          log.error(msg);
          continue;
        }

        if (category === "") {
          // Try to get the category from the asset name
          const categoryMatch = assetName.match(/^([^\s:]+::[^\s:]+)::(.+)$/);
          if (categoryMatch && categoryMatch[1] !== undefined) {
            category = categoryMatch[1];
          }
        }

        const qualifications = await parseSimpleFuncDirectory(
          `${assetsPath}/${assetName}/qualifications`,
          assetName,
          log,
        );

        const actions = await parseActions(
          `${assetsPath}/${assetName}/actions`,
          assetName,
          log,
        );

        const codeGenerators = await parseSimpleFuncDirectory(
          `${assetsPath}/${assetName}/codeGenerators`,
          assetName,
          log,
        );

        const managementFuncs = await parseSimpleFuncDirectory(
          `${assetsPath}/${assetName}/management`,
          assetName,
          log,
        );

        schemas.push({
          name,
          code,
          category,
          description,
          link,
          qualifications,
          actions,
          codeGenerators,
          managementFuncs,
        });
      } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
          log.error(
            `No index.ts file found for asset "${assetName}", skipping...\n`,
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

  // Create schemas on the server
  const changeSetsApi = new ChangeSetsApi(apiConfiguration);

  const changeSetName = "Conduit Push " + new Date().toISOString();

  const createChangeSetResponse = await changeSetsApi.createChangeSet({
    workspaceId,
    createChangeSetV1Request: { changeSetName },
  });

  const changeSetId = createChangeSetResponse.data.changeSet.id;
  if (!changeSetId) {
    throw new Error("Error creating changeset");
  }

  let pushedSchemas = 0;
  try {
    const siSchemasApi = new SchemasApi(apiConfiguration);

    for (let schema of schemas) {
      const { code, name, category, description } = schema;

      let existingSchemaId = undefined;
      let existingSchemaIsInstalled = undefined;
      try {
        const response = await siSchemasApi.findSchema({
          workspaceId,
          changeSetId,
          schema: name,
        });

        existingSchemaId = response.data.schemaId;
        existingSchemaIsInstalled = response.data.installed;
      } catch (error) {
        // Swallow error if it's a schema not found error, as we will create the schema
        if (!(error instanceof AxiosError) || error.status !== 404) {
          throw error;
        }
      }

      let schemaId;
      let variantId;
      if (existingSchemaId) {
        log.info(
          `existing schema ${name} (${existingSchemaId}), unlocking and updating...`,
        );

        const unlockSchemaResponse = await siSchemasApi.unlockSchema({
          workspaceId,
          changeSetId,
          schemaId: existingSchemaId,
        });

        const schemaVariantId = unlockSchemaResponse.data.unlockedVariantId;

        // TODO add optional fields (link)
        await siSchemasApi.updateSchemaVariant({
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
        });

        schemaId = existingSchemaId;
        variantId = schemaVariantId;
      } else {
        log.info(`creating schema ${name}...`);

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
        variantId = createSchemaResponse.data.defaultVariantId;
      }
      pushedSchemas += 1;

      // Create qualifications
      for (let qualification of schema.qualifications) {
        await siSchemasApi.createVariantQualification({
          workspaceId,
          changeSetId,
          schemaId,
          schemaVariantId: variantId,
          createVariantQualificationFuncV1Request: qualification,
        });
      }

      for (const action of schema.actions) {
        await siSchemasApi.createVariantAction({
          workspaceId,
          changeSetId,
          schemaId,
          schemaVariantId: variantId,
          createVariantActionFuncV1Request: action,
        });
      }

      for (const func of schema.codeGenerators) {
        await siSchemasApi.createVariantCodegen({
          workspaceId,
          changeSetId,
          schemaId,
          schemaVariantId: variantId,
          createVariantCodegenFuncV1Request: func,
        });
      }

      for (const func of schema.managementFuncs) {
        await siSchemasApi.createVariantManagement({
          workspaceId,
          changeSetId,
          schemaId,
          schemaVariantId: variantId,
          createVariantManagementFuncV1Request: func,
        });
      }
    }

    const changeSetUrl =
      `${workspaceUrlPrefix}/w/${workspaceId}/${changeSetId}/l/a`;

    analytics.trackEvent("push_assets", {
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
      log.error(
        `API error creating schemas: (${error.status}) ${error.response?.data.message}`,
      );
      log.error(`Request: ${error.request.method} ${error.request.path}`);
    } else {
      log.error(`Error creating schemas: ${unknownValueToErrorMessage(error)}`);
    }
    log.info("Deleting changeset...");
    changeSetsApi.abandonChangeSet({
      workspaceId,
      changeSetId,
    });
  }
}
