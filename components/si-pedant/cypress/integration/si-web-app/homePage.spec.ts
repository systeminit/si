describe("The home page", () => {
  beforeEach(() => {
    cy.createUserBobo();
    cy.loginBobo();
  });

  it("successfully loads", () => {
    cy.visit("/");
  });
});
