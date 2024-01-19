describe("Login", () => {
  beforeEach(() => {
    cy.visit("/");
  });

  it("lets the user log in", () => {
    cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);
    cy.visit("/");
    // check that you're on head
    cy.url().should("contain", "head");
  });

});
