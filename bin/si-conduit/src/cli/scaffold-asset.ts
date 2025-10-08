import { CliContext } from "../cli.ts";
import { makeStringSafeForFilename } from "../helpers.ts";
import { Log } from "../log.ts";
import { ensureDir } from "jsr:@std/fs/ensure-dir";

export async function scaffoldAsset(context: CliContext, assetName: string, assetFolder: string) {
  const { log } = context;

  const assetDirName = makeStringSafeForFilename(assetName);

  const assetPath = `${assetFolder}/${assetDirName}`;

  // Check if asset folder already exists
  // The only way this function does not end here is if stat throws AND the error is `NotFound`
  try {
    await Deno.stat(assetPath);
    log.error(`Asset "${assetName}" already exists at ${assetPath}`);
    Deno.exit(1);
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw error;
    }
  }

  // Create asset folder
  try {
    await Deno.mkdir(assetPath, { recursive: true });
    log.debug(`Created asset folder: ${assetPath}`);

    // Create metadata.json
    const metadata = {
      name: assetName,
      category: "",
      description: "optional",
      documentation: "optional, should be a link",
    };
    await Deno.writeTextFile(
      `${assetPath}/metadata.json`,
      JSON.stringify(metadata, null, 2)
    );
    log.debug(`Created metadata.json`);

    // Create index.ts
    const indexContent = `function main() {\n  return new AssetBuilder().build();\n}\n`;
    await Deno.writeTextFile(`${assetPath}/index.ts`, indexContent);
    log.debug(`Created index.ts`);

    await createQualificationScaffold(assetPath, log);
    await createActionScaffold(assetPath, "create", log);
    await createActionScaffold(assetPath, "destroy", log);
    await createActionScaffold(assetPath, "refresh", log);
    await createActionScaffold(assetPath, "update", log);
    await createCodegenScaffold(assetPath, log);
    await createMgmtScaffold(assetPath, log);

    log.info(`Asset "${assetName}" created successfully at ${assetPath}`);
  } catch (error) {
    log.error(`Error creating asset: ${error.message}`);
    Deno.exit(1);
  }
}

async function createFunctionScaffold(
  log: Log,
  path: string,
  fileName: string,
  metadata: Record<string, unknown>,
  code: string,
) {
  await ensureDir(path);

  // Create the metadata file
  await Deno.writeTextFile(
      `${path}/${fileName}.json`,
      JSON.stringify(metadata, null, 2)
  );
  log.debug(`Created metadata file`);

  // Create the code file
  await Deno.writeTextFile(`${path}/${fileName}.ts`, code);
  log.debug(`Created code file`);
}

async function createQualificationScaffold(assetPath: string, log: Log) {
  // FIXME qualification code
  const code = `function main() {\n  return new AssetBuilder().build();\n}\n`;

  await createFunctionScaffold(
    log,
    `${assetPath}/qualifications`,
    "sample",
    {
      name: "Does asset comply with criteria?",
      displayName: "optional",
      description: "optional",
      inputs: [
        "code",
        "deletedAt",
        "domain",
        "resource",
        "secrets"
      ],
    },
     code
  );

  log.debug(`Created scaffold qualification`);
}
// Create one of each kind of action (create, destroy, refresh, update)
type funcKind = "create" | "destroy" | "refresh" | "update";
async function createActionScaffold(assetPath: string, kindOrName: funcKind | string,  log: Log) {
  // FIXME action function code
  const code = `function main() {\n  return new AssetBuilder().build();\n}\n`;

  await createFunctionScaffold(
      log,
      `${assetPath}/actions`,
      kindOrName,
      {
        name: "Do Something!",
        displayName: "optional",
        description: "optional",
      },
      code
  );

  log.debug(`Created scaffold ${kindOrName} action`);
}


// Create a codegen func
async function createCodegenScaffold(assetPath: string, log: Log) {
  const code = `function main() {
  const code = {};
  return {
    format: "json",
    code: JSON.stringify(code, null, 2),
  };
}`;

  await createFunctionScaffold(
      log,
      `${assetPath}/codeGenerators`,
      "sample",
      {
        name: "Does asset comply with criteria?",
        displayName: "optional",
        description: "optional",
        inputs: [
          "code",
          "deletedAt",
          "domain",
          "resource",
          "secrets"
        ],
      },
      code
  );

  log.debug(`Created scaffold codegen`);
}

async function createMgmtScaffold(assetPath: string, log: Log) {
  // FIXME management function code
  const code = `function main() {\n  return new AssetBuilder().build();\n}\n`;

  // FIXME make sample mgmt func name
  await createFunctionScaffold(
      log,
      `${assetPath}/management`,
      "sample",
      {
        name: "Does asset comply with criteria?",
        displayName: "optional",
        description: "optional",
      },
      code
  );

  log.debug(`Created scaffold qualification`);
}
