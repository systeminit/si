// sdf_client.ts
import JWT from "npm:jsonwebtoken";


type HTTP_METHOD = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
type ROUTE_VARS = Record<string, string>;
interface API_DESCRIPTION {
  path: (ROUTE_VARS) => string;
  method: HTTP_METHOD;
  headers?: Record<string, string>;
}

export const ROUTES = {

  // Change Set Management ------------------------------------------------------
  create_change_set: { path: () => '/change_set/create_change_set', method: 'POST'}, 
  abandon_change_set: { path: () => '/change_set/abandon_change_set', method: 'POST'}, 
  open_change_sets: { path: () => '/change_set/list_open_change_sets', method: 'GET' },
  schema_variants: { path: (vars: ROUTE_VARS) => `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/schema-variants`, method: 'GET' },

  // Diagram Management ---------------------------------------------------------
  get_diagram: { path: (vars: ROUTE_VARS) => `/diagram/get_diagram?visibility_change_set_pk=${vars.changeSetId}&workspaceId=${vars.workspaceId}`, method: 'GET' },

  // Component Management -------------------------------------------------------
  delete_component: { path: () => `/diagram/delete_components`, method: 'POST'},
  create_component: { path: () => '/diagram/create_component', method: 'POST' },
  create_connection: { path: () => '/diagram/create_connection', method: 'POST' },

  // Property Editor
  component_property_values: { path: (vars: ROUTE_VARS) => `/component/get_property_editor_values?visibility_change_set_pk=${vars.changeSetId}&component_id=${vars.componentId}`, method: 'GET'}

  // Add more groups below ------------------------------------------------------

} satisfies Record<string, API_DESCRIPTION>;

export type ROUTE_NAMES = keyof typeof ROUTES;

interface API_CALL {
  route: ROUTE_NAMES,
  params?: Record<string, string | number | undefined>,
  routeVars?: ROUTE_VARS,
  body?: Record<string, unknown>,
}
export class SdfApiClient {
  private readonly token: string;
  private readonly baseUrl: string;
  public readonly workspaceId: string;

  public async call({route, routeVars, params, body }: API_CALL) {
    let { path, method, headers } = ROUTES[route] as API_DESCRIPTION;
    if (!routeVars) routeVars = {};
    routeVars.workspaceId = this.workspaceId;
    const url = path(routeVars);

    const response = await this.fetch(url, {
        method,
        headers,
        body,
    });

    // Some endpoints return a body, others return nothing on success
    try {
      return await response.json()
    } catch {
      return null
    }
     
}

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