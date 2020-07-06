import { Module } from "vuex";
import _ from "lodash";
import { registry, Props, PropMethod, PropLink, PropObject } from "si-registry";

import { generateName } from "@/api/names";
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

interface AddEntitiesToWorkspacePayload {
  workspaceId: string;
  partial: boolean;
  entities: Entity[];
}

interface CreateEntityPayload {
  typeName: string;
  data: {
    name?: string;
    description?: string;
    displayName?: string;
    [field: string]: any;
  };
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
    propertiesListRepeated: (_state: EntityStore, getters) => (
      entityProperty: EntityProperty,
      index: number,
    ): EntityProperty[] => {
      interface PropEntry {
        prop: Props;
        path: (string | number)[];
      }

      let updateField = entityProperty.prop as PropObject;

      const objectProperties: PropEntry[] = updateField.properties.attrs.map(
        prop => {
          return { prop, path: _.clone(entityProperty.path) };
        },
      );
      const result: EntityProperty[] = [];

      for (const propEntry of objectProperties) {
        let path = propEntry.path;
        let prop = propEntry.prop;
        path.push(index);
        path.push(prop.name);

        if (prop.kind() == "link") {
          let cprop = prop as PropLink;
          const realProp = cprop.lookupMyself();

          result.push({
            name: prop.name,
            label: prop.label,
            path,
            prop: realProp,
            required: prop.required,
            repeated: prop.repeated,
            kind: realProp.kind(),
            hidden: prop.hidden,
          });
          if (realProp.kind() == "object" && prop.repeated == false) {
            const rProp = realProp as PropObject;
            let newProps = rProp.properties.attrs.map(prop => {
              return { prop, path: _.clone(path) };
            });
            for (let nProp of newProps) {
              objectProperties.push(nProp);
            }
          }
        } else {
          if (prop.kind() == "object" && prop.repeated == false) {
            const rProp = prop as PropObject;
            let newProps = rProp.properties.attrs.map(prop => {
              return { prop, path: _.clone(path) };
            });
            for (let nProp of newProps) {
              objectProperties.push(nProp);
            }
          }
          result.push({
            name: prop.name,
            label: prop.label,
            path,
            prop,
            required: prop.required,
            repeated: prop.repeated,
            kind: prop.kind(),
            hidden: prop.hidden,
          });
        }
      }
      // This groups things according to their nesting, so we can just
      // walk the results and have everything in the proper order.
      const grouped = _.groupBy(result, value => {
        if (value.kind == "object") {
          return value.path;
        } else {
          return value.path.slice(0, -1);
        }
      });
      return _.flatten(Object.values(grouped));
    },

    propertiesList: (_state: EntityStore, getters) => (
      id: string,
    ): EntityProperty[] => {
      const entity: Entity = getters["get"](id);
      const typeName = entity.siStorable?.typeName;
      if (!typeName) {
        throw new Error(
          "Cannot generate properties list for item without a typeName",
        );
      }

      const registryObject = registry.get(typeName);
      const updateMethod = registryObject.methods.getEntry(
        "update",
      ) as PropMethod;
      const updateField = updateMethod.request.properties.getEntry(
        "update",
      ) as PropObject;

      interface PropEntry {
        prop: Props;
        path: string[];
      }

      const objectProperties: PropEntry[] = updateField.properties.attrs.map(
        prop => {
          return { prop, path: [] };
        },
      );
      const result: EntityProperty[] = [];

      for (const propEntry of objectProperties) {
        let path = propEntry.path;
        let prop = propEntry.prop;
        path.push(prop.name);

        if (prop.kind() == "link") {
          let cprop = prop as PropLink;
          const realProp = cprop.lookupMyself();

          result.push({
            name: prop.name,
            label: prop.label,
            path,
            prop: realProp,
            required: prop.required,
            repeated: prop.repeated,
            kind: realProp.kind(),
            hidden: prop.hidden,
          });
          if (realProp.kind() == "object" && prop.repeated == false) {
            const rProp = realProp as PropObject;
            let newProps = rProp.properties.attrs.map(prop => {
              return { prop, path: _.clone(path) };
            });
            for (let nProp of newProps) {
              objectProperties.push(nProp);
            }
          }
        } else {
          if (prop.kind() == "object" && prop.repeated == false) {
            const rProp = prop as PropObject;
            let newProps = rProp.properties.attrs.map(prop => {
              return { prop, path: _.clone(path) };
            });
            for (let nProp of newProps) {
              objectProperties.push(nProp);
            }
          }
          result.push({
            name: prop.name,
            label: prop.label,
            path,
            prop,
            required: prop.required,
            repeated: prop.repeated,
            kind: prop.kind(),
            hidden: prop.hidden,
          });
        }
      }
      // This groups things according to their nesting, so we can just
      // walk the results and have everything in the proper order.
      const grouped = _.groupBy(result, value => {
        if (value.kind == "object") {
          return value.path;
        } else {
          return value.path.slice(0, -1);
        }
      });
      return _.flatten(Object.values(grouped));
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
      const workspaceId = rootGetters["user/currentWorkspaceId"];
      const changeSetId = rootGetters["changeSet/currentId"];
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
      if (payload.hypotheticalState) {
        console.log("doing the hypotheticalState", { payload });
        _.set(
          entity,
          payload.hypotheticalState.path,
          payload.hypotheticalState.value,
        );
      }
      const addPayload: AddEntitiesToWorkspacePayload = {
        workspaceId,
        partial: false,
        entities: [entity],
      };
      commit("add", addPayload);
      let node = {
        id: entity["id"],
        name: entity["name"],
        isEntity: true,
        changeSetId: changeSetId,
      };
      await dispatch("editor/addEditNode", entity, { root: true });
      await dispatch("editor/addNode", node, { root: true });
      await dispatch("editor/selectNode", node, { root: true });
    },

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
      if (!variables.name) {
        let name = generateName();
        variables.name = name;
        variables.displayName = name;
        variables.description = name;
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
      const node = {
        id: entity["id"],
        name: entity["name"],
        isEntity: true,
        changeSetId: changeSetId,
      };
      await dispatch("editor/addEditNode", entity, { root: true });
      await dispatch("editor/addNode", node, { root: true });
      await dispatch("editor/selectNode", node, { root: true });
    },
    async get(
      { state, commit, rootGetters, dispatch },
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
        await dispatch("editor/addEditNode", entityGetResult["item"], {
          root: true,
        });
      }
    },
    async load({ commit, dispatch, rootGetters }): Promise<void> {
      let workspaceId = rootGetters["user/currentWorkspaceId"];
      let changeSetId = undefined;
      try {
        changeSetId = rootGetters["changeSet/currentId"];
      } catch (e) {
        console.log("caught an error getting changeset id in enttiy load", {
          e,
        });
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
          await dispatch("editor/addEditNode", entity, { root: true });
          await dispatch(
            "editor/addNode",
            {
              id: entity["id"],
              name: entity["name"],
              isEntity: true,
              changeSetId: entity["siStorable"]["changeSetId"],
              typeName: entity["siStorable"]["typeName"],
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
