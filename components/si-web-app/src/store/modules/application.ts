import Vue from "vue";
import { Module } from "vuex";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import { ChangeSet, ChangeSetParticipant } from "@/api/sdf/model/changeSet";
import { EditSession } from "@/api/sdf/model/editSession";
import { Node, NodeKind } from "@/api/sdf/model/node";
import { Edge, EdgeKind } from "@/api/sdf/model/edge";
import { System } from "@/api/sdf/model/system";
import { Resource } from "@/api/sdf/model/resource";
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
  resources: {
    [key: string]: Resource[];
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

export interface MutationUpdateServices {
  applicationNodeId: string;
  entity: Entity;
}

export const application: Module<ApplicationStore, RootStore> = {
  namespaced: true,
  state: {
    list: [],
    systems: {},
    changeSetCounts: {},
    services: {},
    resources: {},
  },
  getters: {},
  mutations: {
    updateList(state, payload: Entity) {
      state.list = _.orderBy(
        _.unionBy([payload], state.list, "id"),
        ["name"],
        ["asc"],
      );
    },
    bulkUpdateList(state, payload: Entity[]) {
      state.list = _.orderBy(
        _.unionBy(payload, state.list, "id"),
        ["name"],
        ["asc"],
      );
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
    updateResources(
      state,
      payload: { resource: Resource; applicationId: string },
    ) {
      let currentResources = state.resources[payload.applicationId] || [];
      state.resources[payload.applicationId] = _.unionBy(
        [payload.resource],
        currentResources,
        "id",
      );
    },
    updateServices(state, payload: MutationUpdateServices) {
      let currentServices = state.services[payload.applicationNodeId] || [];
      state.services[payload.applicationNodeId] = _.unionBy(
        [payload.entity],
        currentServices,
        "id",
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
      let entity = (await appNode.headObject()) as Entity;
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
    async fromResource({ state, commit }, payload: Resource) {
      const node = await Node.get({ id: payload.nodeId });
      const predecessors = await node.predecessors();
      let application: Node | undefined = undefined;
      for (const pNode of predecessors) {
        if (pNode.objectType == "application") {
          application = pNode;
          break;
        }
      }
      let applicationEntity: Entity | undefined = undefined;
      if (application) {
        try {
          applicationEntity = (await application.headObject()) as Entity;
        } catch {
          applicationEntity = undefined;
        }
      }
      if (application && applicationEntity) {
        let servicesList = state.services[applicationEntity.nodeId];
        if (servicesList) {
          if (_.find(servicesList, ["id", payload.entityId])) {
            commit("updateResources", {
              applicationId: applicationEntity.id,
              resource: payload,
            });
          }
        }
      }
    },
    async fromEntity({ commit }, payload: Entity) {
      if (payload.objectType == "application" && payload.head == true) {
        commit("updateList", payload);
      }
      if (payload.objectType == "service" && payload.head == true) {
        const node = await Node.get({ id: payload.nodeId });
        const predecessors = await node.predecessors();
        for (const pNode of predecessors) {
          if (pNode.objectType == "application") {
            commit("updateServices", {
              applicationNodeId: pNode.id,
              entity: payload,
            });
          }
        }
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
