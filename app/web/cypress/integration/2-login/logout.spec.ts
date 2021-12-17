describe("Logout", () => {
  beforeEach(() => {
    cy.visit("/");
    cy.signupAndLogin();
  });

  it("lets the user log out", () => {
    cy.visit("/");
    cy.get("@nba").then((testCtx: any) => {
      cy.getBySel("logout")
        .click()
        .should(() => {
          expect(localStorage.getItem("si-sdf-token")).to.be.null;
        });
      cy.url().should("be.match", /\/authenticate\/login$/);
    });
  });
});
