import { Command } from "@cliffy/command";
import { Configuration, ChangeSetsApi, WhoamiApi } from "@systeminit/api-client";
import { extractConfig } from "./config.ts";
import { exportAssets } from "./cli/export-assets.ts";
import { Log } from "./log.ts";
import { AuthApiClient } from "./auth-api-client.ts";

export type CliContext = {
  apiConfiguration: Configuration;
  workspaceId: string;
  log: Log;
  authApi: AuthApiClient;
};

async function listChangesets(context: CliContext) {
  const { apiConfiguration, workspaceId, log } = context;
  const changeSetsApi = new ChangeSetsApi(apiConfiguration);

  try {
    const changesetsResponse = await changeSetsApi.listChangeSets({
      workspaceId
    });

    const changesets = changesetsResponse.data.changeSets;

    if (!changesets) {
      log.error("Malformed changeset list response");
      Deno.exit(1);
    }

    log.info(JSON.stringify(changesets));
    const headChangeset = changesets.find((cs) => cs.isHead);

    if (!headChangeset) {
      log.error("No head changeset found");
      Deno.exit(1);
    }

    console.log(`${headChangeset.id}: ${headChangeset.name} *`);
    changesets.forEach((cs) => {
      if (cs.isHead) return;
      console.log(`${cs.id}: ${cs.name}`);
    })
  } catch (error) {
    log.error(error.message);
    Deno.exit(1);
  }
}

async function whoami(context: CliContext) {
  const r = await context.authApi.getWorkspaceDetails()
  console.log(r)

  const { apiConfiguration, log } = context;
  const whoamiApi = new WhoamiApi(apiConfiguration);

  try {
    const result = await whoamiApi.whoami();
    console.log(JSON.stringify(result.data, null, 2));
  } catch (error) {
    log.error(error.message);
    Deno.exit(1);
  }
}

export async function run() {
  const { apiUrl, apiToken, workspaceId } = extractConfig();

  const apiConfiguration = new Configuration({
    basePath: apiUrl,
    accessToken: apiToken,
    baseOptions: {
      headers: {
        Authorization: `Bearer ${apiToken}`,
      },
    },
  });

  const authApi = new AuthApiClient(apiToken, workspaceId);

  const command = new Command()
    .name("si-conduit")
    .version("0.1.0")
    .description("The System Initiative Asset Management CLI Tool")
    .globalOption(
      "-v, --verbose [level:number]",
      "Enable verbose logging (optionally specify level: 0 -> only errors and output, 1 -> 0 + info messages or  2 -> 1 + debug messages)",
      (value) => value === true ? 1 : value,
    )
    .action(() => {
      command.showHelp();
      Deno.exit(1);
    })
    .command("change-sets", "List changesets")
    .action(({ verbose }) => {
      const log = new Log(verbose);
      const context: CliContext = { apiConfiguration, workspaceId, log, authApi };
      return listChangesets(context);
    })
    .command("whoami", "Get current user information")
    .action(({ verbose }) => {
      const log = new Log(verbose);
      const context: CliContext = { apiConfiguration, workspaceId, log, authApi };
      return whoami(context);
    })
    .command("export-assets [assets-path:string]", "Export all assets from the assets folder")
    .option("-s, --skip-confirmation", "Skip confirmation prompt")
    .action(({ verbose, skipConfirmation }, assetsPath) => {
      const log = new Log(verbose);
      const context: CliContext = { apiConfiguration, workspaceId, log, authApi };
      return exportAssets(context, assetsPath, skipConfirmation);
    });

  await command.parse(Deno.args);
}

