const SI_CYPRESS_MULTIPLIER = Cypress.env('VITE_SI_CYPRESS_MULTIPLIER') || import.meta.env.VITE_SI_CYPRESS_MULTIPLIER || 1;
const AUTH0_USERNAME = Cypress.env('VITE_AUTH0_USERNAME') || import.meta.env.VITE_AUTH0_USERNAME;
const AUTH0_PASSWORD = Cypress.env('VITE_AUTH0_PASSWORD') || import.meta.env.VITE_AUTH0_PASSWORD;
const AUTH_API_URL = Cypress.env('VITE_AUTH_API_URL') || import.meta.env.VITE_AUTH_API_URL;
const SI_WORKSPACE_ID = Cypress.env('VITE_SI_WORKSPACE_ID') || import.meta.env.VITE_SI_WORKSPACE_ID;
const UUID = Cypress.env('VITE_UUID') || import.meta.env.VITE_UUID || "local";
const AUTH_PORTAL_URL = Cypress.env('VITE_AUTH_PORTAL_URL') || import.meta.env.VITE_AUTH_PORTAL_URL;

Cypress._.times(SI_CYPRESS_MULTIPLIER, () => {
  describe("logout", () => {
    beforeEach(() => {
      cy.visit("/");
    });

    it("log_out", () => {
      cy.loginToAuth0(AUTH0_USERNAME, AUTH0_PASSWORD);
      cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", UUID);
      cy.contains('Create change set', { timeout: 10000 }).should('be.visible');
      cy.get('.modal-close-button').should('exist').click();
      cy.get('[aria-label="Profile"]').should('exist').click();
      cy.get('#dropdown-menu-item-1').should('exist').should('be.visible').click({ force: true });

      // There is a bug currently where you log out of the product & it just logs you out to the dashboard page
      // of the UI in auth portal
      cy.url().should("contain", AUTH_PORTAL_URL + '/dashboard');
    });
  });
});
