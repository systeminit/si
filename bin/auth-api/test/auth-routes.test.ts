import t from 'tap';
import { expect } from 'chai';
import _ from 'lodash';

import { request } from './helpers/supertest-agents';
import { testSuiteAfter, testSuiteBefore } from './helpers/test-suite-hooks';
import { mockAuth0TokenExchange } from './helpers/auth0-mocks';
import { verifyJWT } from '../src/lib/jwt';
import { decodeAuthToken } from '../src/services/auth.service';

t.before(testSuiteBefore);
t.teardown(testSuiteAfter);

t.test('Auth routes', async () => {

  t.test('GET /auth/login - begin login flow', async (t) => {
    await t.test('redirects to auth0', async () => {
      await request.get('/auth/login')
        .expect(302)
        .expect((res) => {
        // example redirect url
        // https://systeminit.auth0.com/authorize?response_type=code&client_id=XXX&redirect_uri=http%3A%2F%2Flocalhost%3A9001%2Fauth%2Flogin-callback&state=ZZZ&scope=openid+profile+email'
          const redirectUrl = res.headers.location;
          expect(redirectUrl.startsWith(`https://${process.env.AUTH0_DOMAIN}/authorize?`)).to.eq(true);
        });
    });
  });

  t.test('GET /auth/login-callback - auth0 login callback', async (t) => {
    let validState: string;

    t.test('(initiate login to get valid state)', async () => {
      await request.get('/auth/login')
        .expect(302)
        .expect((res) => {
          const redirectUrl = res.headers.location;
          // record the state value from our redirect url
          validState = redirectUrl.match(/state=([^&]+)/)[1];
        });
    });

    t.test(`works with a valid state`, async () => {
      mockAuth0TokenExchange();

      await request.get('/auth/login-callback')
        .query({
          code: 'mockedbutvalidcode',
          state: validState,
        })
        .expect(302)
        .expect(async (res) => {

          const setCookie = res.headers['set-cookie'][0];
          const [,authToken] = setCookie.match(/si-auth=([^;]+); path=\/; httponly/);

          const authData = await decodeAuthToken(authToken);
          expect(authData.userId).to.be.ok;

          expect(res.headers.location).to.eq(`${process.env.AUTH_PORTAL_URL}/login-success`);
        });
    });

    t.test(`fails if state is reused`, async () => {
      await request.get('/auth/login-callback')
        .query({
          code: 'mockedbutvalidcode',
          state: validState,
        })
        .expectError('Conflict');
    });

    _.each({
      'missing code': { code: undefined },
      'missing state': { state: undefined },
      // non-string values are treated as strings since they come in querystring
      // currently we do no other validation of if the values look like they are in the right format
    }, (queryOverride, description) => {
      t.test(`bad params - ${description}`, async () => {
        await request.get('/auth/login-callback')
          .query({
            code: 'somecode',
            state: 'somestate',
            ...queryOverride,
          })
          .expectError('BadRequest');
      });
    });

  });

});
