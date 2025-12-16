import { getApiUrlForInstance } from "./config.ts";
import { tryGetUserDataFromToken } from "./jwt.ts";
import { getUserAgent } from "../git_metadata.ts";

export interface WorkspaceDetails {
  instanceUrl: string;
  displayName: string;
  id: string;
  instanceEnvType: string;
  baseUrl: string;
}

export interface UserDetails {
  id: string;
  email: string;
  nickname?: string;
  firstName: string;
  lastName: string;
}

export class AuthApiClient {
  private authApiUrl: string;
  private apiToken?: string;

  constructor(authApiUrl: string, apiToken?: string) {
    this.authApiUrl = authApiUrl;
    this.apiToken = apiToken;
  }

  async apiRequest(args: {
    method: string;
    apiPath: string;
    queryParams?: { [key: string]: string };
    body?: object;
    noAuth?: boolean;
  }): Promise<Response> {
    const { method, apiPath, queryParams, body, noAuth } = args;

    if (!noAuth && !this.apiToken) {
      throw new Error("API token required");
    }

    const baseUrl = `${this.authApiUrl}/${apiPath}`;
    const params = queryParams ? new URLSearchParams(queryParams) : undefined;
    const url = params ? `${baseUrl}?${params}` : baseUrl;
    const requestBody = body ? JSON.stringify(body) : undefined;
    let headers: { [key: string]: string } | undefined = noAuth
      ? { "User-Agent": getUserAgent() }
      : {
          Authorization: `Bearer: ${this.apiToken}`,
          "User-Agent": getUserAgent(),
        };
    if (requestBody) {
      if (headers) {
        headers["Content-Type"] = "application/json";
      } else {
        headers = {
          "Content-Type": "application/json",
          "User-Agent": getUserAgent(),
        };
      }
    }

    return await fetch(url, { method, headers, body: requestBody });
  }

  async getAuthApiTokenFromNonce(nonce: string): Promise<string> {
    const response = await this.apiRequest({
      method: "GET",
      apiPath: "auth/cli-auth-api-token",
      queryParams: { nonce },
      noAuth: true,
    });

    if (!response.ok) {
      throw new Error(`Failed to get auth API token: ${response.statusText}`);
    }

    const data = await response.json();
    this.apiToken = data.token;

    return data.token;
  }

  async createWorkspaceToken(workspaceId: string): Promise<string> {
    const response = await this.apiRequest({
      method: "POST",
      apiPath: `workspaces/${workspaceId}/authTokens`,
      body: {
        name: "SI CLI Token",
      },
    });

    if (!response.ok) {
      throw new Error(
        `Failed to create workspace token: ${response.statusText}`,
      );
    }

    const data = await response.json();
    return data.token;
  }

  async whoami(): Promise<UserDetails> {
    const response = await this.apiRequest({
      method: "GET",
      apiPath: "whoami",
    });

    if (!response.ok) {
      throw new Error(`Failed to get user details: ${response.statusText}`);
    }

    const payload = (await response.json()) as { user: UserDetails };
    return payload.user;
  }

  async getWorkspaces(): Promise<WorkspaceDetails[]> {
    const response = await this.apiRequest({
      method: "GET",
      apiPath: "workspaces",
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch workspaces: ${response.statusText}`);
    }

    const workspaces = await response.json();
    return workspaces.map((workspace: WorkspaceDetails) => ({
      instanceUrl: workspace.instanceUrl,
      displayName: workspace.displayName,
      id: workspace.id,
      instanceEnvType: workspace.instanceEnvType,
      baseUrl: getApiUrlForInstance(workspace.instanceUrl),
    }));
  }

  async getWorkspaceDetails(workspaceId: string): Promise<WorkspaceDetails> {
    const response = await this.apiRequest({
      method: "GET",
      apiPath: `workspaces/${workspaceId}`,
    });

    if (!response.ok) {
      throw new Error(
        `Failed to fetch workspace details: ${response.statusText}`,
      );
    }

    const { instanceUrl, displayName, id, instanceEnvType } =
      await response.json();

    if (!instanceUrl || !displayName || !id || !instanceEnvType) {
      throw new Error(
        "Failed to fetch workspace details: missing required fields",
      );
    }

    return {
      instanceUrl,
      displayName,
      id,
      instanceEnvType,
      baseUrl: getApiUrlForInstance(instanceUrl),
    };
  }
}

export const isTokenAboutToExpire = (
  token: string,
  graceMinutes: number = 60,
): boolean => {
  try {
    const userData = tryGetUserDataFromToken(token);

    if (!userData.exp) {
      return true;
    }

    const now = new Date();
    const expirationDate = userData.exp;
    const gracePeriodMs = graceMinutes * 60 * 1000;

    const timeUntilExpiration = expirationDate.getTime() - now.getTime();

    return timeUntilExpiration <= gracePeriodMs;
  } catch (_err) {
    return true;
  }
};
