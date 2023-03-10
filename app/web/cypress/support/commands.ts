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

import { Workspace } from "../../src/api/sdf/dal/workspace";
import { User } from "../../src/api/sdf/dal/user";
import { PublicKey } from "../../src/api/sdf/dal/key_pair";
import Bottle from "bottlejs";
import { user$ } from "../../src/observable/user";
import { SDF } from "../../src/api/sdf";
import { SessionService } from "../../src/service/session";
import faker from "faker";
import { CreateAccountRequest } from "../../src/service/signup/create_account";
import { firstValueFrom } from "rxjs";

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
    .window({ log: false })
    .its("SignupService", { log: false })
    .then((signup) => {
      const workspaceName = faker.company.companyName();
      const userName = faker.name.findName();
      const userEmail = faker.internet.email();
      const userPassword = "snakes";
      const signupSecret = "cool-steam";
      const request = {
        workspaceName,
        userName,
        userEmail,
        userPassword,
        signupSecret,
      };
      cy.wrap(request).as("nw");
      return firstValueFrom(signup.createAccount(request));
    })
    .then((response) => {
      log.set({
        consoleProps() {
          return response;
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

  cy.signup();
  return cy.get<CreateAccountRequest>("@nw").then((ctx) => {
    return cy
      .window()
      .its("SessionService")
      .then((SessionService) => {
        return firstValueFrom(
          SessionService.login({
            workspaceName: ctx.workspaceName,
            userEmail: ctx.userEmail,
            userPassword: ctx.userPassword,
          }),
        );
      })
      .then((response) => {
        log.set({
          consoleProps() {
            return response;
          },
        });
        log.end();
      });
  });
});

export interface TestCtx {
  workspace: Workspace;
  workspace: Workspace;
  user: User;
  key_pair: PublicKey;
}
