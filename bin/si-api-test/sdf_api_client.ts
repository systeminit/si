// sdf_client.ts
import JWT from "npm:jsonwebtoken";
import { retryWithBackoff } from "./test_helpers.ts";

type HTTP_METHOD = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
type ROUTE_VARS = Record<string, string>;

interface API_DESCRIPTION {
  path: (vars: ROUTE_VARS) => string;
  method: HTTP_METHOD;
  headers?: Record<string, string>;
}

export const ROUTES = {
  // /api/change_set - Change Set Management ---------------------------------------------------
  abandon_change_set: {
    path: () => "/change_set/abandon_change_set",
    method: "POST",
  },
  abandon_vote: {
    path: () => "/change_set/abandon_vote",
    method: "POST",
  },
  add_action: {
    path: () => "/change_set/add_action", 
    method: "POST",
  },
  apply_change_set: {
    path: () => "/change_set/apply_change_set",
    method: "POST",
  },
  begin_abandon_approval_process: {
    path: () => "/change_set/begin_abandon_approval_process",
    method: "POST",
  },
  begin_approval_process: {
    path: () => "/change_set/begin_approval_process",
    method: "POST",
  },
  cancel_abandon_approval_process: {
    path: () => "/change_set/cancel_abandon_approval_process",
    method: "POST",
  },
  cancel_approval_process: {
    path: () => "/change_set/cancel_approval_process",
    method: "POST",
  },
  create_change_set: {
    path: () => "/change_set/create_change_set",
    method: "POST",
  },
  list_open_change_sets: {
    path: () => "/change_set/list_open_change_sets",
    method: "GET",
  },
  merge_vote: {
    path: () => "/change_set/merge_vote",
    method: "POST",
  },
  rebase_on_base: {
    path: () => "/change_set/rebase_on_base",
    method: "POST",
  },
  status_with_base: {
    path: () => "/change_set/status_with_base",
    method: "POST",
  },

  // V2/Workspaces  ---------------------------------------------------------
  // TODO(MegaWatt01): come back to properly format this hanging route
  schema_variants: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/schema-variants`,
    method: "GET",
  },

  // Diagram Management ---------------------------------------------------------
  add_components_to_view: {
    path: () => "/diagram/add_components_to_view",
    method: "POST",
  },
  delete_connection:{
    path: () => "/diagram/delete_connection",
    method: "POST",
  },
  dvu_roots: {
    path: (vars: ROUTE_VARS) =>
      `/diagram/dvu_roots?visibility_change_set_pk=${vars.changeSetId}&workspaceId=${vars.workspaceId}`,
    method: "GET",
  },
  get_all_components_and_edges: {
    path: () => "/diagram/get_all_components_and_edges",
    method: "GET",
  },
  get_diagram: {
    path: (vars: ROUTE_VARS) =>
      `/diagram/get_diagram?visibility_change_set_pk=${vars.changeSetId}&workspaceId=${vars.workspaceId}`,
    method: "GET",
  },
  list_schemas: {
    path: () => "/diagram/list_schemas",
    method: "GET"
  },
  remove_delete_intent: {
    path: () => "/diagram/remove_delete_intent",
    method: "POST",
  },
  set_component_position: {
    path: () => "/diagram/set_component_position",
    method: "POST",
  },
  set_component_type: {
    path: () => "/component/set_type",
    method: "POST",
  },


  // Component Management -------------------------------------------------------
  delete_components: {
    path: () => "/diagram/delete_components",
    method: "POST",
  },
  create_component: {
    path: () => "/diagram/create_component",
    method: "POST",
  },
  create_connection: {
    path: () => "/diagram/create_connection",
    method: "POST",
  },

  // Property Editor ------------------------------------------------------------
  get_property_schema: {
    path: (vars: ROUTE_VARS) =>
      `/component/get_property_editor_schema?visibility_change_set_pk=${vars.changeSetId}&componentId=${vars.componentId}`,
    method: "GET",
  },
  get_property_values: {
    path: (vars: ROUTE_VARS) =>
      `/component/get_property_editor_values?visibility_change_set_pk=${vars.changeSetId}&componentId=${vars.componentId}`,
    method: "GET",
  },
  update_property_value: {
    path: () => `/component/update_property_editor_value`,
    method: "POST",
  },

  // Variant Management -----------------------------------------------------------
  create_variant: {
    path: () => `/variant/create_variant`,
    method: "POST",
  },
  save_variant: {
    path: () => `/variant/save_variant`,
    method: "POST",
  },
  regenerate_variant: {
    path: () => `/variant/regenerate_variant`,
    method: "POST",
  },
  get_variant: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/schema-variants/${vars.schemaVariantId}`,
    method: "GET",
  },

  // Action Management -----------------------------------------------------------
  action_list: {
    path: (vars: ROUTE_VARS) =>
      `/action/list?visibility_change_set_pk=${vars.changeSetId}`,
    method: "GET",
  },

  // Qualification ------------------------------------------------------
  qualification_summary: {
    path: (vars: ROUTE_VARS) =>
      `/qualification/get_summary?visibility_change_set_pk=${vars.changeSetId}`,
    method: "GET",
  },

  // Funcs ------------------------------------------------------
  func_list: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/funcs`,
    method: "GET",
  },
  create_func: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/funcs`,
    method: "POST",
  },
  create_func_arg: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/funcs/${vars.funcId}/arguments`,
    method: "POST",
  },
  create_func_binding: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/funcs/${vars.funcId}/bindings`,
    method: "PUT",
  },
  update_func_code: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/funcs/${vars.funcId}/code`,
    method: "PUT",
  },

  // Websockets -----------------------------------------
  workspace_updates_ws: {
    path: (vars: ROUTE_VARS) => `/ws/workspace_updates?token=${vars.token}`,
    method: "GET", // Not really relevant for WebSocket, but keeps the structure consistent
  },
  // Add more groups below ------------------------------------------------------
} satisfies Record<string, API_DESCRIPTION>;

export type ROUTE_NAMES = keyof typeof ROUTES;

interface API_CALL {
  route: ROUTE_NAMES;
  params?: Record<string, string | number | undefined>;
  routeVars?: ROUTE_VARS;
  body?: Record<string, unknown>;
}

export class SdfApiClient {
  public readonly workspaceId: string;
  public dvuListener: DVUListener;
  private readonly token: string;
  private readonly baseUrl: string;

  // Constructor is private to enforce using the init method
  private constructor(token: string, baseUrl: string, workspaceId: string) {
    this.token = token;
    this.baseUrl = baseUrl;
    this.workspaceId = workspaceId;
  }

  // Initializes the SdfApiClient with authentication
  public static async init(args: {
    workspaceId: string;
    userEmailOrId?: string;
    password?: string;
    token?: string;
  }) {
    let { workspaceId, userEmailOrId, password, token } = args;

    if (!token) {
      if (!userEmailOrId) {
        throw new Error("Must set token or userEmail!");
      }

      token = await getSdfJWT(workspaceId, userEmailOrId, password);
    }
    if (!token) {
      throw new Error("No auth token has been set!");
    }

    const baseUrl = Deno.env.get("SDF_API_URL");

    if (!baseUrl) {
      throw new Error("SDF_API_URL environment variable is missing.");
    }

    return new SdfApiClient(token, baseUrl, workspaceId);
  }

  public async call({ route, routeVars, params, body }: API_CALL) {
    const { path, method, headers } = ROUTES[route] as API_DESCRIPTION;
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
      return await response.json();
    } catch {
      return null;
    }
  }

  // General fetch method
  private async fetch(
    path: string,
    options?: {
      headers?: Record<string, string>;
      body?: Record<string, unknown>;
      method?: "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
    },
  ) {
    const resp = await this.fetch_no_throw(path, options);
    if (!resp.ok) {
      throw new Error(`Error ${resp.status}: ${await resp.text()}`);
    }

    return resp;
  }

  // Fetch method without automatic error throwing
  private fetch_no_throw(
    path: string,
    options?: {
      headers?: Record<string, string>;
      body?: Record<string, unknown>;
      method?: "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
    },
  ) {
    const url = `${this.baseUrl}${path}`;
    const method = options?.method || "GET";
    console.log(`calling ${method} ${url}`);

    const headers = {
      "Content-Type": "application/json",
      Authorization: `Bearer ${this.token}`,
      "Cache-Control": "no-cache",
      ...(options?.headers || {}),
    };

    const body = options?.body ? JSON.stringify(options.body) : undefined;

    return fetch(url, {
      headers,
      body,
      method,
    });
  }

  public listenForDVUs() {
    const url = `${this.baseUrl}${ROUTES.workspace_updates_ws.path({
      token: `Bearer ${this.token}`,
    })}`;
    const dvuListener = new DVUListener(url, this.workspaceId);
    this.dvuListener = dvuListener;

    console.log("Starting WebSocket listener for workspace updates...");
    dvuListener.listen();
  }

  public async waitForDVURoots(
    changeSetId: string,
    interval_ms: number,
    timeout_ms: number,
  ): Promise<void> {
    console.log(`Waiting on DVUs for ${this.workspaceId}...`);
    const dvuPromise = new Promise<void>((resolve) => {
      const interval = setInterval(async () => {
        const remainingRoots = await this.call({ route: "dvu_roots", routeVars: { changeSetId } });
        if (remainingRoots?.count === 0) {
          console.log(`All DVUs for ${this.workspaceId} finished!`);
          clearInterval(interval);
          resolve();
        } else {
          console.log(
            `Waiting for DVUs in workspace ${this.workspaceId} to finish, ${remainingRoots?.count} remain...`,
          );
        }
      }, interval_ms);
    });

    const timeoutPromise = new Promise<void>((_, reject) => {
      setTimeout(() => {
        console.log(
          `Timeout reached while waiting for DVUs in workspace ${this.workspaceId}.`,
        );
        reject(new Error("Timeout while waiting for DVUs to finish."));
      }, timeout_ms);
    });

    return Promise.race([dvuPromise, timeoutPromise]);
  }

  public async waitForDVUs(
    interval_ms: number,
    timeout_ms: number,
  ): Promise<void> {
    console.log(`Waiting on DVUs for ${this.workspaceId}...`);
    const dvuPromise = new Promise<void>((resolve) => {
      const interval = setInterval(() => {
        const remainingEvents = this.dvuListener.openEventCount();
        if (remainingEvents === 0) {
          console.log(`All DVUs for ${this.workspaceId} finished!`);
          clearInterval(interval);
          resolve();
        } else {
          console.log(
            `Waiting for DVUs in workspace ${this.workspaceId} to finish, ${remainingEvents} remain...`,
          );
        }
      }, interval_ms);
    });

    const timeoutPromise = new Promise<void>((_, reject) => {
      setTimeout(() => {
        console.log(
          `Timeout reached while waiting for DVUs in workspace ${this.workspaceId}.`,
        );
        reject(new Error("Timeout while waiting for DVUs to finish."));
      }, timeout_ms);
    });

    return Promise.race([dvuPromise, timeoutPromise]);
  }
}

// Helper functions for JWT generation and fetching
async function getSdfJWT(
  workspaceId: string,
  userEmailOrId: string,
  password?: string,
) {
  const privateKey = Deno.env.get("JWT_PRIVATE_KEY");
  if (privateKey && privateKey.length > 0) {
    console.log(
      "JWT_PRIVATE_KEY is set, signing jwt locally. UserId should be passed in instead of email",
    );

    return createJWTFromPrivateKey(workspaceId, userEmailOrId, privateKey);
  } else {
    if (!password) {
      throw new Error("No password provided");
    }
    let token;
    await retryWithBackoff(async () => {
      token = await getSdfJWTFromAuth0(workspaceId, userEmailOrId, password);
    });
    return token;
  }
}

async function getSdfJWTFromAuth0(
  workspaceId: string,
  email: string,
  password: string,
): Promise<string> {
  const authApiUrl = Deno.env.get("AUTH_API_URL");

  if (!authApiUrl || authApiUrl.length === 0) {
    throw new Error("Missing AUTH_API_URL");
  }

  const loginResp = await fetch(`${authApiUrl}/auth/login`, {
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      email,
      password,
      workspaceId,
    }),
    method: "POST",
  });

  if (!loginResp.ok) {
    throw new Error(`Could not get token: response status ${loginResp.status}`);
  }

  const { token, message } = await loginResp.json();

  if (!token) {
    const errorMessage = message ?? "Unknown Error";
    throw new Error(`Could not get token: ${errorMessage}`);
  }

  return token;
}

function createJWTFromPrivateKey(
  workspaceId: string,
  userId: string,
  privateKey: string,
): Promise<string> {
  return JWT.sign(
    {
      user_pk: userId,
      workspace_pk: workspaceId,
    },
    privateKey,
    { algorithm: "RS256", subject: userId },
  );
}

class DVUListener {
  private ws: WebSocket;
  private workspace: string;
  private events: {
    componentId: string;
    statusStarted: boolean;
  }[] = [];

  constructor(url: string, workspace: string) {
    this.ws = new WebSocket(url);
    this.workspace = workspace;
  }

  public listen() {
    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };

    this.ws.onopen = () => {
      console.log("WebSocket connection opened");
    };

    this.ws.onclose = () => {
      console.log("WebSocket connection closed");
    };
  }

  public openEventCount(): number {
    return this.events.length;
  }

  private handleMessage(message: any) {
    if (
      message.workspace_pk == this.workspace &&
      message.payload.kind === "StatusUpdate" &&
      message.payload.data.kind == "dependentValueUpdate"
    ) {
      const { status, componentId } = message.payload.data;

      if (status === "statusStarted") {
        const event = {
          componentId,
          statusStarted: true,
          statusFinished: false,
        };
        this.events.push(event);
      } else if (status === "statusFinished") {
        const eventIndex = this.events.findIndex(
          (event) => event.componentId === componentId && event.statusStarted,
        );
        if (eventIndex !== -1) {
          this.events.splice(eventIndex, 1);
        }
      }
    }
  }
}
