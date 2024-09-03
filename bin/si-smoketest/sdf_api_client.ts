import JWT from "npm:jsonwebtoken";

export class SdfApiClient {
  readonly token: string;
  readonly base_url: string;

  // We can't have async constructor so init() should be used to get a client instance
  private constructor(
    token: string,
    base_url: string
  ) {
    this.token = token;
    this.base_url = base_url;
  }

  /// Get a client to do web requests to sdf. if JWT_PRIVATE_KEY is set, the
  /// second argument should be the userId to be baked in the token.
  public static async init(
    workspaceId: string,
    userEmailOrId: string,
    password: string
  ) {
    const token = await getSdfJWT(workspaceId, userEmailOrId, password);
    const base_url = Deno.env.get("SDF_API_URL");

    return new SdfApiClient(token, base_url);
  }

  fetch(path: string, options?: {
    headers?: Record<string, string>
    body?: Record<string, string>
    method?: "GET" | "POST" | "PUT" | "DELETE"
  }) {
    const url = `${this.base_url}${path}`;

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

  const { token } = await login_resp.json();

  if (!token) {
    throw Error("Could not get token");
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


