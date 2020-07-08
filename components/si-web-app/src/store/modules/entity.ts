import { Module } from "vuex";
import _ from "lodash";
import { registry, Props, PropMethod, PropLink, PropObject } from "si-registry";

import { generateName } from "@/api/names";
import { graphqlQuery, graphqlMutation } from "@/api/apollo";
import { RootStore } from "@/store";
import { NodeType, Item } from "./node";

interface EntityMeta {
  workspaceId: string;
  partial: boolean;
  entity: Entity;
}

interface Entity {
  id: string;
  name: string;
  description: string;
  siStorable: {
    typeName: string;
    [key: string]: any;
  };
  siProperties: {
    [key: string]: any;
  };
  properties: {
    [key: string]: any;
  };
  constraints: {
    [key: string]: any;
  };
  implicitConstraints: {
    [key: string]: any;
  };
}

export interface EntityStore {
  entities: Entity[];
}

export interface EntityProperty {
  path: (string | number)[];
  prop: Props;
  name: string;
  label: string;
  required: boolean;
  repeated: boolean;
  kind: string;
  hidden: boolean;
}

interface AddMutation {
  entities: Entity[];
}

interface DeleteEntityAction {
  typeName: string;
  id: string;
}

interface CreateEntityPayload {
  typeName: string;
}

interface UpdateEntityPayload {
  typeName: string;
  data: {
    name?: string;
    description?: string;
    displayName?: string;
    [field: string]: any;
  };
  hypotheticalState?: {
    path: string[];
    value: any;
  };
}

export const entity: Module<EntityStore, RootStore> = {
  namespaced: true,
  state: {
    entities: [],
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.entities = _.unionBy(payload.entities, state.entities, "id");
    },
  },
  actions: {
    async update(
      { commit, dispatch, rootGetters },
      payload: UpdateEntityPayload,
    ): Promise<void> {
      const variables: Record<string, any> = {
        id: payload.data.id,
        update: {
          name: payload.data.name,
          displayName: payload.data.displayName,
          description: payload.data.description,
          properties: payload.data.properties,
        },
      };
      const workspaceId = rootGetters["workspace/current"].id;
      const changeSetId = rootGetters["changeSet/current"].id;
      variables.changeSetId = changeSetId;
      variables.workspaceId = workspaceId;
      if (variables.update.properties?.kubernetesObjectYaml != undefined) {
        delete variables.update.properties.kubernetesObjectYaml;
      }

      const result = await graphqlMutation({
        typeName: payload.typeName,
        methodName: "update",
        variables,
      });
      const entity = result["item"];
      commit("add", { entities: [entity] });
      let node = {
        id: entity.siStorable?.itemId,
        name: entity.name,
        nodeType: "Entity",
        object: entity,
      };
      await dispatch(
        "node/add",
        {
          items: [node],
        },
        { root: true },
      );
    },
    async create(
      { commit, dispatch, rootGetters },
      payload: CreateEntityPayload,
    ): Promise<void> {
      const variables: Record<string, any> = {};
      const workspaceId = rootGetters["workspace/current"].id;
      let changeSetId: string;
      try {
        changeSetId = rootGetters["changeSet/currentId"];
      } catch (err) {
        await dispatch("changeSet/createDefault", {}, { root: true });
        changeSetId = rootGetters["changeSet/currentId"];
      }
      variables.changeSetId = changeSetId;
      variables.workspaceId = workspaceId;
      let name = generateName();
      variables.name = name;
      variables.displayName = name;
      variables.description = name;
      if (payload.typeName == "kubernetesDeploymentEntity") {
        variables.properties = {
          kubernetesObject: {
            apiVersion: "apps/v1",
            kind: "Deployment",
          },
        };
      } else {
        variables.properties = {};
      }
      variables.constraints = {};

      const result = await graphqlMutation({
        typeName: payload.typeName,
        methodName: "create",
        variables,
      });
      const entity = result["item"];
      const addPayload: AddMutation = {
        entities: [entity],
      };
      commit("add", addPayload);
      let entityId: string;
      if (entity.siStorable.itemId) {
        entityId = entity.siStorable.itemId;
      } else {
        entityId = entity.id;
      }
      let node = {
        id: entityId,
        name: entity.name,
        nodeType: "Entity",
        object: entity,
      };
      await dispatch(
        "node/add",
        {
          items: [node],
        },
        { root: true },
      );
    },
    async delete(
      { commit, getters, rootGetters, rootState, dispatch },
      payload: DeleteEntityAction,
    ) {
      let changeSetId: string;
      try {
        changeSetId = rootGetters["changeSet/current"].id;
      } catch (err) {
        await dispatch("changeSet/createDefault", {}, { root: true });
        changeSetId = rootGetters["changeSet/current"].id;
      }
      const result = await graphqlMutation({
        typeName: payload.typeName,
        methodName: "delete",
        variables: {
          id: payload.id,
          changeSetId,
        },
      });
      const entity = result["item"];
      commit("add", { entities: [entity] });
      await dispatch(
        "node/add",
        {
          items: [
            {
              id: entity.siStorable.itemId,
              name: entity.name,
              nodeType: NodeType.Entity,
              object: entity,
            },
          ],
        },
        { root: true },
      );
    },

    async get(
      { state, commit, rootGetters, dispatch },
      { id, typeName }: { id: string; typeName: string },
    ): Promise<void> {
      const entityGetResult = await graphqlQuery({
        typeName,
        methodName: "get",
        variables: {
          id,
        },
      });
      const entity = entityGetResult["item"];
      commit("add", { entities: [entity] });
      let node;
      if (entity.siStorable.itemId) {
        node = {
          id: entity.siStorable.itemId,
          name: entity.name,
          nodeType: NodeType.Entity,
          object: entity,
        };
      } else {
        node = {
          id: entity.id,
          name: entity.name,
          nodeType: NodeType.Entity,
          object: entity,
        };
      }
      await dispatch("node/add", { items: [node] }, { root: true });
    },
    async load({ commit, dispatch, rootState }): Promise<void> {
      let workspaceIdList = _.map(rootState.workspace.workspaces, "id");

      // HACK: For now, we load all the changeset data by just loading all
      // the data a fuckload of times. This isn't what we want long term, but
      // its just fine for now.
      let changeSetIdList = _.map(rootState.changeSet.changeSets, "id");
      // Make sure we get the raw data, too. Probably overkill.
      changeSetIdList.push(undefined);

      let fullEntities: Entity[] = [];

      // Load all the data for every workspace, for every changeSet.
      //
      // Right now, the API is wrong, as we don't require you to specify the workspace!!
      for (let _workspaceId of workspaceIdList) {
        for (let changeSetId of changeSetIdList) {
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
            for (let entity of entities) {
              if (!_.find(fullEntities, entity.id)) {
                let fullEntity = await graphqlQuery({
                  typeName: entity.siStorable.typeName,
                  methodName: "get",
                  variables: {
                    id: entity.id,
                  },
                });
                fullEntities.push(fullEntity.item);
              }
            }
            nextPageToken = itemList["nextPageToken"];
            if (!nextPageToken) {
              remainingItems = false;
            }
          }
        }
      }
      commit("add", {
        entities: fullEntities,
      });
      let addEntitiesToNodes: Item[] = _.map(fullEntities, entity => {
        if (entity.siStorable.itemId) {
          return {
            id: entity.siStorable.itemId,
            name: entity.name,
            nodeType: NodeType.Entity,
            object: entity,
          };
        } else {
          return {
            id: entity.id,
            name: entity.name,
            nodeType: NodeType.Entity,
            object: entity,
          };
        }
      });
      await dispatch("node/add", { items: addEntitiesToNodes }, { root: true });
    },
  },
};
