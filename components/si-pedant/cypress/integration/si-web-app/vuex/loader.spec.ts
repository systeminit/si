import { ChangeSetFactory } from "../../../support/changeSet";
import { KubernetesDeploymentEntityFactory } from "../../../support/kubernetesDeploymentEntity";

describe("loader module", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
    cy.visit("/");
  });

  describe("actions", () => {
    it("load", () => {
      cy.get("@profile").then(async (profile: Record<string, any>) => {
        const csf = new ChangeSetFactory({
          name: "coffee",
          workspaceId: profile.workspaceDefault?.id,
          createdByUserId: profile.user?.id,
        });
        const changeSet = await csf.create();
        if (!changeSet.id) {
          throw new Error("changeset is missing an id");
        }
        const kdef = new KubernetesDeploymentEntityFactory({
          name: "kexp",
          workspaceId: profile.workspaceDefault?.id,
          changeSetId: changeSet.id,
        });
        await kdef.create();
      });

      cy.vuex().then(async (store) => {
        expect(store.state).to.have.property("loader").and.property("loading")
          .to.be.false;

        await store.dispatch("loader/load");

        expect(store.state)
          .to.have.property("workspace")
          .and.property("current").to.not.be.null;

        expect(store.state)
          .to.have.property("billingAccount")
          .and.property("current")
          .and.property("name")
          .to.equal("boboCorp");

        expect(store.state)
          .to.have.property("changeSet")
          .and.property("changeSets")
          .to.have.lengthOf(1)
          .and.property("0")
          .and.property("name")
          .be.equal("coffee");

        expect(store.state)
          .to.have.property("entity")
          .and.property("entities")
          .to.have.lengthOf(1)
          .and.property("0")
          .and.property("name")
          .be.equal("kexp");
      });
    });
  });
});
