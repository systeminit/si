describe("Sign in", () => {
  beforeEach(() => {
    cy.createUserBobo();
  });

  it("sets the auth cookie when logging in", () => {
    cy.visit("/signin");
    cy.get("[data-cy=billingAccountName]").type("boboCorp");
    cy.get("[data-cy=userEmail]").type("bobo@bobotclown.co");
    cy.get("[data-cy=userPassword]").type("bobotclown42");
    cy.get("[data-cy=signInButton]")
      .click()
      .should(() => {
        const apolloToken = localStorage.getItem("apollo-token");
        expect(apolloToken).to.exist;
      });
    cy.url().should("eq", `${Cypress.config().baseUrl}/`);
  });
});
