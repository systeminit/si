describe("workspace module", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
    cy.visit("/");
  });

  describe("actions", () => {
    describe("load", () => {
      it("loads all the users workspaces", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("workspace/load");
          expect(store.state.workspace.workspaces).to.have.lengthOf(
            1,
            "should have 1 workspace",
          );
        });
      });

      it("sets the current workspace to the default", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("workspace/load");
          expect(store.state.workspace.current)
            .to.have.property("name")
            .and.to.equal("default");
        });
      });
    });
  });
});
