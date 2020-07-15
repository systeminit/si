import { ChangeSetFactory } from "../../../support/changeSet";
import { KubernetesDeploymentEntityFactory } from "../../../support/kubernetesDeploymentEntity";
import * as _ from "lodash";

describe("changeSet module", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
    cy.visit("/");
    cy.get("@profile").then(async (profile: Record<string, any>) => {
      const csf = new ChangeSetFactory({
        name: "coffee",
        workspaceId: profile.workspaceDefault?.id,
        createdByUserId: profile.user?.id,
      });
      for (let x = 0; x < 5; x++) {
        await csf.create(x);
      }
    });
  });

  describe("actions", () => {
    describe("load", () => {
      it("it loads all the changeSets", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          expect(store.state)
            .to.have.property("changeSet")
            .and.property("changeSets")
            .to.have.lengthOf(5);
        });
      });
    });
    describe("create", () => {
      it("adds it to changeSets and makes it current", () => {
        cy.get("@profile").then((profile: Record<string, any>) => {
          cy.vuex().then(async (store) => {
            await store.dispatch("loader/load");
            await store.dispatch("changeSet/create", {
              createdByUserId: profile.user?.id,
              workspaceId: profile.workspaceDefault?.id,
              name: "blackCrowes",
              displayName: "The Black Crowes",
            });
            expect(store.state)
              .to.have.property("changeSet")
              .and.property("changeSets")
              .to.have.lengthOf(6);
            const newChangeSet = _.find(store.state.changeSet.changeSets, {
              name: "blackCrowes",
            });
            console.log("found it?", { newChangeSet, store });
            expect(newChangeSet)
              .to.have.property("name")
              .and.be.equal("blackCrowes");
            expect(store.state)
              .to.have.property("changeSet")
              .and.property("current")
              .and.property("id")
              .to.be.eq(newChangeSet.id);
          });
        });
      });
    });
    describe("createDefault", () => {
      it("adds it to changeSets and makes it current", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          await store.dispatch("changeSet/createDefault");
          expect(store.state)
            .to.have.property("changeSet")
            .and.property("changeSets")
            .to.have.lengthOf(6);
          const newChangeSet = _.find(
            store.state.changeSet.changeSets,
            (item) => {
              if (_.startsWith(item.name, "coffee")) return false;
              return true;
            },
          );
          expect(newChangeSet).to.have.property("name").be.not.null;
          expect(store.state)
            .to.have.property("changeSet")
            .and.property("current")
            .and.property("id")
            .to.be.eq(newChangeSet.id);
        });
      });
    });
    describe("execute", () => {
      it("executes all the changeSet items and updates the state", () => {
        cy.get("@profile").then(async (profile: Record<string, any>) => {
          const csf = new ChangeSetFactory({
            name: "lacroix",
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
          cy.vuex().then(async (store) => {
            await store.dispatch("loader/load");
            store.commit("changeSet/current", changeSet);
            await store.dispatch("changeSet/execute");

            expect(store.state)
              .to.have.property("changeSet")
              .to.have.property("current")
              .to.have.property("status")
              .to.be.eq("EXECUTING");
          });
        });
      });
    });
  });
});

