// @ts-check
///<reference path="../global.d.ts"/>

describe("workspace", () => {
  beforeEach(() => {
    cy.visit("/");
  });

  it("dashboard_redirect", () => {
    cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);

    // Go to the Synthetic User's Dashboard
    cy.visit(import.meta.env.VITE_AUTH_PORTAL_URL + '/dashboard')
    cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", import.meta.env.VITE_UUID ? import.meta.env.VITE_UUID: "local");

    cy.wait(5000)

    // Find the URL for the synthetic workspace and go there
    cy.get('a[href="' + import.meta.env.VITE_AUTH_API_URL + '/workspaces/' + import.meta.env.VITE_SI_WORKSPACE_ID + '/go"]')
       .should('be.visible')
       .invoke('removeAttr', 'target')
       .click();

    cy.wait(5000)

    // Check that it loaded the workspace + prompted to create a new changeset
    cy.contains('Create change set', { timeout: 10000 }).should('be.visible');

  });

});
