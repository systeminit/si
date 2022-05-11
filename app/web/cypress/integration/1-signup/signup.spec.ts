/// <reference types="cypress" />

describe("Signup", () => {
  beforeEach(() => {
    cy.visit("authenticate/signup");
  });

  it("lets the user create a new account", () => {
    cy.getBySel("billingAccountName").type("bobo");
    cy.getBySel("userName").type("bobo clown");
    cy.getBySel("userEmail").type("bobo@systeminit.com");
    cy.getBySel("userPassword").type("Bobo42!ggz");
    cy.getBySel("signupSecret").type("cool-steam");
    cy.getBySel("signUp").click();
    cy.url().should("be.match", /\/authenticate\/login$/);
  });
});
