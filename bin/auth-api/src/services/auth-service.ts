import Url from "url";
import Axios from "axios";
import { nanoid } from "nanoid";
import { ApiError } from "../lib/api-error";

const AUTH_CALLBACK_URL = `${process.env.AUTH_API_URL}/auth/auth-callback`;

const auth0Api = Axios.create({
  baseURL: `https://${process.env.AUTH0_DOMAIN}`,
});

function getQueryString(obj: Record<string, any>) {
  return new Url.URLSearchParams(obj).toString();
}

function getAuth0LoginUrl() {
  // lots of ways to generate this, but... nanoid is pretty good and already url-safe
  const randomState = nanoid(16);

  const loginParams = getQueryString({
    response_type: "code", // or 'token'
    client_id: process.env.AUTH0_CLIENT_ID,
    redirect_uri: AUTH_CALLBACK_URL,
    state: randomState,
    scope: "openid profile email",
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

function getAuth0LogoutUrl() {
  const logoutParams = getQueryString({
    client_id: process.env.AUTH0_CLIENT_ID,
    returnTo: `${process.env.AUTH_API_URL}/logout-callback`,
  });
  return `https://${process.env.AUTH0_DOMAIN}/v2/logout?${logoutParams}`;
}

async function completeAuth0TokenExchange(code: string) {
  let token: string;
  try {
    const tokenReq = await auth0Api.post(
      "/oauth/token",
      getQueryString({
        grant_type: "authorization_code",
        client_id: process.env.AUTH0_CLIENT_ID,
        client_secret: process.env.AUTH0_CLIENT_SECRET,
        code,
        redirect_uri: AUTH_CALLBACK_URL,
      }),
      {
        headers: { "content-type": "application/x-www-form-urlencoded" },
      },
    );
    console.log(tokenReq.data);
    token = tokenReq.data.access_token;
  } catch (err) {
    // if err is an http error from auth0, it will usually look something like:
    // err.response.data.error -- ex: 'invalid_grant'
    // err.response.data.error_description -- ex: 'Invalid authorization code'

    // if the error doesn't look like a normal auth0 response just throw it
    if (!err.response.data.error_description) throw err;

    throw new ApiError(
      "Conflict",
      "AUTH0_EXCHANGE_FAILURE",
      err.response.data.error_description,
    );
  }

  // token is an "opaque token" so does not contain any info and cannot be decoded

  // but we can fetch profile data from auth0
  try {
    const profileReq = await Axios.get(
      `https://${process.env.AUTH0_DOMAIN}/userinfo`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      },
    );
    return { token, profile: profileReq.data };
  } catch (err) {
    if (!err.response.data.error_description) throw err;
    throw new ApiError(
      "Conflict",
      "AUTH0_PROFILE_FAILURE",
      err.response.data.error_description,
    );
  }
}

export default {
  getAuth0LoginUrl,
  getAuth0LogoutUrl,
  completeAuth0TokenExchange,
};
