import { ChangeSetsApi, SchemasApi } from "@systeminit/api-client";
import { CliContext } from "../cli.ts";
import { AxiosError } from "npm:axios@1.11.0";
import { Log } from "../log.ts";

export async function exportAssets(context: CliContext, assetsPath?: string, skipConfirmation?: boolean) {
  const { log, authApi } = context;

  const { instanceUrl: workspaceUrlPrefix } = await authApi.getWorkspaceDetails();

  assetsPath = assetsPath || new URL("../../assets", import.meta.url).pathname;

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
    log.error(`Error reading assets directory: ${error.message}`);
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
      const line = new TextDecoder().decode(buf.subarray(0, n)).trim();
      const input = line.charAt(0).toLowerCase();

      if (input === 'l') {
        console.log("\nAssets to be imported:");
        schemas.forEach(schema => console.log(`  - ${schema.name}`));
        console.log();
      } else if (input === 'y') {
        confirmed = true;
      } else {
        console.log("Import cancelled.");
        Deno.exit(0);
      }
    }
  }

  // Create schemas on the server
  const { apiConfiguration, workspaceId } = context;
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
        if (error.status !== 404) {
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
      log.error(`Error creating schemas: ${error.message}`);
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
      throw new Error(`Error reading actions directory for asset "${assetName}": ${error.message}`);
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
        name: actionMetadata.name,
        displayName: actionMetadata.displayName,
        description: actionMetadata.description,
        kind,
        code: code,
      };

      actions.push(actionObject);
    } catch (error) {
      log.error(`Error processing action "${strippedFileName}" for asset "${assetName}": ${error.message}, skipping...`);
    }
  }

  return actions;
}

