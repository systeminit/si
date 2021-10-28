import { TestCtx } from "../../support/commands";

describe("Login", () => {
  beforeEach(() => {
    cy.signup();
  });

  it("lets the user log in", () => {
    cy.visit("authenticate/login");
    cy.get("@testCtx").then((testCtx: any) => {
      testCtx = testCtx as TestCtx;
      cy.getBySel("billingAccountName").type(testCtx.billing_account.name);
      cy.getBySel("userEmail").type(testCtx.user.email);
      cy.getBySel("password").type("snakes");
      cy.getBySel("login")
        .click()
        .should(() => {
          expect(localStorage.getItem("si-sdf-token")).to.not.be.null;
        });
      cy.url().should("be.equal", `${Cypress.config("baseUrl")}/`);
    });
  });
});
