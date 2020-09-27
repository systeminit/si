import Vue from "vue";
import { Module } from "vuex";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import { ChangeSet, ChangeSetParticipant } from "@/api/sdf/model/changeSet";
import { EditSession } from "@/api/sdf/model/editSession";
import { Node, NodeKind } from "@/api/sdf/model/node";
import { Edge, EdgeKind } from "@/api/sdf/model/edge";
import { System } from "@/api/sdf/model/system";
import { RootStore } from "@/store";

export type Application = Entity;

export interface ApplicationStore {
  list: Entity[];
  systems: {
    [key: string]: System[];
  };
  services: {
    [key: string]: Entity[];
  };
  changeSetCounts: {
    [key: string]: {
      open: number;
      closed: number;
    };
  };
}

export interface ActionCreate {
  name: string;
}

export interface MutationUpdateSystem {
  system: System;
  application: {
    id: string;
  };
}

export interface MutationUpdateChangeSetCount {
  application: Entity;
  changeSetCounts: {
    open: number;
    closed: number;
  };
}

export const application: Module<ApplicationStore, RootStore> = {
  namespaced: true,
  state: {
    list: [],
    systems: {},
    changeSetCounts: {},
    services: {},
  },
  getters: {},
  mutations: {
    updateList(state, payload: Entity) {
      state.list = _.unionBy([payload], state.list, "id");
    },
    bulkUpdateList(state, payload: Entity[]) {
      state.list = _.unionBy(payload, state.list, "id");
    },
    updateSystem(state, payload: MutationUpdateSystem) {
      if (!state.hasOwnProperty(payload.application.id)) {
        Vue.set(state.systems, payload.application.id, []);
      }
      state.systems[payload.application.id] = _.unionBy(
        [payload.system],
        state.systems[payload.application.id],
      );
    },
    updateChangeSetCount(state, payload: MutationUpdateChangeSetCount) {
      Vue.set(
        state.changeSetCounts,
        payload.application.id,
        payload.changeSetCounts,
      );
    },
    clear(state) {
      state.list = [];
      state.systems = {};
      state.changeSetCounts = {};
    },
  },
  actions: {
    async list({ state, commit }): Promise<ApplicationStore["list"]> {
      if (state.list.length == 0) {
        let applications = await Entity.list_head_by_object_type("application");
        commit("bulkUpdateList", applications.items);
      }
      return state.list;
    },
    async create(
      { commit, rootGetters },
      payload: ActionCreate,
    ): Promise<Entity> {
      let workspace = rootGetters["workspace/current"];
      let organization = rootGetters["organization/current"];
      let system = rootGetters["system/current"];

      let changeSet = await ChangeSet.create({
        workspaceId: workspace.id,
        organizationId: organization.id,
      });
      let editSession = await EditSession.create(changeSet.id, {
        workspaceId: workspace.id,
        organizationId: organization.id,
      });
      let appNode = await Node.create({
        name: payload.name,
        kind: NodeKind.Entity,
        objectType: "application",
        organizationId: organization.id,
        workspaceId: workspace.id,
        changeSetId: changeSet.id,
        editSessionId: editSession.id,
        systemIds: [system.id],
      });
      await changeSet.execute({ hypothetical: false });
      let entity = (await appNode.head_object()) as Entity;
      commit("updateList", entity);
      return entity;
    },
    async fromChangeSet({ commit }, payload: ChangeSet) {
      let participants = await payload.participants();
      let entities = _.filter(participants, [
        "siStorable.typeName",
        "entity",
      ]) as Entity[];
      for (let p of entities) {
        if (p.objectType == "application") {
          let changeSetCounts = await p.changeSetCounts();
          commit("updateChangeSetCount", {
            application: p,
            changeSetCounts,
          });
        }
      }
    },
    async fromEntity({ commit }, payload: Entity) {
      if (payload.objectType == "application") {
        commit("updateList", payload);
      }
    },
    async fromChangeSetParticipant({ commit }, payload: ChangeSetParticipant) {
      if (payload.objectId.startsWith("entity:")) {
        let application;
        try {
          application = await Entity.get_head({ id: payload.objectId });
        } catch (err) {
          try {
            application = await Entity.get_projection({
              id: payload.objectId,
              changeSetId: payload.changeSetId,
            });
          } catch (err) {}
        }
        if (application?.objectType == "application") {
          let changeSetCounts = await application.changeSetCounts();
          commit("updateChangeSetCount", { application, changeSetCounts });
        }
      }
    },
    async fromEdge({ commit }, payload: Edge) {
      if (
        payload.kind == EdgeKind.Includes &&
        payload.tailVertex.typeName == "system" &&
        payload.headVertex.typeName == "application"
      ) {
        let system = await System.get({ id: payload.tailVertex.objectId });
        commit("updateSystem", {
          system,
          application: { id: payload.headVertex.objectId },
        });
      }
    },
    async clear({ commit }) {
      commit("clear");
    },
  },
};

//export interface ApplicationStore {
//  applications: ApplicationEntity[];
//  current: null | ApplicationEntity;
//}
//
//interface AddMutation {
//  applications: ApplicationEntity[];
//}
//
//interface CreateMutation {
//  name: string;
//}
//
//interface GetGetter {
//  id: string;
//}
//
//
//export const application: Module<ApplicationStore, RootStore> = {
//  namespaced: true,
//  state: {
//    applications: [],
//    current: null,
//  },
//  getters: {
//    current(state): ApplicationEntity {
//      if (state.current) {
//        return state.current;
//      } else {
//        throw new Error("Cannot get current application; it is not set!");
//      }
//    },
//    saved(state): ApplicationEntity[] {
//      return _.filter(state.applications, entity => {
//        if (!entity.siStorable?.changeSetId) {
//          return true;
//        } else {
//          return false;
//        }
//      });
//    },
//    // prettier-ignore
//    get: (state) => (filter: GetGetter): ApplicationEntity => {
//      const app = _.find(state.applications, ["id", filter.id]);
//      if (app) {
//        return app;
//      } else {
//        throw new Error(`cannot find application id ${filter.id}`);
//      }
//    }
//  },
//  mutations: {
//    add(state, payload: AddMutation) {
//      state.applications = _.unionBy(
//        payload.applications,
//        state.applications,
//        "id",
//      );
//    },
//    current(state, payload: ApplicationEntity) {
//      state.current = payload;
//    },
//  },
//  actions: {
//    setCurrentById({ commit, state }, applicationId: string) {
//      let app = _.find(state.applications, ["id", applicationId]);
//      if (app) {
//        commit("current", app);
//      } else {
//        console.log("cannot find application", {
//          applications: state.applications,
//        });
//        throw new Error(`cannot find application for ${applicationId}`);
//      }
//    },
//    add({ commit }, payload: AddMutation) {
//      commit("add", payload);
//    },
//    async create(
//      { dispatch, rootGetters, commit },
//      payload: CreateMutation,
//    ): Promise<ApplicationEntity> {
//      // let workspace = rootGetters["workspace/current"];
//      // let organization = rootGetters["organization/current"];
//      // let cs = new ChangeSet({ workspaceId: workspace.id });
//      // let es = new EditSession({ changeSetId: cs.id });
//      // let app = new Node({ editSessionId: es.id, objectType: "application" });
//      //
//      await dispatch("changeSet/createDefault", {}, { root: true });
//      await dispatch("editSession/create", {}, { root: true });
//      let currentSystem = rootGetters["system/current"];
//      let newApp = await dispatch(
//        "entity/create",
//        {
//          typeName: "application_entity",
//          data: {
//            name: payload.name,
//            properties: { inSystems: [currentSystem.id] },
//          },
//        },
//        { root: true },
//      );
//      await dispatch("changeSet/execute", { wait: true }, { root: true });
//      return newApp;
//    },
//  },
//};
