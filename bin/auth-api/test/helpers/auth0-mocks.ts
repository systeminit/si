import nock from 'nock';

const AUTH0_API_URL = `https://${process.env.AUTH0_DOMAIN}`;

type AuthProviders = 'google' | 'github' | 'password';

const DEFAULT_PROFILE_DATA_BY_PROVIDER = {
  google: {
    sub: 'google-oauth2|108933039912882011657',
    given_name: 'Theo',
    family_name: 'Ephraim',
    nickname: 'theo',
    name: 'Theo Ephraim',
    picture: 'https://lh3.googleusercontent.com/a/AGNmyxZ1h4N1UM02p5F938k8qIA_G16oGtLdg8W7T647=s96-c',
    locale: 'en',
    updated_at: '2023-03-09T06:28:20.845Z',
    email: 'theo@systeminit.com',
    email_verified: true,

  },
  github: {
    sub: 'github|1158956',
    nickname: 'theoephraim',
    name: 'Theo Ephraim',
    picture: 'https://avatars.githubusercontent.com/u/1158956?v=4',
    updated_at: '2023-03-09T06:27:35.133Z',
    email: 'theozero@gmail.com',
    email_verified: true,
  },
  password: {
    sub: 'auth0|64097e57b62dadad87ac788a',
    nickname: 'theozero',
    name: 'theozero@gmail.com',
    picture: 'https://s.gravatar.com/avatar/ff33e7ba3ea72a853ed7d668439d4639?s=480&r=pg&d=https%3A%2F%2Fcdn.auth0.com%2Favatars%2Fth.png',
    updated_at: '2023-03-09T06:36:07.055Z',
    email: 'theozero@gmail.com',
    email_verified: false,
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
  nock(AUTH0_API_URL)
    .post('/oauth/token')
    .reply(options?.exchangeErrorMessage ? 400 : 200, {
      ...options?.exchangeErrorMessage ? {
        error_description: options.exchangeErrorMessage,
      } : {
        access_token: `auth0-fake-token-${+new Date()}`,
      },
    });

  if (options?.exchangeErrorMessage) return;

  nock(AUTH0_API_URL)
    .get('/userinfo')
    .reply(options?.profileErrorMessage ? 400 : 200, {
      ...options?.profileErrorMessage ? {
        error_description: options.profileErrorMessage,
      } : getAuth0ProfileData(options?.provider, options?.profileOverrides),
    });
}
