import Axios from "axios";
import { nanoid } from "nanoid";
import Auth0 from 'auth0';
import JWT from "jsonwebtoken";

import { ApiError } from "../lib/api-error";
import { tryCatch } from "../lib/try-catch";
import { getQueryString } from "../lib/querystring";
import { getCache, setCache } from "../lib/cache";

// const auth0Client = new Auth0.AuthenticationClient({
//   /* eslint-disable @typescript-eslint/no-non-null-assertion */
//   domain: process.env.AUTH0_DOMAIN!,
//   clientId: process.env.AUTH0_CLIENT_ID!,
// });

const LOGIN_CALLBACK_URL = `${process.env.AUTH_API_URL}/auth/login-callback`;
const LOGOUT_CALLBACK_URL = `${process.env.AUTH_API_URL}/auth/logout-callback`;

const auth0Api = Axios.create({
  baseURL: `https://${process.env.AUTH0_DOMAIN}`,
});

export type Auth0UserData = Auth0.UserData;

export function getAuth0LoginUrl(signup = false) {
  // lots of ways to generate this, but... nanoid is pretty good and already url-safe
  const randomState = nanoid(16);

  const loginParams = getQueryString({
    response_type: "code", // or 'token'
    client_id: process.env.AUTH0_CLIENT_ID,
    redirect_uri: LOGIN_CALLBACK_URL,
    state: randomState,
    scope: "openid profile email",
    ...signup && { screen_hint: 'signup' },
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
const AUTH0_MANAGEMENT_TOKEN_REDIS_KEY = 'auth0-management-api-key';
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
    method: 'post',
    url: '/oauth/token',
    headers: { 'content-type': 'application/json' },
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
    // expire the key from redis 5 minutes before token expiration
    expiresIn: result.data.expires_in - (5 * 60),
  });

  return token;
}
export async function setManagementApiTokenForTesting() {
  if (process.env.NODE_ENV !== 'test') {
    throw new Error('This should only be used in test mode...');
  }
  await setCache(AUTH0_MANAGEMENT_TOKEN_REDIS_KEY, {
    clientId: process.env.AUTH0_M2M_CLIENT_ID!,
    token: 'mocktoken',
  });
}

async function getManagementClient() {
  const m2mToken = await getManagementApiToken();
  return new Auth0.ManagementClient({
    /* eslint-disable @typescript-eslint/no-non-null-assertion */
    domain: process.env.AUTH0_DOMAIN!,
    // clientId: process.env.AUTH0_M2M_CLIENT_ID!,
    token: m2mToken,
  });
}

export async function fetchAuth0Profile(auth0Id: string) {
  const auth0ManagementClient = await getManagementClient();

  const profile = await tryCatch(async () => {
    return await auth0ManagementClient.getUser({ id: auth0Id });
  }, (err) => {
    if (!err?.response.data.error_description) throw err;
    throw new ApiError(
      "Conflict",
      "Auth0ProfileError",
      err.response.data.error_description,
    );
  });
  if (!profile) throw new Error('no profile'); // just for TS

  return profile;
}

export async function resendAuth0EmailVerification(auth0Id: string) {
  const auth0ManagementClient = await getManagementClient();
  await auth0ManagementClient.sendEmailVerification({ user_id: auth0Id });
}
