describe("Schema Route", () => {
  beforeEach(() => {
    cy.signupAndLogin();
  });

  it("can navigate to the schema editor", () => {
    cy.visit("/");
    cy.getBySel("schema-nav-link")
      .click()
      .then(() => {
        cy.url().should("be.equal", `${Cypress.config("baseUrl")}/schema`);
      });
  });
});
