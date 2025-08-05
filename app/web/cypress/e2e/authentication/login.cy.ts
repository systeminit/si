const SI_CYPRESS_MULTIPLIER = Cypress.env('VITE_SI_CYPRESS_MULTIPLIER') || import.meta.env.VITE_SI_CYPRESS_MULTIPLIER || 1;
const AUTH0_USERNAME = Cypress.env('VITE_AUTH0_USERNAME') || import.meta.env.VITE_AUTH0_USERNAME;
const AUTH0_PASSWORD = Cypress.env('VITE_AUTH0_PASSWORD') || import.meta.env.VITE_AUTH0_PASSWORD;
const AUTH_API_URL = Cypress.env('VITE_AUTH_API_URL') || import.meta.env.VITE_AUTH_API_URL;
const SI_WORKSPACE_ID = Cypress.env('VITE_SI_WORKSPACE_ID') || import.meta.env.VITE_SI_WORKSPACE_ID;
const UUID = Cypress.env('VITE_UUID') || import.meta.env.VITE_UUID || "local";

Cypress._.times(SI_CYPRESS_MULTIPLIER, () => {
  describe("login", () => {
    beforeEach(() => {
      cy.visit("/");
    });

    it("log_in", () => {
      cy.loginToAuth0(AUTH0_USERNAME, AUTH0_PASSWORD);
      cy.visit({
        url:AUTH_API_URL + '/workspaces/' + SI_WORKSPACE_ID + '/go',
        failOnStatusCode: false
      });
      cy.on('uncaught:exception', (e) => {
        console.log(e);
        return false;
      });
      cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", UUID);
      // check that you're on head i.e. that you were redirected correctly
      cy.wait(4000);
      cy.url().should("contain", SI_WORKSPACE_ID);
    });

  });
});
