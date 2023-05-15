import t from 'tap';
import { expect } from 'chai';

import { request } from './helpers/supertest-agents';
import { testSuiteAfter, testSuiteBefore } from './helpers/test-suite-hooks';
import { setSwallowErrors } from '../src/lib/logger';

t.before(testSuiteBefore);
t.teardown(testSuiteAfter);

t.test('general routes and route handling', async () => {

  t.test('GET / - api status endpoint', async () => {
    await request.get('/')
      .expectOk()
      .expectBody({ systemStatus: 'ok' });
  });

  t.test('404 handling', async () => {
    await request.get('/bad-url')
      // check http code using string name (clearer than number), and optionally specific error code
      .expectError('NotFound', 'NoMatchingURL')
      // helper that looks for the string in body.message
      // should be used sparingly as we don't want to rely on error messages much
      .expectErrorMessageContains('No matching URL found');
  });

  t.test('500 error handling', async () => {
    // temporarily disable logging of unexpected errors
    setSwallowErrors(true);
    // endpoint only exists while running tests
    await request.get('/boom')
      // endpoint just throws an error, not an ApiError, with a specific code
      // so we ensure it is exposed as a 500 / InternalServerError
      .expectError('InternalServerError')
      .expect((res) => {
        // and check that the details are hidden from the user
        expect(res.body.message).not.to.contain('crash boom bang');
      });

    setSwallowErrors(false);
  });
});
