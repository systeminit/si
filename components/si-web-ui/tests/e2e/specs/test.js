// https://docs.cypress.io/api/introduction/api.html

describe("Login Flow", () => {
  it("redirects to signin when not logged in", () => {
    cy.visit("/");
    cy.url().should("match", /signin/);
  });

  it("provides the login page", () => {
    cy.visit("/signin");
    cy.get("input[name=email]");
    cy.get("input[name=password]");
  });
});
