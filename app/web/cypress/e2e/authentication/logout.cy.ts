Cypress._.times(import.meta.env.VITE_SI_CYPRESS_MULTIPLIER ? import.meta.env.VITE_SI_CYPRESS_MULTIPLIER : 1, () => {
  describe("Logout", () => {
    beforeEach(() => {
      cy.visit("/");
      cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);
    });

    it("lets the user log out", () => {
      cy.visit("/");
      cy.contains('Create change set', { timeout: 10000 }).should('be.visible').click();
      cy.get('[aria-label="Profile"]').should('exist').click();
      cy.get('#dropdown-menu-item-1').should('exist').should('be.visible').click({ force: true });

      // There is a bug currently where you log out of the product & it just logs you out to the dashboard page
      // of the UI in auth portal
      cy.url().should("contain", import.meta.env.VITE_AUTH_PORTAL_URL + '/dashboard');

    });
  });
});