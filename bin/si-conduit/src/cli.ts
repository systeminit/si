import { Command } from "@cliffy/command";
import { Configuration, WhoamiApi } from "@systeminit/api-client";
import { extractConfig, tryGetUserDataFromToken } from "./config.ts";
import { pushAssets } from "./cli/push-assets.ts";
import { scaffoldAsset } from "./cli/scaffold-asset.ts";
import { Log } from "./log.ts";
import { AuthApiClient, WorkspaceDetails } from "./auth-api-client.ts";
import { unknownValueToErrorMessage } from "./helpers.ts";
import { Analytics } from "./analytics.ts";

export type BaseCliContext = {
  log: Log;
  analytics: Analytics;
};

export type AuthenticatedCliContext = BaseCliContext & {
  apiConfiguration: Configuration;
  workspace: WorkspaceDetails;
};

/// From the environment variables, extract the configuration needed to run auth commands
async function initializeCliContextWithAuth(
  baseCtx: BaseCliContext,
): Promise<AuthenticatedCliContext> {
  const { log, analytics } = baseCtx;
  const { apiUrl, apiToken, workspaceId } = extractConfig();

  log.debug(`Initializing CLI context with auth, pointing at ${apiUrl}, workspace ${workspaceId}`)

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

  return { apiConfiguration, workspace, log, analytics };
}

async function whoami(context: AuthenticatedCliContext) {
  const { apiConfiguration, log } = context;
  const whoamiApi = new WhoamiApi(apiConfiguration);

  const result = await whoamiApi.whoami();
  console.log(JSON.stringify(result.data, null, 2));
}

export async function run() {
  const analytics = new Analytics(tryGetUserDataFromToken());

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
      Deno.exit(0);
    })
    .command("whoami", "Get current user information")
    .action(async ({ verbose }) => {
      const log = new Log(verbose);

      const ctx = await initializeCliContextWithAuth({ log, analytics });

      return whoami(ctx);
    })
    .command(
      "push <assets-path:string>",
      "Push all assets from the assets folder",
    )
    .option("-s, --skip-confirmation", "Skip confirmation prompt")
    .action(async ({ verbose, skipConfirmation }, assetsPath) => {
      const log = new Log(verbose);

      const ctx = await initializeCliContextWithAuth({ log, analytics });

      return pushAssets(ctx, assetsPath, skipConfirmation);
    })
    .command("scaffold <asset-name:string>", "Scaffold a new asset")
    .option("-f, --folder <folder:string>", "Asset folder path", {
      default: ".",
    })
    .action(({ verbose, folder }, assetName) => {
      const log = new Log(verbose);

      return scaffoldAsset({ analytics, log }, assetName, folder);
    });

  let exitCode = 0;
  try {
    await cmd.parse(Deno.args);
  } catch (error) {
    const errorMsg = unknownValueToErrorMessage(error);

    (new Log(0)).error(errorMsg);

    const [command, ...args] = Deno.args;

    analytics.trackEvent("cli_error", {
      error: errorMsg,
      command,
      args,
    });

    exitCode = 1;
  }

  // Idle for a bit to allow the analytics event to be sent
  await analytics.shutdown();

  Deno.exit(exitCode);
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
