// sdf_client.ts
import JWT from "npm:jsonwebtoken";
import { retryUntil, retryWithBackoff, sleep, sleepBetween } from "./test_helpers.ts";

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
  add_action: {
    path: () => "/change_set/add_action",
    method: "POST",
  },
  force_apply: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/force_apply`,
    method: "POST",
  },
  apply: {
    path: (vars: ROUTE_VARS) => `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/apply`,
    method: "POST",
  },
  create_change_set: {
    path: () => "/change_set/create_change_set",
    method: "POST",
  },
  list_open_change_sets: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets`,
    method: "GET",
  },

  // Component Management -------------------------------------------------------
  create_component_v2: {
    path: (vars: ROUTE_VARS) => `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/views/${vars.viewId}/component`,
    method: "POST",
  },
  delete_components_v2: {
    path: (vars: ROUTE_VARS) => `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/components/delete`,
    method: "DELETE",
  },
  attributes: {
    path: (vars: ROUTE_VARS) => `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/components/${vars.componentId}/attributes`,
    method: "PUT",
  },
  upgrade: {
    path: (vars: ROUTE_VARS) => `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/components/upgrade`,
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
  create_unlocked_copy: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/schema-variants/${vars.schemaVariantId}`,
    method: "POST",
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
  test_execute: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/funcs/${vars.funcId}/test_execute`,
    method: "POST",
  },
  get_func_run: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/funcs/runs/${vars.funcRunId}`,
    method: "GET",
  },

  // Modules --------------------------------------
  install_module: {
    path: () => `/module/install_module`,
    method: "POST",
  },

  // Materialized Views --------------------------------------
  index: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/index`,
    method: "GET",
  },
  mjolnir: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/index/mjolnir?changeSetId=${vars.changeSetId}&kind=${vars.referenceKind}&id=${vars.materializedViewId}`,
    method: "GET",
  },
  multi_mjolnir: {
    path: (vars: ROUTE_VARS) =>
      `/v2/workspaces/${vars.workspaceId}/change-sets/${vars.changeSetId}/index/multi_mjolnir`,
    method: "POST",
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

  public async call({ route, routeVars, params, body }: API_CALL, noThrow?: boolean) {
    const { path, method, headers } = ROUTES[route] as API_DESCRIPTION;

    // Ensure routeVars is always defined and contains workspaceId
    routeVars = { ...routeVars, workspaceId: this.workspaceId };

    const url = path(routeVars);

    // Merge headers with default headers
    const optionsWithDefaultHeaders = {
      headers: {
        Authorization: `Bearer ${this.token}`,
        "Content-Type": "application/json",
        "Cache-Control": "no-cache",
        "User-Agent": "si.git/api-tests (support@systeminit.com)",
        ...(headers || {}),
      },
      method,
      body,
    };

    if (noThrow) {
      // If the caller wants the raw response, let's give it to them
      return await this.fetch_no_throw(url, optionsWithDefaultHeaders);
    } else {
      const response = await this.fetch(url, optionsWithDefaultHeaders);

      // Some endpoints return a body, others return nothing on success
      try {
        return await response.json();
      } catch {
        return null;
      }
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
    const optionsWithDefaultHeaders = {
      ...options,
      headers: {
        "User-Agent": "si.git/api-tests (support@systeminit.com)",
        ...(options?.headers || {}),
      },
    };

    const resp = await this.fetch_no_throw(path, optionsWithDefaultHeaders); // Fix: Pass the correct optionsWithDefaultHeaders
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
      Authorization: `Bearer ${this.token}`,
      "Cache-Control": "no-cache",
      "User-Agent": "si.git/api-tests (support@systeminit.com)",
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

    await retryUntil(async () => {
      const dvuRoots = await this.mjolnir(changeSetId, "DependentValueComponentList", this.workspaceId);
      if (dvuRoots.components && dvuRoots.components.length !== 0) {
        throw new Error("DVU is still being processed");
      }
    }, timeout_ms, "Timeout waiting for dvu roots to clear", interval_ms);

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


  // Helper functions for interacting with MVs
  public async mjolnir(
    changeSetId: string,
    kind: string,
    id: string,
  ): Promise<any | null> {

    const response = await this.call({
      route: "mjolnir",
      routeVars: { changeSetId, materializedViewId: id, referenceKind: kind },
    }, true);
    if (response?.status === 200) {
      try {
        const json = await response.json();
        return json.frontEndObject.data;
      } catch (err) {
        console.error("Error trying to parse response body as JSON", err);
      }

    } else if (response?.status === 404) {
      console.warn(`Materialized view for ${kind} with ID ${id} not (yet?) found`);
      throw new Error("Materialized view not (yet?) found for kind: " + kind + ", id: " + id);
    } else {
      // Fail on non-200 and non-404 errors
      console.error(`Error ${response.status}: Unable to fetch MV for ${kind} with ID ${id}:`, await response.text());
      throw new Error(`Error ${response.status}: ${await response.text()}`);
    }
    return null;
  }

  public async multiMjolnir(changeSetId: string, mvs: { kind: string; id: string }[]) {
    const response = await this.call({
      route: "multi_mjolnir",
      routeVars: { changeSetId },
      body: { requests: mvs },
    }, true);

    if (response?.status === 200) {
      try {
        const json = await response.json();
        if (json.failed && json.failed.length > 0) {
          console.warn("Some MVs were not found during multi mjolnir:", json.failed);
        }
        return json.successful.map((v: any) => v.frontEndObject.data);
      } catch (err) {
        console.error("Error trying to parse response body as JSON", err);
      }
    } else {
      // Fail on non-200 errors
      console.error(`Error ${response.status}: Unable to fetch MVs:`, await response.text());
      throw new Error(`Error ${response.status}: ${await response.text()}`);
    }
  }

  public async fetchChangeSetIndex(changeSetId: string, timeout_ms: number = 30000): Promise<any> {
    await retryUntil(
      async () => {
        // your operation that might fail
        const response = await this.call({
          route: "index",
          routeVars: { changeSetId },
        }, true);
        if (response?.status === 200) {
          const json = await response.json();
          return json;
        } else if (response?.status === 404) {
          console.warn(`ChangeSet index for ID ${changeSetId} not (yet?) found`);
          throw new Error("Index not found yet for changeset: " + changeSetId);
        }
        else if (response?.status === 202) {
          console.log(`ChangeSet index for ID ${changeSetId} is still being built (status 202)`);
          throw new Error("Index still being built for changeset: " + changeSetId);
        }
        else {
          // Fail on non-200 and non-404 errors
          console.error(`Error ${response.status}: Unable to fetch ChangeSet index for ID ${changeSetId}:`, await response.text());
          throw new Error(`Error ${response.status}: ${await response.text()}`);
        }
      },
      timeout_ms,
    );
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
      "User-Agent": "si.git/api-tests (support@systeminit.com)",
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
