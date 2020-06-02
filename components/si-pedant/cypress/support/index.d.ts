// in cypress/support/index.d.ts
// load type definitions that come with Cypress module
/// <reference types="cypress" />

type QueryArgs = import("si-registry").QueryArgs;

interface GraphqlQueryArgs {
  typeName: string;
  queryArgs: QueryArgs;
  variables?: Record<string, any>;
}

interface CreateUserArgs {
  billingAccount: {
    name: string;
    displayName: string;
  };
  user: {
    name: string;
    displayName: string;
    email: string;
    password: string;
  };
}

declare namespace Cypress {
  interface Chainable {
    /**
     * Login to the system initiative, without routing through the webpage. Must faster, and memoized.
     * @example cy.login('boboCorp', "bobo@bobotclown.co", "bobotclown42")
     */
    login(
      billingAccountName: string,
      email: string,
      password: string,
    ): Chainable<Record<string, any>>;
    loginBobo(): Chainable<Record<string, any>>;

    createUser(args: CreateUserArgs): Chainable<Element>;
    createUserBobo(): Chainable<Element>;

    graphqlQuery(args: GraphqlQueryArgs): Chainable<Element>;
    graphqlMutation(args: GraphqlQueryArgs): Chainable<Element>;
  }
}
