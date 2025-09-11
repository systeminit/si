// @ts-check
///<reference path="../global.d.ts"/>

const SI_CYPRESS_MULTIPLIER = Cypress.env('VITE_SI_CYPRESS_MULTIPLIER') || import.meta.env.VITE_SI_CYPRESS_MULTIPLIER || 1;
const AUTH0_USERNAME = Cypress.env('VITE_AUTH0_USERNAME') || import.meta.env.VITE_AUTH0_USERNAME;
const AUTH0_PASSWORD = Cypress.env('VITE_AUTH0_PASSWORD') || import.meta.env.VITE_AUTH0_PASSWORD;
const AUTH_PORTAL_URL = Cypress.env('VITE_AUTH_PORTAL_URL') || import.meta.env.VITE_AUTH_PORTAL_URL;
const AUTH_API_URL = Cypress.env('VITE_AUTH_API_URL') || import.meta.env.VITE_AUTH_API_URL;
const SI_WORKSPACE_ID = Cypress.env('VITE_SI_WORKSPACE_ID') || import.meta.env.VITE_SI_WORKSPACE_ID;
const UUID = Cypress.env('VITE_UUID') || import.meta.env.VITE_UUID || "local";

Cypress._.times(SI_CYPRESS_MULTIPLIER, () => {
  describe("workspace", () => {
    beforeEach(() => {
      cy.visit("/");
    });

    it("dashboard_redirect", () => {

      cy.loginToAuth0(AUTH0_USERNAME, AUTH0_PASSWORD);

      // Go to the Synthetic User's Dashboard
      cy.visit(AUTH_PORTAL_URL + '/dashboard')
      cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", UUID);

      cy.wait(5000);

      // Find the URL for the synthetic workspace and go there
      cy.get('a[href="' + AUTH_API_URL + '/workspaces/' + SI_WORKSPACE_ID + '/go"]')
        .should('be.visible')
        .invoke('removeAttr', 'target')
        .click();
      cy.on('uncaught:exception', (e) => {
        console.log(e);
        return false;
      });

      // check to confirm that we have reached either the lobby or the app itself
      cy.get("#app-layout").should("exist", { timeout: 60000 });
      cy.url().should("contain", SI_WORKSPACE_ID, { timeout: 60000 });

      // checks for the new hotness UI Explore page
      // cy.appModelPageLoaded();
    });
  });
});