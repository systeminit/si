import nock from 'nock';
import JWT from "jsonwebtoken";
import { AuthProviders } from '../../src/services/auth.service';

const AUTH0_API_URL = `https://${process.env.AUTH0_DOMAIN}`;

const DEFAULT_PROFILE_DATA = {
  user_id: "google-oauth2|108933039912882011657",
  nickname: "theo",
  name: "Theo Ephraim",
  picture: "https://lh3.googleusercontent.com/a/ACg8ocJ5QCA3AXLpqO99Lk-wraGAKXKv17371hnLPqxyTz4W=s96-c",
  email: "theo@systeminit.com",
  locale: "en",
  last_ip: "2001:569:72b2:7e00:82:d5f9:6e60:72ae",
  created_at: "2023-03-14T22:02:45.649Z",
  given_name: "Theo",
  identities: [
    {
      user_id: "108933039912882011657",
      isSocial: true,
      provider: "google-oauth2",
      connection: "google-oauth2",
      expires_in: 3599,
      access_token: "ya29.REDACTED",
    },
  ],
  last_login: "2023-11-23T17:55:56.036Z",
  updated_at: "2023-11-23T17:55:56.036Z",
  family_name: "Ephraim",
  logins_count: 181,
  email_verified: true,
};

const DEFAULT_PROFILE_DATA_BY_PROVIDER = {
  google: {
    ...DEFAULT_PROFILE_DATA,
    user_id: "google-oauth2|108933039912882011657",
  },
  github: {
    ...DEFAULT_PROFILE_DATA,
    sub: 'github|1158956',
    nickname: 'githubuser',
    name: 'Github User',
    picture: 'https://avatars.githubusercontent.com/u/1158956?v=4',
    email: 'githubuser@example.dev',
  },
  password: {
    ...DEFAULT_PROFILE_DATA,
    sub: 'auth0|64097e57b62dadad87ac788a',
    nickname: 'passworduser',
    name: 'passworduser@example.com',
    picture: 'https://s.gravatar.com/avatar/ff33e7ba3ea72a853ed7d668439d4639?s=480&r=pg&d=https%3A%2F%2Fcdn.auth0.com%2Favatars%2Fth.png',
    email: 'passworduser@example.com',
  },
};

function getAuth0ProfileData(provider?: AuthProviders, overrides?: any) {
  return {
    ...DEFAULT_PROFILE_DATA_BY_PROVIDER[provider || 'google'],
    ...overrides,
  };
}

export function mockAuth0TokenExchange(
  options?: {
    exchangeErrorMessage?: string,
    profileErrorMessage?: string,
    provider?: AuthProviders,
    profileOverrides?: any,
  },
) {
  const profileData = getAuth0ProfileData(options?.provider, options?.profileOverrides);

  nock(AUTH0_API_URL)
    .post('/oauth/token')
    .reply(options?.exchangeErrorMessage ? 400 : 200, {
      ...options?.exchangeErrorMessage ? {
        error_description: options.exchangeErrorMessage,
      } : {
        access_token: `auth0-fake-token-${+new Date()}`,
        id_token: JWT.sign({
          sub: profileData.user_id,
        }, 'asdf'), // we will decode but not verify this token so secret doesnt matter
      },
    });

  if (options?.exchangeErrorMessage) return;

  // NOTE - another request is made to get a token for the management api
  // but we avoid it by filling in a fake token in redis

  nock(AUTH0_API_URL)
    .get(`/api/v2/users/${profileData.user_id.replace('|', '%7C')}`)
    .reply(options?.profileErrorMessage ? 400 : 200, {
      ...options?.profileErrorMessage ? {
        error_description: options.profileErrorMessage,
      } : profileData,
    });
}
