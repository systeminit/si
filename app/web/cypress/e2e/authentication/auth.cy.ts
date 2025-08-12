const SI_CYPRESS_MULTIPLIER = Cypress.env('VITE_SI_CYPRESS_MULTIPLIER') || import.meta.env.VITE_SI_CYPRESS_MULTIPLIER || 1;
const AUTH_PORTAL_URL = Cypress.env('VITE_AUTH_PORTAL_URL') || import.meta.env.VITE_AUTH_PORTAL_URL;
const SI_WORKSPACE_ID = Cypress.env('VITE_SI_WORKSPACE_ID') || import.meta.env.VITE_SI_WORKSPACE_ID;

Cypress._.times(SI_CYPRESS_MULTIPLIER, () => {
  describe("auth", () => {
    beforeEach(() => {
      cy.visit("/");
    });

    it("log_in", () => {
      cy.basicLogin();
      cy.appModelPageLoaded();
    });

    it("log_out", () => {
      cy.basicLogin();
      cy.appModelPageLoaded();
      
      // log out
      cy.get('[aria-label="Profile"]').should('exist').click();
      cy.get('.profile-dropdown-menu-logout').should('exist').should('be.visible').click({ force: true });

      // There is a bug currently where you log out of the product & it just logs you out to the workspaces page of the UI in auth portal
      cy.url().should("contain", AUTH_PORTAL_URL + '/workspaces');
    });
  });
});
