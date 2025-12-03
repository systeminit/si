import { createServer } from "node:net";
import { open } from "jsr:@opensrc/deno-open";

const START_PORT = 9003;
const MAX_ATTEMPTS = 67;

export class NoPortAvailable extends Error {
  constructor(
    public readonly startPort: number,
    public readonly endPort: number,
  ) {
    super(
      `No available port found in range ${startPort}-${endPort}`,
    );
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

/**
 * Spawns a temporary web server that waits for an authentication callback.
 *
 * @returns Promise<{port: number, queryParams: { [key: string]: string }> - The port used and query parameters from the GET request
 * @throws NoPortAvailable if no available port is found
 */
export async function waitForAuthCallback(): Promise<{
  port: number;
  queryParams: { [key: string]: string };
}> {
  const port = await findAvailablePort();

  return new Promise((resolve, reject) => {
    const server = Deno.serve(
      {
        port,
        hostname: "localhost",
      },
      (req: Request) => {
        const url = new URL(req.url);

        if (req.method !== "GET") {
          return new Response("Method not allowed", { status: 405 });
        }

        const queryParams = Object.fromEntries(url.searchParams);

        // Respond to the request
        const response = new Response(
          "Authentication callback received. You can close this window.",
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
            resolve({ port, queryParams });
          } catch (error) {
            reject(error);
          }
        }, 250);

        return response;
      },
    );
  });
}
