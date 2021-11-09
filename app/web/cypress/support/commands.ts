///<reference path="../global.d.ts">

// ***********************************************
// This example commands.js shows you how to
// create various custom commands and overwrite
// existing commands.
//
// For more comprehensive examples of custom
// commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************
//
//
// -- This is a parent command --
// Cypress.Commands.add('login', (email, password) => { ... })
//
//
// -- This is a child command --
// Cypress.Commands.add('drag', { prevSubject: 'element'}, (subject, options) => { ... })
//
//
// -- This is a dual command --
// Cypress.Commands.add('dismiss', { prevSubject: 'optional'}, (subject, options) => { ... })
//
//
// -- This will overwrite an existing command --
// Cypress.Commands.overwrite('visit', (originalFn, url, options) => { ... })

import { BillingAccount } from "../../src/api/sdf/dal/billing_account";
import { Organization } from "../../src/api/sdf/dal/organization";
import { Workspace } from "../../src/api/sdf/dal/workspace";
import { User } from "../../src/api/sdf/dal/user";
import { Group } from "../../src/api/sdf/dal/group";
import { PublicKey } from "../../src/api/sdf/dal/key_pair";
import Bottle from "bottlejs";
import { user$ } from "../../src/observable/user";
import { billingAccount$ } from "../../src/observable/billing_account";
import { SDF } from "../../src/api/sdf";
import { SessionService } from "../../src/api/sdf/service/session";

Cypress.Commands.add("getBySel", (selector, ...args) => {
  return cy.get(`[data-test=${selector}]`, ...args);
});

Cypress.Commands.add("getBySelLike", (selector, ...args) => {
  return cy.get(`[data-test*=${selector}]`, ...args);
});

Cypress.Commands.add("signup", () => {
  const log = Cypress.log({
    name: "signup",
    displayName: "signup",
    message: [`Signup new account`],
    // @ts-ignore
    autoEnd: false,
  });

  return cy
    .request({
      method: "POST",
      url: "/api/test/fixtures/signup",
      log: false,
    })
    .then((response) => {
      cy.wrap(response.body.data, { log: false }).as("testCtx");
      log.set({
        consoleProps() {
          return response.body.data;
        },
      });
      log.end();
    });
});

Cypress.Commands.add("signupAndLogin", () => {
  const log = Cypress.log({
    name: "signup",
    displayName: "signup and login",
    message: [`Signup new account`],
    // @ts-ignore
    autoEnd: false,
  });
  cy.visit("/", { log: false });

  return cy
    .request({
      method: "POST",
      url: "/api/test/fixtures/signup",
      log: false,
    })
    .then((response) => {
      cy.wrap(response.body.data, { log: false }).as("testCtx");
      log.set({
        consoleProps() {
          return response.body.data;
        },
      });
      log.end();
      cy.window({ log: false })
        .its("SessionService", { log: false })
        .then((s) => {
          return s.login({
            billingAccountName: response.body.data.billing_account.name,
            userEmail: response.body.data.user.email,
            userPassword: "snakes",
          });
        });
    });
});

export interface TestCtx {
  billing_account: BillingAccount;
  organization: Organization;
  workspace: Workspace;
  user: User;
  admin_group: Group;
  key_pair: PublicKey;
}
