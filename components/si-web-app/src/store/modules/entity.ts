import { Module } from "vuex";
import _ from "lodash";

import { graphqlQuery, graphqlMutation } from "@/api/apollo";
import { RootStore } from "@/store";

interface EntityMeta {
  workspaceId: string;
  partial: boolean;
  entity: Entity;
}

interface Entity {
  id: string;
  name?: string;
  description?: string;
  siStorable?: {
    [key: string]: any;
  };
  siProperties?: {
    [key: string]: any;
  };
  properties?: {
    [key: string]: any;
  };
  constraints?: {
    [key: string]: any;
  };
  implicitConstraints?: {
    [key: string]: any;
  };
}

export interface EntityStore {
  entities: EntityMeta[];
}

interface AddEntitiesToWorkspacePayload {
  workspaceId: string;
  partial: boolean;
  entities: Entity[];
}

interface CreateEntityPayload {
  typeName: string;
  data: Record<string, any>;
}

interface LoadEntitiesPayload {
  changeSetId?: string;
}

// TODO: Tomorrow, get the entity create loop fully working, so you can
// view them after the fact!
export const entity: Module<EntityStore, RootStore> = {
  namespaced: true,
  state: {
    entities: [],
  },
  mutations: {
    add(state, payload: AddEntitiesToWorkspacePayload) {
      const newEntities = _.map(
        payload.entities,
        (entity): EntityMeta => {
          return {
            workspaceId: payload.workspaceId,
            partial: payload.partial,
            entity: entity,
          };
        },
      );
      const finalEntities = _.unionBy(newEntities, state.entities, "entity.id");
      state.entities = finalEntities;
    },
  },
  getters: {
    allForWorkspace: (state: EntityStore) => (
      workspaceId: string,
    ): Entity[] => {
      const workspaceResult = _.filter(state.entities, [
        "workspaceId",
        workspaceId,
      ]);
      if (workspaceResult) {
        return _.map(workspaceResult, "entity");
      } else {
        return [];
      }
    },
    get: (state: EntityStore) => (id: string): Entity => {
      const entityMeta = _.find(state.entities, ["entity.id", id]);
      if (entityMeta) {
        return entityMeta.entity;
      } else {
        throw new Error(
          `Cannot find entity ${id}; is it loaded/added/fetched?`,
        );
      }
    },
  },
  actions: {
    async create(
      { commit, dispatch, rootGetters },
      payload: CreateEntityPayload,
    ): Promise<void> {
      const variables = payload.data;
      const workspaceId = rootGetters["user/currentWorkspaceId"];
      const changeSetId = rootGetters["changeSet/currentId"];
      variables.changeSetId = changeSetId;
      variables.workspaceId = workspaceId;
      if (variables.properties?.kubernetesObjectYaml != undefined) {
        delete variables.properties.kubernetesObjectYaml;
      }

      const result = await graphqlMutation({
        typeName: payload.typeName,
        methodName: "create",
        variables,
      });
      const entity = result["item"];
      const addPayload: AddEntitiesToWorkspacePayload = {
        workspaceId,
        partial: false,
        entities: [entity],
      };
      commit("add", addPayload);
      dispatch(
        "editor/addNode",
        {
          id: entity["id"],
          name: entity["name"],
          isEntity: true,
          changeSetId: changeSetId,
        },
        { root: true },
      );
    },
    async get(
      { state, commit, rootGetters },
      { id, typeName }: { id: string; typeName: string },
    ): Promise<void> {
      const entityMeta = _.find(state.entities, ["entity.id", id]);
      if (!entityMeta || entityMeta?.partial == true) {
        if (!typeName) {
          throw new Error(
            `Cannot load partial entity; invalid typeName for ${id}!`,
          );
        }
        const entityGetResult = await graphqlQuery({
          typeName,
          methodName: "get",
          variables: {
            id,
          },
        });
        if (entityMeta) {
          commit("add", {
            workspaceId: entityMeta.workspaceId,
            partial: false,
            entities: [entityGetResult["item"]],
          });
        } else {
          let currentWorkspace = rootGetters["user/currentWorkspace"];
          commit("add", {
            workspaceId: currentWorkspace.id,
            partial: false,
            entities: [entityGetResult["item"]],
          });
        }
      }
    },
    async load({ commit, dispatch, rootGetters }): Promise<void> {
      let workspaceId = rootGetters["user/currentWorkspaceId"];
      let changeSetId = undefined;
      try {
        changeSetId = rootGetters["changeSet/currentId"];
      } catch (e) {
        console.log(e);
      }
      let remainingItems = true;
      let nextPageToken = "";
      let defaultVariables: Record<string, any> = {};
      if (changeSetId) {
        defaultVariables["query"] = {
          changeSetId,
        };
      }

      while (remainingItems) {
        let itemList;
        if (nextPageToken) {
          itemList = await graphqlQuery({
            typeName: "item",
            methodName: "list",
            variables: {
              pageToken: nextPageToken,
              ...defaultVariables,
            },
          });
        } else {
          itemList = await graphqlQuery({
            typeName: "item",
            methodName: "list",
            variables: {
              pageSize: "100",
              ...defaultVariables,
            },
          });
        }
        let entities = _.filter(itemList["items"], (item): boolean => {
          if (/_entity$/.exec(item["siStorable"]["typeName"])) {
            return true;
          } else {
            return false;
          }
        });
        commit("add", {
          workspaceId,
          entities,
          partial: true,
        });
        for (let entity of entities) {
          dispatch(
            "editor/addNode",
            {
              id: entity["id"],
              name: entity["name"],
              isEntity: true,
              changeSetId: entity["siStorable"]["changeSetId"],
            },
            { root: true },
          );
        }
        nextPageToken = itemList["nextPageToken"];
        if (!nextPageToken) {
          remainingItems = false;
        }
      }
    },
  },
};
