import * as _ from 'lodash';

import supertest from 'supertest';

// add some more functionality to supertest
import './extend-supertest';

import { app } from '../../src/main';

const appCallback = app.callback();

type HttpVerb = 'get' | 'put' | 'post' | 'patch' | 'del';

// supertest wrapper that lets us set default values, which we use to easily make requests as admin
class SupertestAgentWithDefaults {
  constructor(public defaults: any) {}

  makeRequest(verb: HttpVerb, url: string) {
    const agent = supertest(this.defaults.app || this.defaults.url || null)[verb](url);
    // set default headers
    if (this.defaults.headers) {
      _.each(this.defaults.headers, (val, key) => agent.set(key, val));
    }
    // set default auth (as in basic http auth)
    if (this.defaults.auth) {
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      agent.auth(this.defaults.auth.user, this.defaults.auth.password);
    }
    return agent;
  }

  get(url: string) { return this.makeRequest('get', url); }
  put(url: string) { return this.makeRequest('put', url); }
  patch(url: string) { return this.makeRequest('patch', url); }
  post(url: string) { return this.makeRequest('post', url); }
  del(url: string) { return this.makeRequest('del', url); }
}

export const request = supertest(appCallback);

export const adminRequest = new SupertestAgentWithDefaults({
  app: appCallback,
  headers: { 'spoof-auth': 'admin' },
});

export const superadminRequest = new SupertestAgentWithDefaults({
  app: appCallback,
  headers: { 'spoof-auth': 'superadmin' },
});
