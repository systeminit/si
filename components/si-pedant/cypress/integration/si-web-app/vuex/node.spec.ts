import { ChangeSetFactory } from "../../../support/changeSet";
import { KubernetesDeploymentEntityFactory } from "../../../support/kubernetesDeploymentEntity";
import * as _ from "lodash";

describe("node module", () => {
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
  });

  describe("getters", () => {
    describe("list", () => {
      beforeEach(() => {
        cy.get("@profile")
          .then(async (profile: Record<string, any>) => {
            // Creating a new KubernetesDeploymentEntity, without saving it.
            const csf = new ChangeSetFactory({
              name: "sneezy",
              workspaceId: profile.workspaceDefault?.id,
              createdByUserId: profile.user?.id,
            });
            const changeSet = await csf.create();
            if (!changeSet.id) {
              throw new Error("changeset is missing an id");
            }
            const kdef = new KubernetesDeploymentEntityFactory({
              name: "sloopy",
              workspaceId: profile.workspaceDefault?.id,
              changeSetId: changeSet.id,
            });
            await kdef.create();
            return changeSet;
          })
          .as("changeSet");
      });

      it("returns the list of saved nodes without a changeset", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          const nodeList = await store.getters["node/list"];
          // Would be six if the code to not show non-saved items fails
          expect(nodeList).to.have.lengthOf(5);
          expect(_.find(nodeList, ["name", "sloopy"])).to.be.undefined;
        });
      });

      it("returns the list of saved nodes and changeset nodes with a changeset", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          cy.get("@changeSet").then(async (changeSet) => {
            store.commit("changeSet/current", changeSet);
            const nodeList = await store.getters["node/list"];
            expect(nodeList).to.have.lengthOf(6);
            expect(_.find(nodeList, ["name", "sloopy"])).to.have.property("id");
          });
        });
      });
    });
    describe("displayById", () => {
      it("displays the item without a changeset", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          const displayNode = store.state.node.nodes[0];
          const display = store.getters["node/displayById"](displayNode.id);
          expect(display)
            .to.have.property("siStorable")
            .and.property("changeSetId").to.be.null;
          expect(display)
            .to.have.property("name")
            .and.to.be.equal(displayNode.name);
        });
      });
      it("displays the highest changeSet item with a changeset", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          const displayNode = store.state.node.nodes[0];
          const currentChangeSet = _.cloneDeep(
            store.state.changeSet.changeSets[0],
          );
          store.commit("changeSet/current", currentChangeSet);
          const display = store.getters["node/displayById"](displayNode.id);
          expect(display)
            .to.have.property("siStorable")
            .and.property("changeSetId")
            .to.be.equal(currentChangeSet.id);
          expect(display)
            .to.have.property("name")
            .and.to.be.equal(displayNode.name);
        });
      });
    });
    describe("displayCurrent", () => {
      it("displays the current item without a changeset", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          const currentNode = _.cloneDeep(store.state.node.nodes[0]);
          store.commit("node/current", currentNode);
          const currentDisplay = store.getters["node/displayCurrent"];
          expect(currentDisplay)
            .to.have.property("siStorable")
            .and.property("changeSetId").to.be.null;
          expect(currentDisplay)
            .to.have.property("name")
            .and.to.be.equal(currentNode.name);
        });
      });
      it("displays the highest changeSet item with a changeset", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          const currentNode = _.cloneDeep(store.state.node.nodes[0]);
          store.commit("node/current", currentNode);
          const currentChangeSet = _.cloneDeep(
            store.state.changeSet.changeSets[0],
          );
          store.commit("changeSet/current", currentChangeSet);
          const currentDisplay = store.getters["node/displayCurrent"];
          expect(currentDisplay)
            .to.have.property("siStorable")
            .and.property("changeSetId")
            .to.be.equal(currentChangeSet.id);
          expect(currentDisplay)
            .to.have.property("name")
            .and.to.be.equal(currentNode.name);
        });
      });
    });
  });

  describe("actions", () => {
    describe("add", () => {
      it("adds a node for a stack of items", () => {
        cy.vuex().then(async (store) => {
          // This is through the loader, because we actually do care
          // about the order things are loaded in - and thats the loaders job!
          await store.dispatch("loader/load");
          expect(store.state)
            .to.have.property("node")
            .and.property("nodes")
            .to.have.lengthOf(5); // Only the total number, not the changeset number as well!
        });
      });
    });
    describe("create", () => {
      it("creates a new node of the given type and selects it", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          await store.dispatch("node/create", {
            nodeType: "Entity",
            typeName: "kubernetesDeploymentEntity",
          });
          expect(store.state)
            .to.have.property("node")
            .and.property("nodes")
            .to.have.lengthOf(6);
        });
      });
    });
    describe("delete", () => {
      it.only("marks a node as deleted", () => {
        cy.vuex().then(async (store) => {
          await store.dispatch("loader/load");
          await store.dispatch("node/create", {
            nodeType: "Entity",
            typeName: "kubernetesDeploymentEntity",
          });
          let node = store.state.node.nodes[0];
          const stackSize = node.stack.length;
          await store.dispatch("node/current", { node });
          await store.dispatch("node/delete");
          node = store.state.node.nodes[0];
          expect(node)
            .to.have.property("stack")
            .and.lengthOf(stackSize + 1);
        });
      });
    });
  });
});
