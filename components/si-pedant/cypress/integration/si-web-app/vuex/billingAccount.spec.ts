describe("billingAccount module", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
    cy.visit("/");
  });

  describe("actions", () => {
    describe("get", () => {
      it("gets the users billingAccount", () => {
        cy.get("@profile").then((profile: Record<string, any>) => {
          cy.vuex().then(async (store) => {
            await store.dispatch("billingAccount/get", {
              billingAccountId: profile.billingAccount.id,
            });
            expect(store.state.billingAccount.billingAccounts).to.have.lengthOf(
              1,
              "should have 1 billingAccount",
            );
            expect(store.state.billingAccount.current)
              .to.have.property("name")
              .and.to.equal("boboCorp");
          });
        });
      });
    });
  });
});
