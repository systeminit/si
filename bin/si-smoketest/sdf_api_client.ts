// sdf_client.ts
import JWT from "npm:jsonwebtoken";

export class SdfApiClient {
  private readonly token: string;
  private readonly baseUrl: string;
  public readonly workspaceId: string;

  // Constructor is private to enforce using the init method
  private constructor(
    token: string,
    baseUrl: string,
    workspaceId: string
  ) {
    this.token = token;
    this.baseUrl = baseUrl;
    this.workspaceId = workspaceId;
  }

  // Initializes the SdfApiClient with authentication
  public static async init(
    workspaceId: string,
    userEmailOrId: string,
    password: string
  ) {
    const token = await getSdfJWT(workspaceId, userEmailOrId, password);
    const baseUrl = Deno.env.get("SDF_API_URL");

    if (!baseUrl) {
      throw new Error("SDF_API_URL environment variable is missing.");
    }

    return new SdfApiClient(token, baseUrl, workspaceId);
  }

  // General fetch method
  private async fetch(path: string, options?: {
    headers?: Record<string, string>
    body?: Record<string, unknown>
    method?: "GET" | "POST" | "PUT" | "DELETE" | "PATCH"
  }) {
    const resp = await this.fetch_no_throw(path, options);
    if (!resp.ok) {
      throw new Error(`Error ${resp.status}: ${await resp.text()}`);
    }

    return resp;
  }

  // Fetch method without automatic error throwing
  private fetch_no_throw(path: string, options?: {
    headers?: Record<string, string>
    body?: Record<string, unknown>
    method?: "GET" | "POST" | "PUT" | "DELETE" | "PATCH"
  }) {
    const url = `${this.baseUrl}${path}`;
    const method = options?.method || "GET";
    console.log(`calling ${method} ${url}`);

    const headers = {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${this.token}`,
      "Cache-Control": "no-cache",
      ...options?.headers || {}
    };

    const body = options?.body ? JSON.stringify(options.body) : undefined;

    return fetch(url, {
      headers, body, method
    });
  }

  // High-level API methods are located below
  // --------------------------------------------------------------------------------

  // Fetch list of open change sets
  public async listOpenChangeSets() {
    return listOpenChangeSets(this);
  }

  // Create a new change set
  public async createChangeSet(changeSetName: string) {
    return createChangeSet(this, changeSetName);
  }

  // Fetch list of schema variants for a specific change set
  public async listSchemaVariants(changeSetId: string) {
    return listSchemaVariants(this, changeSetId);
  }

  // Create a component in a diagram
  public async createComponent(payload: Record<string, unknown>) {
    return createComponent(this, payload);
  }

  // Get the current state of a diagram
  public async getDiagram(changeSetId: string) {
    return getDiagram(this, changeSetId);
  }

  // Delete components from a diagram
  public async deleteComponents(payload: Record<string, unknown>) {
    return deleteComponents(this, payload);
  }

  // Abandon a change set
  public async abandonChangeSet(changeSetId: string) {
    return abandonChangeSet(this, changeSetId);
  }
}

// Helper functions for JWT generation and fetching
async function getSdfJWT(workspaceId: string, userEmailOrId: string, password: string) {
  const privateKey = Deno.env.get("JWT_PRIVATE_KEY");
  if (privateKey && privateKey.length > 0) {
    console.log("JWT_PRIVATE_KEY is set, signing jwt locally. UserId should be passed in instead of email");

    return createJWTFromPrivateKey(workspaceId, userEmailOrId, privateKey);
  } else {
    return getSdfJWTFromAuth0(workspaceId, userEmailOrId, password);
  }
}

async function getSdfJWTFromAuth0(workspaceId: string, email: string, password: string): Promise<string> {
  const authApiUrl = Deno.env.get("AUTH_API_URL");

  if (!authApiUrl || authApiUrl.length === 0) {
    throw new Error("Missing AUTH_API_URL");
  }

  const loginResp = await fetch(`${authApiUrl}/auth/login`, {
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      email,
      password,
      workspaceId
    }),
    method: "POST"
  });

  const { token, message } = await loginResp.json();

  if (!token) {
    const errorMessage = message ?? "Unknown Error";
    throw new Error(`Could not get token: ${errorMessage}`);
  }

  return token;
}

async function createJWTFromPrivateKey(
  workspaceId: string,
  userId: string,
  privateKey: string
): Promise<string> {
  return JWT.sign({
    user_pk: userId,
    workspace_pk: workspaceId
  }, privateKey, { algorithm: "RS256", subject: userId });
}

// API-Endpoint Specific Functions
// --------------------------------------------------------------------------------

// List open change sets
async function listOpenChangeSets(client: SdfApiClient) {
  const response = await client.fetch("/change_set/list_open_change_sets");
  return await response.json();
}

// Create a change set
async function createChangeSet(client: SdfApiClient, changeSetName: string) {
  const response = await client.fetch("/change_set/create_change_set", {
    method: "POST",
    body: { changeSetName },
  });
  return await response.json();
}

// List schema variants for a change set
async function listSchemaVariants(client: SdfApiClient, changeSetId: string) {
  const response = await client.fetch(`/v2/workspaces/${client.workspaceId}/change-sets/${changeSetId}/schema-variants`);
  return await response.json();
}

// Create a component in a diagram
async function createComponent(client: SdfApiClient, payload: Record<string, unknown>) {
  const response = await client.fetch("/diagram/create_component", {
    method: "POST",
    body: payload,
  });
  return await response.json();
}

// Get the current state of a diagram
async function getDiagram(client: SdfApiClient, changeSetId: string) {
  const response = await client.fetch(`/diagram/get_diagram?visibility_change_set_pk=${changeSetId}&workspaceId=${client.workspaceId}`);
  return await response.json();
}

// Delete components from a diagram
async function deleteComponents(client: SdfApiClient, payload: Record<string, unknown>) {
  await client.fetch("/diagram/delete_components", {
    method: "POST",
    body: payload,
  });
}

// Abandon a change set
async function abandonChangeSet(client: SdfApiClient, changeSetId: string) {
  await client.fetch("/change_set/abandon_change_set", {
    method: "POST",
    body: { changeSetId },
  });
}
