import { Command } from "@cliffy/command";
import { Configuration, WhoamiApi } from "@systeminit/api-client";
import { extractConfig } from "./config.ts";
import { exportAssets } from "./cli/export-assets.ts";
import { scaffoldAsset } from "./cli/scaffold-asset.ts";
import { Log } from "./log.ts";
import { AuthApiClient, WorkspaceDetails } from "./auth-api-client.ts";
import { unknownValueToErrorMessage } from "./helpers.ts";

export type CliContext = {
  apiConfiguration: Configuration;
  log: Log;
  workspace: WorkspaceDetails;
};

/// From the environment variables, extract the configuration needed to run auth commands
async function initializeCliContextWithAuth(log: Log) {
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

  const workspace = await getWorkspaceDetails(apiToken, workspaceId);

  return { apiConfiguration, workspace, log };
}

async function whoami(context: CliContext) {
  const { apiConfiguration, log } = context;
  const whoamiApi = new WhoamiApi(apiConfiguration);

  const result = await whoamiApi.whoami();
  console.log(JSON.stringify(result.data, null, 2));
}

export async function run() {
  const cmd = new Command()
    .name("si-conduit")
    .version("0.1.0")
    .description("The System Initiative Asset Management CLI Tool")
    .globalOption(
      "-v, --verbose [level:number]",
      "Enable verbose logging (optionally specify level: 0 -> only errors and output, 1 -> 0 + info messages or  2 -> 1 + debug messages)",
      (value) => value === true ? 1 : value,
    )
    .action(function () {
      this.showHelp();
      Deno.exit(1);
    })
    .command("whoami", "Get current user information")
    .action(async ({ verbose }) => {
      const log = new Log(verbose);

      const ctx = await initializeCliContextWithAuth(log);

      return whoami(ctx);
    })
    .command("export-assets <assets-path:string>", "Export all assets from the assets folder")
      .option("-s, --skip-confirmation", "Skip confirmation prompt")
    .action(async ({ verbose, skipConfirmation }, assetsPath) => {
      const log = new Log(verbose);

      const ctx = await initializeCliContextWithAuth(log);

      return exportAssets(ctx, assetsPath, skipConfirmation);
    })
    .command("scaffold <asset-name:string>", "Scaffold a new asset")
    .option("-f, --folder <folder:string>", "Asset folder path", { default: "." })
    .action(({ verbose, folder }, assetName) => {
      const log = new Log(verbose);
      return scaffoldAsset(log, assetName, folder);
    });

    try {
      await cmd.parse(Deno.args);
    } catch (error) {
      console.error(unknownValueToErrorMessage(error));
      Deno.exit(1);
    }
}

async function getWorkspaceDetails(apiToken: string, workspaceId?: string) {
  if (!workspaceId) {
    throw new Error("Workspace ID is required");
  }

  const authApi = new AuthApiClient(apiToken, workspaceId);

  let workspace: WorkspaceDetails;
  workspace = await authApi.getWorkspaceDetails();

  return workspace;
}