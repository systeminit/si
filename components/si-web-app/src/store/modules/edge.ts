import { Module } from "vuex";
import _ from "lodash";

import { EdgeEntity, EdgeComponentConstraintsEdgeKind } from "@/graphql-types";
import { RootStore } from "@/store";

export interface EdgeStore {
  edges: EdgeEntity[];
  current: null | EdgeEntity;
}

interface AddMutation {
  edges: EdgeEntity[];
}

interface CreateMutation {
  name?: string;
  headVertex: string;
  tailVertex: string;
  bidirectional?: boolean;
  edgeKind: EdgeComponentConstraintsEdgeKind;
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
      let edges = _.filter(state.edges, (edge: EdgeEntity) => {
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
          edge.properties?.headVertex?.id == filter.id &&
          _.find(entities, ["id", edge.properties?.tailVertex?.id])
        ) {
          return true;
        } else if (
          edge.properties?.tailVertex?.id == filter.id &&
          _.find(entities, ["id", edge.properties?.headVertex?.id])
        ) {
          return true;
        } else {
          return false;
        }
      });
      return edges;
    },
    current(state): EdgeEntity {
      if (state.current) {
        return state.current;
      } else {
        throw new Error("Cannot get current edge; it is not set!");
      }
    },
    saved(state): EdgeEntity[] {
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
    current(state, payload: EdgeEntity) {
      state.current = payload;
    },
  },
  actions: {
    add({ commit }, payload: AddMutation) {
      commit("add", payload);
    },
    async create({ dispatch, rootGetters }, payload: CreateMutation) {
      let head = rootGetters["entity/get"](["id", payload.headVertex]);
      let tail = rootGetters["entity/get"](["id", payload.tailVertex]);
      let data: Record<string, any> = {
        properties: {
          bidirectional: payload.bidirectional ? true : false,
        },
      };
      if (head.siStorable?.changeSetId) {
        data.properties.headVertex = {
          id: head.siStorable?.itemId,
          typeName: head.siStorable?.typeName,
        };
      } else {
        data.properties.headVertex = {
          id: head.id,
          typeName: head.siStorable?.typeName,
        };
      }
      if (tail.siStorable?.changeSetId) {
        data.properties.tailVertex = {
          id: tail.siStorable?.itemId,
          typeName: tail.siStorable?.typeName,
        };
      } else {
        data.properties.tailVertex = {
          id: tail.id,
          typeName: tail.siStorable?.typeName,
        };
      }
      if (payload.name) {
        data.name = payload.name;
      }
      if (payload.edgeKind) {
        data["constraints"] = {
          edgeKind: payload.edgeKind,
        };
      } else {
        data["constraints"] = {
          edgeKind: "CONNECTED",
        };
      }

      await dispatch(
        "entity/create",
        {
          typeName: "edge_entity",
          data,
        },
        { root: true },
      );
    },
  },
};
