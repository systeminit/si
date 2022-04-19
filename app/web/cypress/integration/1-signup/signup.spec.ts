/// <reference types="cypress" />

describe("Signup", () => {
  beforeEach(() => {
    cy.visit("authenticate/signup");
  });

  it("lets the user create a new account", () => {
    cy.getBySel("billingAccountName").type("a");
    cy.getBySel("userName").type("a");
    cy.getBySel("userEmail").type("a");
    cy.getBySel("userPassword").type("a");
    cy.getBySel("signupSecret").type("cool-steam");
    cy.getBySel("signUp").click();
    cy.url().should("be.match", /\/authenticate\/login$/);
  });
});
