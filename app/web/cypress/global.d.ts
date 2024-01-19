/// <reference types="cypress" />

declare namespace Cypress {

  interface CustomWindow extends Window { }

  interface Chainable {
    /**
     * Window object with additional properties used in testing; exposes
     * services!
     */
    window(options?: Partial<Loggable & Timeoutable>): Chainable<CustomWindow>;

    getBySel(dataTestAttribute: string, args?: any): Chainable<JQuery<HTMLElement>>;
    getBySelLike(dataTestPrefixAttribute: string, args?: any): Chainable<JQuery<HTMLElement>>;

    /**
     * Logs in via Auth0 login page
     */
    loginToAuth0(username: string, password: string): Chainable<any>;

    dragTo(sourceElement: string, targetElement: string): void;
  }
}
