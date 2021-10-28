/// <reference types="cypress" />

declare namespace Cypress {
  import { SignupService } from "../src/api/sdf/service/signup";
  import {
    CreateAccountRequest,
    CreateAccountResponse,
  } from "../src/api/sdf/service/signup/create_account";

  interface CustomWindow extends Window {
    SignupService: typeof SignupService;
  }

  interface Chainable {
    /**
     * Window object with additional properties used in testing; exposes
     * services!
     */
    window(options?: Partial<Loggable & Timeoutable>): Chainable<CustomWindow>;

    /**
     * Custom command to select DOM element by data-cy attribute.
     * @example cy.dataCy('greeting')
     */
    getBySel(value: string): Chainable<Element>;

    getBySelLike(value: string): Chainable<Element>;

    signup(): Chainable<Response<any>>;
  }
}
