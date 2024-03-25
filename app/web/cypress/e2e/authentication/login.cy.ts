Cypress._.times(import.meta.env.VITE_SI_CYPRESS_MULTIPLIER ? import.meta.env.VITE_SI_CYPRESS_MULTIPLIER : 1, () => {
  describe("login", () => {
    beforeEach(() => {
      cy.visit("/");
    });

    it("log_in", () => {
      cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);
      cy.visit(import.meta.env.VITE_AUTH_API_URL + '/workspaces/' + import.meta.env.VITE_SI_WORKSPACE_ID + '/go');
      cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", import.meta.env.VITE_UUID ? import.meta.env.VITE_UUID: "local");
      // check that you're on head i.e. that you were redirected correctly
      cy.wait(4000)
      cy.url().should("contain", import.meta.env.VITE_SI_WORKSPACE_ID);
    });

  });
});