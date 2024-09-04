import JWT from "npm:jsonwebtoken";

export class SdfApiClient {
  readonly token: string;
  readonly baseUrl: string;
  readonly workspaceId: string;

  // We can't have async constructor so init() should be used to get a client instance
  private constructor(
    token: string,
    baseUrl: string,
    workspaceId: string
  ) {
    this.token = token;
    this.baseUrl = baseUrl;
    this.workspaceId = workspaceId;
  }

  /// Get a client to do web requests to sdf. if JWT_PRIVATE_KEY is set, the
  /// second argument should be the userId to be baked in the token.
  public static async init(
    workspaceId: string,
    userEmailOrId: string,
    password: string
  ) {
    const token = await getSdfJWT(workspaceId, userEmailOrId, password);
    const baseUrl = Deno.env.get("SDF_API_URL");

    return new SdfApiClient(token, baseUrl, workspaceId);
  }

  async fetch(path: string, options?: {
    headers?: Record<string, string>
    body?: Record<string, string>
    method?: "GET" | "POST" | "PUT" | "DELETE" | "PATCH"
  }) {
    const resp = await this.fetch_no_throw(path, options);
    if (!resp.ok) {
      throw new Error(`Error ${resp.status}: ${await resp.text()}`);
    }

    return resp;
  }

  /// Don't automatically throw on errors
  fetch_no_throw(path: string, options?: {
    headers?: Record<string, string>
    body?: Record<string, string>
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
      headers, body, method: options?.method
    });
  }
}

async function getSdfJWT(workspaceId: string, userEmailOrId: string, password: string) {
  const privateKey = Deno.env.get("JWT_PRIVATE_KEY");
  if (privateKey && privateKey.length > 0) {
    console.log("JWT_PRIVATE_KEY is set, signing jwt locally. UserId  should be passed in instead of email");

    return createJWTFromPrivateKey(workspaceId, userEmailOrId, privateKey);
  } else {
    return getSdfJWTFromAuth0(workspaceId, userEmailOrId, password);
  }
}

async function getSdfJWTFromAuth0(workspaceId: string, email: string, password: string): string {
  const auth_api_url = Deno.env.get("AUTH_API_URL");

  if (!auth_api_url || auth_api_url.length === 0) {
    throw new Error("Missing AUTH_API_URL");
  }

  const login_resp = await fetch(`${auth_api_url}/auth/login`, {
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

  const { token, message } = await login_resp.json();

  if (!token) {
    const error_message = message ?? "Unknown Error";
    throw Error(`Could not get token: ${error_message}`);
  }

  return token;
}

async function createJWTFromPrivateKey(
  workspaceId: String,
  userId: String,
  private_key: String
): string {
  return JWT.sign({
    user_pk: userId,
    workspace_pk: workspaceId
  }, private_key, { algorithm: "RS256", ...{ subject: userId } });
}


