export interface Config {
  apiUrl: string;
  apiToken: string;
  workspaceId: string;
}

function decodeJWT(token: string): Record<string, unknown> {
  try {
    const parts = token.split(".");
    if (parts.length !== 3) {
      throw new Error("Invalid JWT format");
    }
    const payload = parts[1];
    const decoded = atob(payload.replace(/-/g, "+").replace(/_/g, "/"));
    return JSON.parse(decoded);
  } catch (error) {
    throw new Error(`Failed to decode JWT: ${error.message}`);
  }
}

export function extractConfig(): Config {
  // Get configuration from environment variables
  const apiUrl = Deno.env.get("SI_API_URL") || "https://api.systeminit.com";
  const apiToken = Deno.env.get("SI_API_TOKEN");

  if (!apiToken) {
    console.error("Error: SI_API_TOKEN environment variable is required.");
    Deno.exit(1);
  }

  // Extract workspaceId from JWT if not provided
  let workspaceId = Deno.env.get("SI_WORKSPACE_ID");
  if (!workspaceId) {
    try {
      const payload = decodeJWT(apiToken);
      workspaceId = payload.workspaceId as string;
      if (!workspaceId) {
        console.error("Error: workspaceId not found in JWT payload");
        Deno.exit(1);
      }
    } catch (error) {
      console.error(`Error: ${error.message}`);
      Deno.exit(1);
    }
  }

  return {
    apiUrl,
    apiToken,
    workspaceId,
  };
}