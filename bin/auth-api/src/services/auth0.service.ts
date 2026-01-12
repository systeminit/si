/* eslint-disable no-console */
import Axios from "axios";
import { nanoid } from "nanoid";
import { ManagementClient } from "auth0";
import JWT from "jsonwebtoken";

import { ApiError } from "../lib/api-error";
import { tryCatch } from "../lib/try-catch";
import { getQueryString } from "../lib/querystring";
import { getCache, setCache } from "../lib/cache";

// const auth0Client = new AuthenticationClient({
//   /* eslint-disable @typescript-eslint/no-non-null-assertion */
//   domain: process.env.AUTH0_DOMAIN!,
//   clientId: process.env.AUTH0_CLIENT_ID!,
// });

const LOGIN_CALLBACK_URL = `${process.env.AUTH_API_URL}/auth/login-callback`;
const LOGOUT_CALLBACK_URL = `${process.env.AUTH_API_URL}/auth/logout-callback`;

const auth0Api = Axios.create({
  baseURL: `https://${process.env.AUTH0_DOMAIN}`,
  timeout: 30000, // 30 second timeout
});

// Add request/response interceptors for timing
auth0Api.interceptors.request.use((config) => {
  (config as any).metadata = { startTime: Date.now() };
  return config;
});

auth0Api.interceptors.response.use(
  (response) => {
    const duration_ms = Date.now() - (response.config as any).metadata.startTime;
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level: "info",
      type: "auth0",
      method: response.config.method?.toUpperCase(),
      url: response.config.url,
      status: response.status,
      duration_ms,
      ...(duration_ms > 5000 && { slowCall: true }),
    }));
    return response;
  },
  (error) => {
    const startTime = (error.config as any)?.metadata?.startTime;
    const duration_ms = startTime ? Date.now() - startTime : null;
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level: "error",
      type: "auth0",
      method: error.config?.method?.toUpperCase(),
      url: error.config?.url,
      status: error.response?.status,
      duration_ms,
      ...(duration_ms && duration_ms > 5000 && { slowCall: true }),
      error_code: error.code,
      error_message: error.message,
      ...(error.response?.data && { response_body: error.response.data }),
    }));
    throw error;
  },
);

export async function getAuth0UserCredential(username: string, password: string) {
  const authResult = await auth0Api.request({
    method: "post",
    url: "/oauth/token",
    headers: { "content-type": "application/json" },
    data: JSON.stringify({
      grant_type: "password",
      username,
      password,
      client_id: process.env.AUTH0_CLIENT_ID,
      client_secret: process.env.AUTH0_CLIENT_SECRET,
      audience: `https://${process.env.AUTH0_DOMAIN}/api/v2/`,
    }),
  });

  const token_raw = authResult.data?.access_token;
  if (!token_raw) {
    throw Error("Bad User");
  }

  // Only allow login with this method if the account has the "Test User" role
  const token = JWT.decode(token_raw);
  if (!token || typeof token !== "object") {
    throw Error("Bad Token Format");
  }

  const user_roles = token?.["https://systeminit.com/roles"] ?? [];

  if (typeof user_roles !== "object" || !user_roles.includes("Test User")) {
    throw Error("Non 'Test User' account");
  }

  return token_raw;
}

export function getAuth0LoginUrl(signup = false) {
  // lots of ways to generate this, but... nanoid is pretty good and already url-safe
  const randomState = nanoid(16);

  const loginParams = getQueryString({
    response_type: "code", // or 'token'
    client_id: process.env.AUTH0_CLIENT_ID,
    redirect_uri: LOGIN_CALLBACK_URL,
    state: randomState,
    scope: "openid profile email",
    ...signup && { screen_hint: "signup" },
    // `connection=CONNECTION` // not quite sure
    // prompt=none // for silent authentication - https://auth0.com/docs/authenticate/login/configure-silent-authentication
    // audience -- can be used to specify which "api" we are connecting to?
    // invitation -- used for auth0's native orgs / invite system
    // ADDITIONAL_PARAMETERS - anything else will be sent to the provider
    // access_type=offline // for google
  });

  return {
    randomState,
    url: `https://${process.env.AUTH0_DOMAIN}/authorize?${loginParams}`,
  };
}

export function getAuth0LogoutUrl() {
  const logoutParams = getQueryString({
    client_id: process.env.AUTH0_CLIENT_ID,
    returnTo: LOGOUT_CALLBACK_URL,
  });
  return `https://${process.env.AUTH0_DOMAIN}/v2/logout?${logoutParams}`;
}

export async function completeAuth0TokenExchange(code: string) {
  const tokenAndBasicProfile = await tryCatch(async () => {
    const tokenReq = await auth0Api.post(
      "/oauth/token",
      getQueryString({
        grant_type: "authorization_code",
        client_id: process.env.AUTH0_CLIENT_ID,
        client_secret: process.env.AUTH0_CLIENT_SECRET,
        code,
        redirect_uri: LOGIN_CALLBACK_URL,
      }),
      {
        headers: { "content-type": "application/x-www-form-urlencoded" },
      },
    );
    return {
      accessToken: tokenReq.data.access_token as string,
      idTokenData: JWT.decode(tokenReq.data.id_token) as any,
    };
  }, (err) => {
    // if err is an http error from auth0, it will usually look something like:

    // err.response.data.error -- ex: 'invalid_grant'
    // err.response.data.error_description -- ex: 'Invalid authorization code'

    // if the error doesn't look like a normal auth0 response just throw it
    if (!err?.response.data.error_description) throw err;
    throw new ApiError(
      "Conflict",
      "AUTH0_EXCHANGE_FAILURE",
      err.response.data.error_description,
    );
  });
  // access token is an "opaque token" so does not contain any info and cannot be decoded
  // but id_token data is decoded into idTokenData and includes some basic info

  const { accessToken, idTokenData } = tokenAndBasicProfile!;
  const profile = await fetchAuth0Profile(idTokenData.sub);

  return { profile, token: accessToken };
}

const AUTH0_MANAGEMENT_TOKEN_REDIS_KEY = "auth0-management-api-key";
type SavedAuth0TokenData = {
  clientId: string,
  token: string,
};

async function getManagementApiToken() {
  // first check if we have a valid token in redis (and make sure the client id has not changed)
  const savedTokenInfo = await getCache<SavedAuth0TokenData>(AUTH0_MANAGEMENT_TOKEN_REDIS_KEY);
  if (savedTokenInfo?.clientId === process.env.AUTH0_M2M_CLIENT_ID) {
    return savedTokenInfo?.token;
  }

  // otherwise we'll generate a new token, save it in redis for future requests, and return it

  // TODO: with actual volume, we'd want to acquire some kind of mutex so only one running instance is refreshing the token at a time
  // but since nothing bad happens if we refresh the token multiple times and we have very low volume, we can ignore safely ignore for now

  const result = await auth0Api.request({
    method: "post",
    url: "/oauth/token",
    headers: { "content-type": "application/json" },
    data: JSON.stringify({
      client_id: process.env.AUTH0_M2M_CLIENT_ID,
      client_secret: process.env.AUTH0_M2M_CLIENT_SECRET,
      audience: `https://${process.env.AUTH0_DOMAIN}/api/v2/`,
      grant_type: "client_credentials",
    }),
  });
  const token = result.data.access_token;

  await setCache(AUTH0_MANAGEMENT_TOKEN_REDIS_KEY, {
    clientId: process.env.AUTH0_M2M_CLIENT_ID,
    token,
  }, {
    expiresIn: result.data.expires_in - (5 * 60),
  });

  return token;
}

export async function setManagementApiTokenForTesting() {
  if (process.env.NODE_ENV !== "test") {
    throw new Error("This should only be used in test mode...");
  }
  await setCache(AUTH0_MANAGEMENT_TOKEN_REDIS_KEY, {
    clientId: process.env.AUTH0_M2M_CLIENT_ID!,
    token: "mocktoken",
  });
}

async function getManagementClient() {
  const m2mToken = await getManagementApiToken();
  /* eslint-disable @typescript-eslint/no-non-null-assertion */
  return new ManagementClient({
    domain: process.env.AUTH0_DOMAIN!,
    // clientId: process.env.AUTH0_M2M_CLIENT_ID!,
    token: m2mToken,
  });
}

export async function fetchAuth0Profile(auth0Id: string) {
  const auth0ManagementClient = await getManagementClient();

  const profile = await tryCatch(async () => {
    return await auth0ManagementClient.users.get({ id: auth0Id });
  }, (err) => {
    if (!err?.response.data.error_description) throw err;
    throw new ApiError(
      "Conflict",
      "Auth0ProfileError",
      err.response.data.error_description,
    );
  });
  if (!profile) throw new Error("no profile"); // just for TS

  return profile;
}

export async function resendAuth0EmailVerification(auth0Id: string) {
  const auth0ManagementClient = await getManagementClient();
  await auth0ManagementClient.jobs.verifyEmail({ user_id: auth0Id });
}
