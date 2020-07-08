import { ChangeSetFactory } from "../../../support/changeSet";
import { KubernetesDeploymentEntityFactory } from "../../../support/kubernetesDeploymentEntity";

describe("entity module", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
    cy.visit("/");
  });

  describe("actions", () => {
    describe("load", () => {
      it("loads all the entities in all workspaces for all changesets", () => {
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
          for (let x = 0; x < 5; x++) {
            await kdef.create(x);
          }
          await csf.execute(changeSet);
        });

        cy.vuex().then(async (store) => {
          // This is through the loader, because we actually do care
          // about the order things are loaded in - and thats the loaders job!
          await store.dispatch("loader/load");
          expect(store.state)
            .to.have.property("entity")
            .and.property("entities")
            .to.have.lengthOf(10); // 5 in the changeSet, 5 real post exec
        });
      });
    });
  });
});

