import { ChangeSetsApi, SchemasApi } from "@systeminit/api-client";
import { CliContext } from "../cli.ts";
import { AxiosError } from "npm:axios@1.11.0";
import { Log } from "../log.ts";
import { unknownValueToErrorMessage } from "../helpers.ts";

export async function exportAssets(context: CliContext, assetsPath: string, skipConfirmation?: boolean) {
  const { apiConfiguration, log, workspace } = context;

  const {
    instanceUrl: workspaceUrlPrefix,
    id: workspaceId,
  } = workspace;

  const schemas = [] as {name: string, code: string, category: string, description: string, link?: string}[]

  // Read schemas from the local folder
  try {
    const entries = Deno.readDirSync(assetsPath);

    for (const entry of entries) {
      if (!entry.isDirectory) {
        continue;
      }

      const assetName = entry.name;
      const indexPath = `${assetsPath}/${assetName}/index.ts`;

      try {
        const code = await Deno.readTextFile(indexPath);

        let category = "";
        let description = "";
        let link = "";

        const metadataPath = `${assetsPath}/${assetName}/metadata.json`;
        try {
          const metadataContent = await Deno.readTextFile(metadataPath);
          const metadata = JSON.parse(metadataContent);
          category = metadata.category || "";
          description = metadata.description || "";
          link = metadata.link;
        } catch {
          // If metadata.json doesn't exist or can't be read, use empty strings
        }

        if (category === "") {
          // Try to get the category from the asset name
          const categoryMatch = assetName.match(/^([^\s:]+::[^\s:]+)::(.+)$/);
          if (categoryMatch && categoryMatch[1] !== undefined) {
            category = categoryMatch[1];
          }
        }

        schemas.push({name: assetName, code, category, description, link})
      } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
          log.error(`No index.ts file found for asset "${assetName}", skipping...\n`);
          continue;
        }
        throw error;
      }
    }
  } catch (error) {
    log.error(`Error reading assets directory: ${unknownValueToErrorMessage(error)}`);
    Deno.exit(1);
  }

  // Confirmation prompt
  if (!skipConfirmation) {
    console.log(`Found ${schemas.length} asset(s) to import.`);

    let confirmed = false;
    while (!confirmed) {
      console.log("Do you want to continue? (y = yes, l = list assets, any other key = cancel)");

      const buf = new Uint8Array(1024);
      const n = await Deno.stdin.read(buf);
      const line = new TextDecoder().decode(buf.subarray(0, n ?? 0)).trim();
      const input = line.charAt(0).toLowerCase();

      if (input === 'l') {
        console.log("\nAssets to be imported:");
        schemas.forEach(schema => console.log(`  - ${schema.name}`));
        console.log();
      } else if (input === 'y') {
        confirmed = true;
      } else {
        Deno.exit(0);
      }
    }
  }

  // Create schemas on the server
  const changeSetsApi = new ChangeSetsApi(apiConfiguration);

  const changeSetName = "Import Assets " + new Date().toISOString();

  const createChangeSetResponse = await changeSetsApi.createChangeSet({
    workspaceId,
    createChangeSetV1Request: { changeSetName },
  });

  const changeSetId = createChangeSetResponse.data.changeSet.id;
  if (!changeSetId) {
    throw new Error("Error creating changeset");
  }


  let importedSchemas = 0;
  try {
    const siSchemasApi = new SchemasApi(apiConfiguration);
    // console.log("SchemasApi methods:", Object.getOwnPropertyNames(Object.getPrototypeOf(siSchemasApi)));

    for (let schema of schemas) {
      const {code, name, category, description} = schema;

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
        log.info(`existing schema ${name} (${existingSchemaId}), unlocking and updating...`);

        const unlockSchemaResponse = await siSchemasApi.unlockSchema({
          workspaceId,
          changeSetId,
          schemaId: existingSchemaId,
        })

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
        })

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
      importedSchemas += 1;

      // Create qualifications
      const qualifications = await parseQualifications(`${assetsPath}/${name}/qualifications`, name, log);

      for (let qualification of qualifications) {
        await siSchemasApi.createVariantQualification({
          workspaceId,
          changeSetId,
          schemaId,
          schemaVariantId: variantId,
          createVariantQualificationFuncV1Request: qualification,
        })
      }

      // Create action funcs from actions folder
      const actions = await parseActions(`${assetsPath}/${name}/actions`, name, log);

      for (const action of actions) {
        // TODO check if the actions already exist

        await siSchemasApi.createVariantAction({
          workspaceId,
          changeSetId,
          schemaId,
          schemaVariantId: variantId,
          createVariantActionFuncV1Request: action,
        });
      }
    }

    const changeSetUrl = `${workspaceUrlPrefix}/w/${workspaceId}/${changeSetId}/l/a`;
    console.log(`${importedSchemas} schemas imported. To see them, go to: ${changeSetUrl}`);
  } catch (error) {
    if (error instanceof AxiosError) {
      log.error(`API error creating schemas: (${error.status}) ${error.response?.data.message}`);
      log.error(`Request: ${error.request.method} ${error.request.path}`);
    } else {
      log.error(`Error creating schemas: ${unknownValueToErrorMessage(error)}`);
    }
    log.info("Deleting changeset...");
    changeSetsApi.abandonChangeSet({
      workspaceId: workspaceId,
      changeSetId,
    });
  }
}

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
      })
    }
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw new Error(`Error reading actions directory for asset "${assetName}": ${unknownValueToErrorMessage(error)}`);
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
        log.error(`Empty code in action file "${actionTsPath}" for asset "${assetName}", skipping...`);
        continue;
      }

      // Read the matching JSON file
      let actionJsonContent;
      try {
        actionJsonContent = await Deno.readTextFile(actionJsonPath);
      } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
          log.error(`No matching .json file found for action "${actionTsPath}" in asset "${assetName}", skipping...`);
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
          log.error(`Missing required 'name' field in "${actionJsonPath}" for non manual asset "${assetName}", skipping...`);
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
      log.error(`Error processing action "${strippedFileName}" for asset "${assetName}": ${unknownValueToErrorMessage(error)}, skipping...`);
    }
  }

  return actions;
}

async function parseQualifications(qualificationsPath: string, assetName: string, log: Log) {
  const qualificationFiles = [];

  // Read the file list from the actions folder
  try {
    const qualificationEntries = Deno.readDirSync(qualificationsPath);

    for (const qualificationEntry of qualificationEntries) {
      if (!qualificationEntry.isFile || !qualificationEntry.name.endsWith(".ts")) {
        continue;
      }

      const actionFileName = qualificationEntry.name;
      const strippedFileName = actionFileName.replace(/\.ts$/, "");

      qualificationFiles.push({
        strippedFileName,
      })
    }
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw new Error(`Error reading actions directory for asset "${assetName}": ${unknownValueToErrorMessage(error)}`);
    }
    // If actions folder doesn't exist, that's ok - just continue
    return [];
  }

  const qualifications = [];

  // For each listed file, get the contents and the optional metadata file ({kind}.json)
  for (const { strippedFileName } of qualificationFiles) {
    const qualificationTsPath = `${qualificationsPath}/${strippedFileName}.ts`;
    const qualificationJsonPath = `${qualificationsPath}/${strippedFileName}.json`;

    try {
      // Read the TypeScript file
      const code = await Deno.readTextFile(qualificationTsPath);

      if (!code || code.trim() === "") {
        log.error(`Empty code in qualification file "${qualificationTsPath}" for asset "${assetName}", skipping...`);
        continue;
      }

      // Read the matching JSON file
      let qualificationMetadata = {};
      try {
        const qualificationJsonContent = await Deno.readTextFile(qualificationJsonPath);
        qualificationMetadata = JSON.parse(qualificationJsonContent)
      } catch (error) {
        // Not finding the metadata file is ok - just continue
        if (!(error instanceof Deno.errors.NotFound)) throw error;
      }

      let name = qualificationMetadata.name ?? strippedFileName;

      // Build action object
      const qualificationObject = {
        name,
        displayName: qualificationMetadata.displayName ?? null,
        description: qualificationMetadata.description ?? null,
        code: code,
      };

      qualifications.push(qualificationObject);
    } catch (error) {
      log.error(`Error processing qualification "${strippedFileName}" for asset "${assetName}": ${unknownValueToErrorMessage(error)}, skipping...`);
    }
  }

  return qualifications;
}
