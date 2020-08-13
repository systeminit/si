import { Module } from "vuex";
import _ from "lodash";

import { Edge, EdgeEdgeKind } from "@/graphql-types";
import { RootStore } from "@/store";
import { graphqlMutation, graphqlQueryListAll } from "@/api/apollo";

export interface EdgeStore {
  edges: Edge[];
  current: null | Edge;
}

interface AddMutation {
  edges: Edge[];
}

interface CreateMutation {
  name?: string;
  headVertex: string;
  tailVertex: string;
  bidirectional?: boolean;
  edgeKind: EdgeEdgeKind;
}

interface FromIdForTypeGetter {
  id: string;
  typeName: string;
}

export const edge: Module<EdgeStore, RootStore> = {
  namespaced: true,
  state: {
    edges: [],
    current: null,
  },
  getters: {
    // prettier-ignore
    filter: (state) => (filter: any): EdgeStore["edges"] => {
      return _.filter(state.edges, filter);
    },
    // prettier-ignore
    fromIdForType: (state, _getters, rootState) => (filter: FromIdForTypeGetter): EdgeStore["edges"] => {
      let currentChangeSetId = rootState.changeSet.current?.id;
      let entities = _.filter(rootState.entity.entities, [
        "siStorable.typeName",
        filter.typeName,
      ]);
      let edges = _.filter(state.edges, (edge: Edge) => {
        if (currentChangeSetId) {
          if (edge.siStorable?.changeSetId) {
            if (edge.siStorable?.changeSetId != currentChangeSetId) {
              return false;
            }
          }
        } else {
          if (edge.siStorable?.changeSetId) {
            return false;
          }
        }
        if (
          edge.headVertex?.id == filter.id &&
          _.find(entities, ["id", edge.tailVertex?.id])
        ) {
          return true;
        } else if (
          edge.tailVertex?.id == filter.id &&
          _.find(entities, ["id", edge.headVertex?.id])
        ) {
          return true;
        } else {
          return false;
        }
      });
      return edges;
    },
    current(state): Edge {
      if (state.current) {
        return state.current;
      } else {
        throw new Error("Cannot get current edge; it is not set!");
      }
    },
    saved(state): Edge[] {
      return _.filter(state.edges, entity => {
        if (!entity.siStorable?.changeSetId) {
          return true;
        } else {
          return false;
        }
      });
    },
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.edges = _.unionBy(payload.edges, state.edges, "id");
    },
    current(state, payload: Edge) {
      state.current = payload;
    },
  },
  actions: {
    add({ commit }, payload: AddMutation) {
      commit("add", payload);
    },
    async create({ commit, rootGetters }, payload: CreateMutation) {
      const workspace = rootGetters["workspace/current"];
      const profile = rootGetters["user/profile"];

      let head = rootGetters["entity/get"](["id", payload.headVertex]);
      let tail = rootGetters["entity/get"](["id", payload.tailVertex]);
      let data: Record<string, any> = {
        bidirectional: payload.bidirectional ? true : false,
      };
      if (head.siStorable?.changeSetId) {
        data.headVertex = {
          id: head.siStorable?.itemId,
          typeName: head.siStorable?.typeName,
        };
      } else {
        data.headVertex = {
          id: head.id,
          typeName: head.siStorable?.typeName,
        };
      }
      if (tail.siStorable?.changeSetId) {
        data.tailVertex = {
          id: tail.siStorable?.itemId,
          typeName: tail.siStorable?.typeName,
        };
      } else {
        data.tailVertex = {
          id: tail.id,
          typeName: tail.siStorable?.typeName,
        };
      }
      if (payload.name) {
        data.name = payload.name;
      }
      if (payload.edgeKind) {
        data["edgeKind"] = payload.edgeKind;
      } else {
        data["edgeKind"] = "CONNECTED";
      }
      const edge = await graphqlMutation({
        typeName: "edge",
        methodName: "create",
        variables: {
          name: `${head.id}:${tail.id}`,
          displayName: `${head.id}:${tail.id}`,
          siProperties: {
            workspaceId: workspace.id,
            billingAccountId: profile.billingAccount?.id,
            organizationId: profile.organization?.id,
          },
          ...data,
        },
      });
      commit("add", { edges: [edge] });
    },
    async load({ commit }): Promise<void> {
      const edges: Edge[] = await graphqlQueryListAll({
        typeName: "edge",
      });
      if (edges.length > 0) {
        commit("add", { edges });
      }
    },
  },
};
