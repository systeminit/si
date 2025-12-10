import { createServer } from "node:net";
import { open } from "jsr:@opensrc/deno-open";
import { Context } from "../context.ts";
import { AuthApiClient } from "./auth.ts";
import {
  getUserDetails,
  getWorkspaceDetails,
  setCurrentUser,
  setCurrentWorkspace,
  writeUser,
  writeWorkspace,
} from "./config.ts";
import * as prompt from "./prompt.ts";
import { getUserDataFromToken } from "./jwt.ts";

const START_PORT = 9003;
const MAX_ATTEMPTS = 67;

export class NoPortAvailable extends Error {
  constructor(
    public readonly startPort: number,
    public readonly endPort: number,
  ) {
    super(`No available port found in range ${startPort}-${endPort}`);
    this.name = "NoPortAvailable";
  }
}

/**
 * Looks for an available port starting from the given port number.
 *
 * @param startPort - The port number to start probing from (default: 9003)
 * @param maxAttempts - Maximum number of ports to try (default: 67)
 * @returns Promise<number> - The first available port found
 * @throws NoPortAvailable if no available port is found within maxAttempts
 */
export async function findAvailablePort(
  startPort: number = START_PORT,
  maxAttempts: number = MAX_ATTEMPTS,
): Promise<number> {
  for (let port = startPort; port < startPort + maxAttempts; port++) {
    if (await isPortAvailable(port)) {
      return port;
    }
  }

  throw new NoPortAvailable(startPort, startPort + maxAttempts - 1);
}

/**
 * Attempts to bind to a port to see if it's available. This could race of
 * course, if someone else is trying to bind to this port immediately
 * afterwards, but we'll take that chance.
 *
 * @param port - The port number to check
 * @returns Promise<boolean> - True if the port is available, false otherwise
 */
function isPortAvailable(port: number): Promise<boolean> {
  return new Promise((resolve) => {
    const server = createServer();

    server.listen(port, () => {
      server.close(() => {
        resolve(true);
      });
    });

    server.on("error", () => {
      resolve(false);
    });
  });
}

export async function openAuthLogin(
  _ctx: Context,
  authApiUrl: string,
  port: number,
): Promise<void> {
  const url = `${authApiUrl}/auth/login?cli_redir=${port}`;
  console.log(
    `Opening login page. If a browser does not open, open ${url} in a browser running on this machine`,
  );
  await open(url);
}

/**
 * Spawns a temporary web server that waits for an authentication callback.
 *
 * @returns Promise<string> - The user id just authenticated
 * @throws NoPortAvailable if no available port is found
 */
export function waitForAuthCallback(
  authApiUrl: string,
  port: number,
): Promise<string> {
  return new Promise((resolve, reject) => {
    const server = Deno.serve(
      {
        port,
        hostname: "localhost",
      },
      async (req: Request) => {
        const url = new URL(req.url);

        if (req.method !== "GET") {
          return new Response("Method not allowed", { status: 405 });
        }
        if (url.pathname !== "/") {
          return new Response("Not found", { status: 404 });
        }

        const queryParams = Object.fromEntries(url.searchParams);

        const nonce = queryParams.nonce;
        if (!nonce) {
          throw new Response("Nonce required", { status: 422 });
        }

        try {
          const apiClient = new AuthApiClient(authApiUrl);
          const token = await apiClient.getAuthApiTokenFromNonce(nonce);
          const whoami = await apiClient.whoami();
          const userId = whoami.id;
          writeUser(whoami, token);
          const nickname = whoami.nickname ?? whoami.email;

          // Respond to the request
          const response = new Response(
            `Hello ${nickname}. You can close this window.`,
            {
              status: 200,
              headers: { "Content-Type": "text/plain" },
            },
          );

          // Close the server and resolve with the query parameters
          // We need to do this asynchronously to allow the response to be sent first
          setTimeout(async () => {
            try {
              await server.shutdown();
              resolve(userId);
            } catch (error) {
              reject(error);
            }
          }, 250);

          return response;
        } catch (err) {
          setTimeout(async () => {
            await server.shutdown();
            reject(err);
          }, 250);

          throw new Response(`Failed to acquire or store API token: ${err}.`, {
            status: 500,
          });
        }
      },
    );
  });
}

export async function doLogin(authApiUrl: string): Promise<string> {
  const ctx = Context.instance();

  if (!ctx.isInteractive) {
    throw new Error(
      "Login flow is not supported in non-interactive mode. Please set token via command line or environment, or login on an interactive terminal",
    );
  }

  try {
    ctx.logger.info("Starting local oauth callback webserver...");
    const port = await findAvailablePort();
    const serverPromise = waitForAuthCallback(authApiUrl, port);
    await openAuthLogin(ctx, authApiUrl, port);
    const userId = await serverPromise;
    const { userDetails, token } = getUserDetails(userId);
    ctx.logger.info(
      `Logged in as ${userDetails?.email}. Setting as default user.`,
    );
    setCurrentUser(userId);
    const authApiClient = new AuthApiClient(authApiUrl, token);
    const workspaces = await authApiClient.getWorkspaces();

    // Prompt for workspace selection
    const selectedWorkspaceId = await prompt.workspace(undefined, workspaces);
    const details = workspaces.find((w) => w.id === selectedWorkspaceId);
    if (!details) {
      // should be impossible
      throw new Error(`Workspace not found: ${selectedWorkspaceId}`);
    }

    const { workspaceDetails: _, token: maybeWorkspaceToken } =
      getWorkspaceDetails(userId, selectedWorkspaceId);
    let workspaceAutomationToken;
    if (!maybeWorkspaceToken) {
      const workspaceToken =
        await authApiClient.createWorkspaceToken(selectedWorkspaceId);
      workspaceAutomationToken = workspaceToken;
      writeWorkspace(userId, details, workspaceToken);
    } else {
      workspaceAutomationToken = maybeWorkspaceToken;
    }
    setCurrentWorkspace(selectedWorkspaceId);
    const tokenExpiration = getUserDataFromToken(workspaceAutomationToken)?.exp;
    if (tokenExpiration) {
      ctx.logger.info(`Workspace token expires at: ${tokenExpiration}`);
    }

    const selectedWorkspace = workspaces.find(
      (w) => w.id === selectedWorkspaceId,
    );

    const workspaceName = selectedWorkspace?.displayName || selectedWorkspaceId;
    ctx.logger.info(`Set default workspace to: ${workspaceName}`);

    return workspaceAutomationToken;
  } catch (error) {
    ctx.logger.error(`Login failed: ${error}`);
    throw error;
  }
}
