/* eslint-disable max-statements-per-line */
/* eslint-disable no-console */

import * as _ from 'lodash';
import { expect } from 'chai';

import * as supertest from 'supertest';
import { ErrorCodes, HttpErrorCodeNames } from '../../src/lib/api-error';

const Test = (supertest as any).Test;

type GenericObject = Record<string, unknown>;

// have to patch supertest types to add the new helper functions
// MAKE SURE TO KEEP THESE IN SYNC WITH THE IMPLEMENTATIONS BELOW!
declare module 'supertest' {
  interface Test {
    // spoofAuth(authAs: Models.User | Models.Admin | string): this;

    log(): this;
    logBody(): this;

    expectOk(): this;
    expectBody(expected: GenericObject): this;
    expectBodyContainsString(substringToSearchFor: string): this;
    saveBodyInObject(object: GenericObject, saveAsKey: string, pathToSave?: string | string[]): this;
    expectBodyContainsItemMatching(expected: GenericObject): this;
    expectBodyDoesNotContain(expectedToBeMissing: GenericObject): this;
    // expectValAtBodyPath (
    //   pathArray: string[],
    //   expectationFn: (expectedVal: jest.JestMatchers<any>) => void
    // ): this;

    expectError(generalCode: HttpErrorCodeNames, specificCode?: string): this;
    expectErrorMessageContains(partOfMessage: string): this;
    expectBodyDoesNotHaveKeys(keys: string[]): this;
  }
}

// // helper for spoofing auth
// // Test.prototype.spoofAuth = function spoofAuth(authAs: Models.User | Models.Admin | string) {
// //   return this.set('spoof-auth', _.isString(authAs) ? authAs : authAs.id);
// // };

// logging helpers - very useful while debugging tests
Test.prototype.log = function log() {
  return this.expect((res: any) => { console.log(res); });
};
Test.prototype.logBody = function logBody() {
  return this.expect((res: any) => { console.log(res.body); });
};

// helpers to check response ///////////////////////////////////////////////////////////////////////

Test.prototype.expectOk = function expectOk() {
  return this.expect(200);
};

// checks for http error code using our error code names
// and looks inside the response for a specific error code, if one is provided
Test.prototype.expectError = function expectError(genericErrorCode: HttpErrorCodeNames, specificErrorCode: string) {
  return this
    .expect(ErrorCodes[genericErrorCode]) // check http status code
    .expect((res: any) => {
      // if specific code is set, also check the error "type" in the body
      if (specificErrorCode) {
        expect(res.body.kind).to.eq(specificErrorCode);
      }
    });
};

Test.prototype.saveBodyInObject = function saveBody(
  object: any,
  saveAsKey: string,
  pathToSave?: string | string[],
) {
  return this.expect((res: any) => {
    if (res.statusCode === 200) {
      object[saveAsKey] = pathToSave ? _.get(res.body, pathToSave) : res.body;
    }
  });
};

Test.prototype.expectBody = function expectBody(expected: any) {
  return this.expect((res: any) => {
    expect(res.body).to.containSubset(expected);
  });
};
Test.prototype.expectErrorMessageContains = function expectErrorMessageContains(substring: string) {
  return this.expect((res: any) => {
    if (res.statusCode < 400) {
      throw new Error('Expected an http status code >= 400');
    }
    if (!res.body.message.includes(substring)) {
      throw new Error([
        'Expected response error message',
        `("${res.body.message}")`,
        `to contain string "${substring}"`,
      ].join('\n'));
    }
  });
};
Test.prototype.expectBodyContainsString = function expectBodyContainsString(substring: string) {
  return this.expect((res: any) => {
    if (!JSON.stringify(res.body).includes(substring)) {
      throw new Error(`Expected response body to contain string "${substring}"`);
    }
  });
};

Test.prototype.expectBodyContainsItemMatching = function expectBodyContainsItemMatching(expected: any) {
  return this.expect((res: any) => {
    if (_.isArray(expected)) { // handle arrays
      _.each(expected, (item) => expect(res.body).to.containSubset(item));
    } else { // handle objects
      expect(res.body).to.containSubset(expected);
    }
  });
};

Test.prototype.expectBodyDoesNotContain = function expectBodyDoesNotContain(expected: any) {
  return this.expect((res: any) => {
    if (_.isArray(expected)) { // handle arrays
      _.each(expected, (item) => expect(res.body).not.to.containSubset(item));
    } else { // handle objects
      expect(res.body).not.to.containSubset(expected);
    }
  });
};

// Test.prototype.expectValAtBodyPath = function expectValAtBodyPath(
//   pathArray: string[],
//   expectationFn: (expectedVal: jest.JestMatchers<any>) => void
// ) {
//   return this.expect((res: any) => {
//     const val = _.get(res.body, pathArray);
//     expectationFn(expect(val));
//   });
// };

Test.prototype.expectBodyDoesNotHaveKeys = function expectBodyDoesNotHaveKeys(
  keys: string[],
) {
  return this.expect((res: any) => {
    _.each(keys, (key) => {
      expect(res.body).not.to.haveOwnProperty(key);
    });
  });
};
