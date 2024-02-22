Cypress._.times(import.meta.env.VITE_SI_CYPRESS_MULTIPLIER ? import.meta.env.VITE_SI_CYPRESS_MULTIPLIER : 1, () => {
  describe("Login", () => {
    beforeEach(() => {
      cy.visit("/");
    });

    it("lets the user log in", () => {
      cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);
      cy.visit("/");
      // check that you're on head i.e. that you were redirected correctly
      cy.url().should("contain", "head");
    });

  });
});