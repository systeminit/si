describe("itemLoader", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
  });
});
