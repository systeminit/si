// @ts-check
///<reference path="../../global.d.ts"/>

const AUTH0_USERNAME = Cypress.env('VITE_AUTH0_USERNAME') || import.meta.env.VITE_AUTH0_USERNAME;
const AUTH0_PASSWORD = Cypress.env('VITE_AUTH0_PASSWORD') || import.meta.env.VITE_AUTH0_PASSWORD;
const AUTH_PORTAL_URL = Cypress.env('VITE_AUTH_PORTAL_URL') || import.meta.env.VITE_AUTH_PORTAL_URL;
const AUTH_API_URL = Cypress.env('VITE_AUTH_API_URL') || import.meta.env.VITE_AUTH_API_URL;
const SI_WORKSPACE_ID = Cypress.env('VITE_SI_WORKSPACE_ID') || import.meta.env.VITE_SI_WORKSPACE_ID;
const UUID = Cypress.env('VITE_UUID') || import.meta.env.VITE_UUID || "local";

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

    cy.wait(5000);

    // Check that it loaded the workspace + prompted to create a new changeset
    cy.contains('Create change set', { timeout: 10000 }).should('be.visible');
  });
});
