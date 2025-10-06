import { Command } from "@cliffy/command";
import { Configuration, WhoamiApi } from "@systeminit/api-client";
import { extractConfig } from "./config.ts";
import { exportAssets } from "./cli/export-assets.ts";
import { Log } from "./log.ts";
import { AuthApiClient, WorkspaceDetails } from "./auth-api-client.ts";
import { unknownValueToErrorMessage } from "./helpers.ts";

export type CliContext = {
  apiConfiguration: Configuration;
  log: Log;
  workspace: WorkspaceDetails;
};
async function whoami(context: CliContext) {
  const { apiConfiguration, log } = context;
  const whoamiApi = new WhoamiApi(apiConfiguration);

  try {
    const result = await whoamiApi.whoami();
    console.log(JSON.stringify(result.data, null, 2));
  } catch (error) {
    log.error(unknownValueToErrorMessage(error));
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

  await new Command()
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
      const workspace = await getWorkspaceDetails(apiToken, workspaceId);
      const context: CliContext = { apiConfiguration, workspace, log };
      return whoami(context);
    })
    .command("export-assets <assets-path:string>", "Export all assets from the assets folder")
      .option("-s, --skip-confirmation", "Skip confirmation prompt")
    .action(async ({ verbose, skipConfirmation }, assetsPath) => {
      const log = new Log(verbose);
      const workspace = await getWorkspaceDetails(apiToken, workspaceId);
      const context: CliContext = { apiConfiguration, workspace, log };
      return exportAssets(context, assetsPath, skipConfirmation);
    }).parse(Deno.args);
}

async function getWorkspaceDetails(apiToken: string, workspaceId: string) {
  const authApi = new AuthApiClient(apiToken, workspaceId);

  let workspace: WorkspaceDetails;
  try {
    workspace = await authApi.getWorkspaceDetails();
  } catch (error) {
    console.error(unknownValueToErrorMessage(error));
    Deno.exit(1);
  }

  return workspace;
}