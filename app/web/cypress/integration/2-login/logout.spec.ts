describe("Logout", () => {
  beforeEach(() => {
    cy.signupAndLogin();
  });

  it("lets the user log out", () => {
    cy.visit("/");
    cy.get("@testCtx").then((testCtx: any) => {
      cy.getBySel("logout")
        .click()
        .should(() => {
          expect(localStorage.getItem("si-sdf-token")).to.be.null;
        });
      cy.url().should(
        "be.equal",
        `${Cypress.config("baseUrl")}/authenticate/login`,
      );
    });
  });
});
