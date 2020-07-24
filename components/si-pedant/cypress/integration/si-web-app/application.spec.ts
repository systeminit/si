describe("application", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
  });

  it("create", () => {
    cy.visit("/");
    cy.get("[data-cy=application-nav-link]").click();
    cy.location("pathname").should("match", /^\/o\/(.+)\/w\/(.+)\/a$/);
    cy.get("[data-cy=new-application-button]").click();
    cy.get("[data-cy=new-application-form-application-name]").type("alcest");
    cy.get("[data-cy=new-application-form-create-button]").click();
    cy.get("[data-cy=application-card-name]").contains("alcest");
    cy.get("[data-cy=systems-visualization-default]");
    cy.get("[data-cy=change-set-visualization-open-count]").contains("0");
    cy.get("[data-cy=change-set-visualization-closed-count]").contains("1");
    cy.location("pathname").should("match", /^\/o\/(.+)\/w\/(.+)\/a\/(.+)$/);
  });

  it("list", () => {
    cy.visit("/");
    cy.vuex().should(async (store) => {
      await store.dispatch("application/create", { name: "metallica" });
      await store.dispatch("application/create", { name: "moon tooth" });
    });
    cy.get("[data-cy=application-nav-link]").click();
    cy.get("[data-cy=application-card-name]").contains("metallica");
    cy.get("[data-cy=application-card-name]").contains("moon tooth");
  });

  describe("details", () => {
    beforeEach(() => {
      cy.visit("/");
      cy.vuex().should(async (store) => {
        await store.dispatch("application/create", { name: "metallica" });
      });
      cy.get("[data-cy=application-nav-link]").click();
      cy.get("[data-cy=application-list-link-metallica]").click({
        force: true,
      });
    });

    it.only("shows data", () => {
      cy.get("[data-cy=application-details-application-name]").contains(
        "applications/metallica",
      );
      cy.get("[data-cy=application-details-extended]").should("visible");

      // Edit Mode
      cy.get("[data-cy=application-details-current-mode]").contains("view");
      cy.get("[data-cy=application-details-mode-toggle").click();
      cy.get("[data-cy=application-details-current-mode]").contains("edit");

      // Detail toggle
      cy.get("[data-cy=application-details-toggle]").click();
      cy.get("[data-cy=application-details-extended]").should("not.be.visible");
    });
  });
});
