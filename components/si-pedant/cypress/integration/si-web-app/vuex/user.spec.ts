describe("user module", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
    cy.visit("/");
  });

  describe("actions", () => {
    describe("isAuthenticated", () => {
      it("is true when logged in", () => {
        cy.vuex().then((store) => {
          store
            .dispatch("user/isAuthenticated")
            .then((isAuthenticated: boolean) => {
              expect(isAuthenticated).to.be.true;
            });
        });
      });

      it("is false when logged out", () => {
        cy.logout();
        cy.vuex().then((store) => {
          store
            .dispatch("user/isAuthenticated")
            .then((isAuthenticated: boolean) => {
              expect(isAuthenticated).to.be.false;
            });
        });
      });
    });
  });
});
