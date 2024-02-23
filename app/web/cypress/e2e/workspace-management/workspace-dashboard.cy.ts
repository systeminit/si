// @ts-check
///<reference path="../global.d.ts"/>

describe("Check Workspace Dashboard", () => {
  beforeEach(() => {
    cy.visit("/");
    cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", import.meta.env.VITE_UUID ? import.meta.env.VITE_UUID: "local");
  });

  it("lets the user go to their dashboard and click into a workspace", () => {
    cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);

    // Go to the Synthetic User's Dashboard
    cy.visit(import.meta.env.VITE_AUTH_PORTAL_URL + '/dashboard')

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
