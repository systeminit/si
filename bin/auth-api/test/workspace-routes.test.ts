import t from 'tap';
import _ from 'lodash';
import { expect } from 'chai';

import { request } from './helpers/supertest-agents';
import { testSuiteAfter, testSuiteBefore } from './helpers/test-suite-hooks';
import { createDummyUser } from './helpers/dummy-factory';
import { decodeSdfAuthToken } from '../src/services/auth.service';

t.before(testSuiteBefore);
t.teardown(testSuiteAfter);

t.test('Workspace routes', async () => {
  const { user, workspace } = await createDummyUser();
  const { user: anotherUser } = await createDummyUser();

  t.test('GET /workspaces - load user workspaces', async (t) => {
    t.test('fails if user is not logged in', async () => {
      await request.get('/workspaces')
        .expectError('Unauthorized');
    });

    t.test('success', async () => {
      await request.get('/workspaces')
        .set('spoof-auth', user.id)
        .expectOk()
        .expect((res) => {
          expect(res.body).to.have.length(1);
          expect(res.body).to.containSubset([{
            id: workspace.id,
            creatorUserId: user.id,
          }]);
        });
    });
  });

  t.test('GET /workspaces/:workspaceId - load single workspace', async (t) => {
    t.test('fails if user is not logged in', async () => {
      await request.get(`/workspaces/${workspace.id}`)
        .expectError('Unauthorized');
    });

    t.test("fails if accessing another user's workspaces", async () => {
      await request.get(`/workspaces/${workspace.id}`)
        .set('spoof-auth', anotherUser.id)
        .expectError('Forbidden');
    });

    t.test('success', async () => {
      await request.get(`/workspaces/${workspace.id}`)
        .set('spoof-auth', user.id)
        .expectOk()
        .expectBody(workspace);
    });
  });

  // workspace login - uses 2 endpoints

  t.test('GET /workspaces/:workspaceId/go - begin workspace login flow', async (t) => {
    t.test('fails if user is not logged in', async () => {
      await request.get(`/workspaces/${workspace.id}/go`)
        .expectError('Unauthorized');
    });

    t.test("fails if accessing another user's workspaces", async () => {
      await request.get(`/workspaces/${workspace.id}/go`)
        .set('spoof-auth', anotherUser.id)
        .expectError('Forbidden');
    });

    t.test('success - redirects to the instance with a code', async () => {
      await request.get(`/workspaces/${workspace.id}/go`)
        .set('spoof-auth', user.id)
        .expect(302)
        .expect((res) => {
          const [, baseUrl] = res.headers.location.match(/(.+)\?code=.+/);
          expect(baseUrl).to.equal(`${workspace.instanceUrl}/auth-connect`);
        });
    });
  });

  t.test('POST /complete-auth-connect - finish workspace login flow', async (t) => {
    let validCode: string;

    t.before(async () => {
      await request.get(`/workspaces/${workspace.id}/go`)
        .set('spoof-auth', user.id)
        .expect(302)
        .expect((res) => {
          const [,code] = res.headers.location.match(/code=(.*)/);
          validCode = code;
        });
    });

    t.test('fails for a bad code', async () => {
      await request.post('/complete-auth-connect')
        .send({ code: 'not-valid-code' })
        .expectError('Conflict');
    });

    t.test('exchange code for token', async () => {
      await request.post('/complete-auth-connect')
        .send({ code: validCode })
        .expectOk()
        .expectBody({
          user: { id: user.id },
          workspace: { id: workspace.id },
        })
        .expect(async (res) => {
          const token = res.body.token;
          const decoded = await decodeSdfAuthToken(token);
          // check the token includes the user and workspace ids
          expect(decoded.user_pk).to.equal(user.id);
          expect(decoded.workspace_pk).to.equal(workspace.id);
        });
    });

    t.test('fails if code is re-used', async () => {
      await request.post('/complete-auth-connect')
        .send({ code: validCode })
        .expectError('Conflict');
    });

    t.test('fails if code is missing', async () => {
      await request.post('/complete-auth-connect')
        .expectError('BadRequest');
    });

    // check bad params
    _.each({
      'no code': {},
      'null code': { code: null },
      'non-string code': { code: 123 },
    }, (bodyOverride, description) => {
      t.test(`bad params - ${description}`, async () => {
        await request.post(`/complete-auth-connect`)
          .send({
            ...bodyOverride,
          })
          .expectError('BadRequest');
      });
    });

    // also didnt test code expiry, since we just rely on redis for this

    // untested, because we dont actually have deletion yet
    // - handling of deleted workspace
    // - handling of deleted user

  });

  // functionality not yet implemented, but placeholder route exists
  // t.test('PATCH /workspaces/:workspaceId - update single workspace'

});
