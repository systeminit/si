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

import { login, loginBobo, logout } from "./login";
import { createUser, createUserBobo } from "./createUser";
import { graphqlQuery, graphqlMutation } from "./graphql";

Cypress.Commands.add("createUser", createUser);
Cypress.Commands.add("createUserBobo", createUserBobo);
Cypress.Commands.add("login", login);
Cypress.Commands.add("loginBobo", loginBobo);
Cypress.Commands.add("logout", logout);
Cypress.Commands.add("graphqlQuery", graphqlQuery);
Cypress.Commands.add("graphqlMutation", graphqlMutation);

//
//
// -- This is a child command --
// Cypress.Commands.add("drag", { prevSubject: 'element'}, (subject, options) => { ... })
//
//
// -- This is a dual command --
// Cypress.Commands.add("dismiss", { prevSubject: 'optional'}, (subject, options) => { ... })
//
//
// -- This will overwrite an existing command --
// Cypress.Commands.overwrite("visit", (originalFn, url, options) => { ... })
