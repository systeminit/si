import Axios from "axios";
import { nanoid } from "nanoid";
import Auth0 from 'auth0';

import { ApiError } from "../lib/api-error";
import { tryCatch } from "../lib/try-catch";
import { getQueryString } from "../lib/querystring";

const auth0Client = new Auth0.AuthenticationClient({
  /* eslint-disable @typescript-eslint/no-non-null-assertion */
  domain: process.env.AUTH0_DOMAIN!,
  clientId: process.env.AUTH0_CLIENT_ID!,
});

const LOGIN_CALLBACK_URL = `${process.env.AUTH_API_URL}/auth/login-callback`;
const LOGOUT_CALLBACK_URL = `${process.env.AUTH_API_URL}/auth/logout-callback`;

const auth0Api = Axios.create({
  baseURL: `https://${process.env.AUTH0_DOMAIN}`,
});

export type Auth0UserData = Auth0.UserData;

export function getAuth0LoginUrl() {
  // lots of ways to generate this, but... nanoid is pretty good and already url-safe
  const randomState = nanoid(16);

  const loginParams = getQueryString({
    response_type: "code", // or 'token'
    client_id: process.env.AUTH0_CLIENT_ID,
    redirect_uri: LOGIN_CALLBACK_URL,
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

export function getAuth0LogoutUrl() {
  const logoutParams = getQueryString({
    client_id: process.env.AUTH0_CLIENT_ID,
    returnTo: LOGOUT_CALLBACK_URL,
  });
  return `https://${process.env.AUTH0_DOMAIN}/v2/logout?${logoutParams}`;
}

export async function completeAuth0TokenExchange(code: string) {
  const token = await tryCatch(async () => {
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
    // console.log(tokenReq.data);
    return tokenReq.data.access_token as string;
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
  if (!token) throw new Error('no token'); // just for TS

  // token is an "opaque token" so does not contain any info and cannot be decoded

  // but we can fetch profile data from auth0
  const profile = await tryCatch(async () => {
    return await auth0Client.getProfile(token) as Auth0UserData;
    // const profileReq = await Axios.get(
    //   `https://${process.env.AUTH0_DOMAIN}/userinfo`,
    //   {
    //     headers: {
    //       Authorization: `Bearer ${token}x`,
    //     },
    //   },
    // );
    // return profileReq.data;
  }, (err) => {
    // console.log(err);
    if (!err?.response.data.error_description) throw err;
    throw new ApiError(
      "Conflict",
      "Auth0ProfileError",
      err.response.data.error_description,
    );
  });
  if (!profile) throw new Error('no profile'); // just for TS

  // GITHUB
  // {
  //   sub: 'github|1158956',
  //   nickname: 'theoephraim',
  //   name: 'Theo Ephraim',
  //   picture: 'https://avatars.githubusercontent.com/u/1158956?v=4',
  //   updated_at: '2023-03-09T06:27:35.133Z',
  //   email: 'theozero@gmail.com',
  //   email_verified: true
  // }

  // GOOGLE
  // {
  //   sub: 'google-oauth2|108933039912882011657',
  //   given_name: 'Theo',
  //   family_name: 'Ephraim',
  //   nickname: 'theo',
  //   name: 'Theo Ephraim',
  //   picture: 'https://lh3.googleusercontent.com/a/AGNmyxZ1h4N1UM02p5F938k8qIA_G16oGtLdg8W7T647=s96-c',
  //   locale: 'en',
  //   updated_at: '2023-03-09T06:28:20.845Z',
  //   email: 'theo@systeminit.com',
  //   email_verified: true
  // }

  // PASSWORD
  // {
  //   sub: 'auth0|64097e57b62dadad87ac788a',
  //   nickname: 'theozero',
  //   name: 'theozero@gmail.com',
  //   picture: 'https://s.gravatar.com/avatar/ff33e7ba3ea72a853ed7d668439d4639?s=480&r=pg&d=https%3A%2F%2Fcdn.auth0.com%2Favatars%2Fth.png',
  //   updated_at: '2023-03-09T06:36:07.055Z',
  //   email: 'theozero@gmail.com',
  //   email_verified: false
  // }

  return { profile, token };
}
