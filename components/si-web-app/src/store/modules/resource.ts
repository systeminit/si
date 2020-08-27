import { Module } from "vuex";
import _ from "lodash";

import { Resource } from "@/graphql-types";
import { RootStore } from "@/store";
import { graphqlMutation, graphqlQueryListAll } from "@/api/apollo";

// When an entity would have a resource:
//  * it doesn't exist
//  * when an action is dispatched that should create it,
//    it is pending
//  * when the action is finished, it is "created" or "unknown"

export enum ResourceStatus {
  Pending = "PENDING", // There is an action pending for this resource
  Created = "CREATED", // The resource exists, but either has no status endpoint or it hasn't reported yet
  Failed = "FAILED", // The resource creation failed
  Deleted = "DELETED", // The resource has been removed
}

export enum ResourceHealth {
  Ok = "OK", // The status endpoint is OK
  Warning = "WARNING", // I have a warning from the status endpoint
  Error = "ERROR", // I have an error from the status endpoint
  Unknown = "UNKNOWN", // Status endpoint is telling lies, or I have no idea
}

export interface ResourceStore {
  resources: Resource[];
}

interface AddMutation {
  resources: Resource[];
}

export interface AddAction {
  resources: Resource[];
}

export interface CreateAction {
  nodeName: String;
  entityId: Resource["entityId"];
  nodeId: Resource["nodeId"];
  status: Resource["status"];
  health: Resource["health"];
  data: Resource["data"];
}

export interface UpdateOnActionAction {
  entityId: string;
}

export const resource: Module<ResourceStore, RootStore> = {
  namespaced: true,
  state: {
    resources: [],
  },
  getters: {
    forNodeList(state, getters, rootState, rootGetters): Resource[] {
      const nodes = rootGetters["node/list"];
      const resources = _.map(nodes, node => {
        let resource = _.find(state.resources, ["nodeId", node.id]);
        if (resource) {
          return resource;
        } else {
          return undefined;
        }
      });
      const filterResources = _.filter(resources, resource => {
        return resource != undefined;
      });
      if (filterResources != undefined) {
        // @ts-ignore
        return filterResources;
      } else {
        return [];
      }
    },
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.resources = _.unionBy(payload.resources, state.resources, "id");
    },
  },
  actions: {
    add({ commit }, payload: AddAction) {
      commit("add", payload);
    },
    async sync({ commit, dispatch, getters }): Promise<Resource[]> {
      let status = ResourceStatus.Pending;
      let health = ResourceHealth.Unknown;
      let resources = getters["forNodeList"];
      let result = [];
      for (const resource of resources) {
        const newValue = _.cloneDeep(resource);
        newValue.status = status;
        newValue.health = health;
        result.push(newValue);
        await graphqlMutation({
          typeName: "resource",
          methodName: "update",
          variables: {
            ...newValue,
          },
        });
        commit("add", { resources: [newValue] });
        setTimeout(async function() {
          await dispatch(
            "resource/updateOnAction",
            {
              entityId: newValue.entityId,
            },
            { root: true },
          );
        }, Math.floor(Math.random() * 3001));
      }
      return result;
    },
    async updateOnAction(
      { commit, state },
      payload: UpdateOnActionAction,
    ): Promise<Resource[]> {
      let status = ResourceStatus.Created;
      let health = ResourceHealth.Ok;
      let resources = _.filter(state.resources, ["entityId", payload.entityId]);
      let result = [];
      for (const resource of resources) {
        const newValue = _.cloneDeep(resource);
        newValue.status = status;
        newValue.health = health;
        result.push(newValue);
        await graphqlMutation({
          typeName: "resource",
          methodName: "update",
          variables: {
            ...newValue,
          },
        });
      }
      commit("add", { resources: result });
      return result;
    },
    async create(
      { commit, rootGetters },
      payload: CreateAction,
    ): Promise<Resource> {
      const workspace = rootGetters["workspace/current"];
      const profile = rootGetters["user/profile"];
      console.log({ payload, workspace, profile });

      const resource = await graphqlMutation({
        typeName: "resource",
        methodName: "create",
        variables: {
          name: `${payload.nodeName}`,
          displayName: `${payload.nodeName}`,
          siProperties: {
            workspaceId: workspace.id,
            billingAccountId: profile.billingAccount?.id,
            organizationId: profile.organization?.id,
          },
          ...payload,
        },
      });
      commit("add", { resources: [resource.item] });
      return resource.item;
    },
    async load({ commit }): Promise<void> {
      const resources: Resource[] = await graphqlQueryListAll({
        typeName: "resource",
      });
      if (resources.length > 0) {
        commit("add", { resources });
      }
    },
  },
};
