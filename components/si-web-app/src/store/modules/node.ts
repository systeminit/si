import { Module, Store } from "vuex";
import _ from "lodash";

import { registry, Props, PropMethod, PropLink, PropObject } from "si-registry";

import { RootStore } from "@/store";
import { graphqlMutation, graphqlQueryListAll } from "@/api/apollo";
import { ChangeSet, Node as GqlNode, NodePosition } from "@/graphql-types";
import { diffEntity, DiffResult } from "@/utils/diff";

interface NodeConstructor {
  id: Node["id"];
  name: Node["name"];
}

export enum NodeType {
  Entity = "Entity",
}

// -- alex tmp --
// interface Socket {
//   id: string;
//   // name: string; // name that could be displayed
//   socketType: NodeType; // input or output
//   position: Position;
//   // data: any; // Data this socket it accessing or exposing.
// }

// -- alex tmp --
// export enum SocketType {
//   Input = "Input",
//   Output = "Output",
// }

// -- alex tmp --
// interface NodeConnection {
//   id: string;
//   socket: Socket; // a socket on this node.
//   path: Socket; // a socket on another node.
// }

// -- alex tmp --
// interface NodeFlags {
//   trackMouse: boolean;
//   selected: boolean;
//   visible: boolean;
// }
//
// Node
// sockets: Socket[]; -- alex tmp --
// connections: NodeConnection[]; -- alex tmp --

interface Node extends GqlNode {
  stack: any[];
  display: Record<string, any>;
}

export interface NodeStore {
  nodes: Node[];
  current: null | Node;
  mouseTrackSelection: null | string;
}

interface AddMutation {
  nodes: Node[];
}

export interface Item {
  entityId: string;
  name: string;
  nodeType: NodeType;
  object: any;
}

interface AddAction {
  items: Item[];
}

interface CreateAction {
  nodeType: NodeType;
  typeName: string;
}

interface CurrentAction {
  node: Node;
}

// -- alex tmp --
interface SetMouseTrackSelectionAction {
  id: string;
}

interface SendActionAction {
  action: string;
}

interface SetFieldValueAction {
  path: (string | number)[];
  value: any;
  map?: boolean;
}

interface SetFieldValueMutation {
  path: (string | number)[];
  value: any;
  map?: boolean;
  stackEntry: string;
}

export interface RegistryProperty {
  path: (string | number)[];
  prop: Props;
  name: string;
  label: string;
  required: boolean;
  repeated: boolean;
  kind: string;
  hidden: boolean;
}

export const debouncedSetFieldValue = _.debounce(async function({
  store,
  path,
  value,
  map,
}: {
  store: Store<RootStore>;
  path: (string | number)[];
  value: any;
  map?: boolean;
}) {
  await store.dispatch("node/setFieldValue", {
    path,
    value,
    map,
  });
},
1000);

export const debouncedSetPosition = _.debounce(async function({
  store,
  nodeId,
  nodePosition,
}: {
  store: Store<RootStore>;
  nodeId: string;
  nodePosition: NodePosition;
}) {
  await store.dispatch("node/setNodePosition", {
    id: nodeId,
    position: nodePosition,
  });
},
100);

export const node: Module<NodeStore, RootStore> = {
  namespaced: true,
  state: {
    nodes: [],
    current: null,
    mouseTrackSelection: null,
  },
  getters: {
    // For the current node, produce the diff between the base state and the current state
    diffCurrent(state, _getters, rootState, _rootGetters): DiffResult {
      const currentNode: Node | null = state.current;
      const currentChangeSet: ChangeSet | null = rootState.changeSet.current;
      if (currentNode && currentChangeSet) {
        if (currentChangeSet?.id) {
          // We have changes, and the node has been saved before
          if (
            currentNode.display[currentChangeSet.id] &&
            currentNode.display["saved"]
          ) {
            const result = diffEntity(
              currentNode.display["saved"],
              currentNode.display[currentChangeSet.id],
            );
            return result || [];
            // We have a change, and the node hasn't been saved
          } else if (currentNode.display[currentChangeSet.id]) {
            let startEntity;
            let finalEntity;
            for (const entity of currentNode.stack) {
              if (!startEntity) {
                startEntity = entity;
              }
              if (!finalEntity) {
                finalEntity = entity;
              }
              const entityCount = parseInt(
                entity.siStorable?.changeSetEntryCount,
                10,
              );
              const startCount = parseInt(
                startEntity.siStorable?.changeSetEntryCount,
                10,
              );
              const endCount = parseInt(
                finalEntity.siStorable?.changeSetEntryCount,
                10,
              );

              if (entityCount < startCount) {
                startEntity = entity;
              }
              if (entityCount > endCount) {
                finalEntity = entity;
              }
            }
            const result = diffEntity(
              startEntity,
              finalEntity,
              //currentNode.display[currentChangeSet.id],
            );
            return result;
          } else {
            return {
              entries: [],
              count: 0,
            };
          }
        }
      }
      return {
        entries: [],
        count: 0,
      };
    },
    current(state): Node {
      // This should probably move to state.selection: node[]. -- alex tmp --
      if (state.current) {
        return state.current;
      } else {
        throw new Error("Cannot get current node; it is not set!");
      }
    },
    mouseTrackSelection(state): string | null {
      // will need refactor, this is a hack to imnplement mouse tracking on new nodes. -- alex tmp --
      return state.mouseTrackSelection;
    },
    list(state, _getters, rootState): Node[] {
      let changeSetId = rootState.changeSet.current?.id;
      if (changeSetId) {
        return _.filter(state.nodes, node => {
          let inChangeSet = _.find(node.stack, item => {
            if (item.siStorable?.changeSetId == changeSetId) {
              return true;
            } else {
              return false;
            }
          });
          if (inChangeSet) {
            return true;
          } else {
            let isSaved = _.find(node.stack, item => {
              if (!item.siStorable?.changeSetId && !item.siStorable?.deleted) {
                if (!item.siStorable?.deleted) {
                  return true;
                } else {
                  return false;
                }
              } else {
                return false;
              }
            });
            if (isSaved) {
              return true;
            } else {
              return false;
            }
          }
        });
      } else {
        return _.filter(state.nodes, node => {
          let savedItem = _.find(node.stack, item => {
            if (!item.siStorable?.changeSetId) {
              if (!item.siStorable?.deleted) {
                return true;
              } else {
                return false;
              }
            } else {
              return false;
            }
          });
          if (savedItem) {
            return true;
          } else {
            return false;
          }
        });
      }
    },
    getNodebyId: state => (nodeId: string): Node => {
      // returns a node by id. need to refactor and remove the ts-ignore. -- alex tmp --
      // @ts-ignore
      return _.find(state.nodes, ["id", nodeId]);
      // throw an error if the node isn't found and return null. -- alex tmp --
    },
    getFieldValue: (_state, getters, rootState) => (path: string[]): any => {
      const currentNode = getters["current"];
      let entity;
      if (rootState.changeSet.current?.id) {
        let currentChangeSetId = rootState.changeSet.current.id;
        if (currentNode.display[currentChangeSetId]) {
          entity = currentNode.display[rootState.changeSet.current.id];
        } else {
          entity = currentNode.display["saved"];
        }
      } else {
        entity = currentNode.display["saved"];
      }

      return _.get(entity, path);
    },
    // prettier-ignore
    propertiesListRepeated: (_state: NodeStore, _getters) => (entityProperty: RegistryProperty, index: number, ): RegistryProperty[] => {
      interface PropEntry {
        prop: Props;
        path: (string | number)[];
      }

      if (entityProperty.kind == "object") {
        let updateField = entityProperty.prop as PropObject;

        const objectProperties: PropEntry[] = updateField.properties.attrs.map(
          prop => {
            return {prop, path: _.clone(entityProperty.path)};
          },
        );
        const result: RegistryProperty[] = [];

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
                return {prop, path: _.clone(path)};
              });
              for (let nProp of newProps) {
                objectProperties.push(nProp);
              }
            }
          } else {
            if (prop.kind() == "object" && prop.repeated == false) {
              const rProp = prop as PropObject;
              let newProps = rProp.properties.attrs.map(prop => {
                return {prop, path: _.clone(path)};
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
      } else {
        let result: RegistryProperty[] = [];
        let path = entityProperty.path;
        path.push(index);
        result.push({
          name: entityProperty.name,
          label: entityProperty.label,
          path,
          prop: entityProperty.prop,
          required: entityProperty.required,
          repeated: entityProperty.repeated,
          kind: entityProperty.kind,
          hidden: entityProperty.hidden,
        });
        return result;
      }
    },
    propertiesList(_state, getters, rootState): RegistryProperty[] {
      const currentNode: Node = getters["current"];
      let entity;
      if (rootState.changeSet.current?.id) {
        let currentChangeSetId = rootState.changeSet.current.id;
        if (currentNode.display[currentChangeSetId]) {
          entity = currentNode.display[rootState.changeSet.current.id];
        } else {
          entity = currentNode.display["saved"];
        }
      } else {
        entity = currentNode.display["saved"];
      }

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
      const result: RegistryProperty[] = [];

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
  mutations: {
    add(state, payload: AddMutation) {
      for (let node of payload.nodes) {
        if (node.stack) {
          const displayData: Record<string, any> = {};
          for (let item of node.stack) {
            if (item.siStorable?.changeSetId) {
              if (displayData[item.siStorable.changeSetId]) {
                let toCheckCount = parseInt(
                  item.siStorable?.changeSetEntryCount,
                  10,
                );
                let currentCheckCount = parseInt(
                  displayData[item.siStorable?.changeSetId].siStorable
                    ?.changeSetEntryCount,
                  10,
                );

                if (toCheckCount > currentCheckCount) {
                  displayData[item.siStorable?.changeSetId] = _.cloneDeep(item);
                }
              } else {
                displayData[item.siStorable.changeSetId] = _.cloneDeep(item);
              }
            } else {
              displayData["saved"] = _.cloneDeep(item);
            }
          }
          node.display = displayData;
        }
      }
      state.nodes = _.unionBy(payload.nodes, state.nodes, "id");
    },
    current(state, payload: Node) {
      let node = payload;
      const displayData: Record<string, any> = {};
      for (let item of node.stack) {
        if (item.siStorable?.changeSetId) {
          if (displayData[item.siStorable.changeSetId]) {
            let toCheckCount = parseInt(
              item.siStorable?.changeSetEntryCount,
              10,
            );
            let currentCheckCount = parseInt(
              displayData[item.siStorable?.changeSetId].siStorable
                ?.changeSetEntryCount,
              10,
            );
            if (toCheckCount > currentCheckCount) {
              displayData[item.siStorable?.changeSetId] = _.cloneDeep(item);
            }
          } else {
            displayData[item.siStorable.changeSetId] = _.cloneDeep(item);
          }
        } else {
          displayData["saved"] = _.cloneDeep(item);
        }
      }
      node.display = displayData;

      state.current = node;
    },
    // This is a hack to implement mouse tracking on node creation. -- alex tmp --
    setMouseTrackSelection(state, payload: string) {
      let nodeId = payload;
      state.mouseTrackSelection = nodeId;
    },
    // This is a hack to implement mouse tracking on node creation. -- alex tmp --
    unsetMouseTrackSelection(state, payload: string) {
      state.mouseTrackSelection = null;
    },
    setFieldValue(state, payload: SetFieldValueMutation) {
      if (!state.current) {
        throw new Error(
          `Cannot set the field value - there is no current node: ${JSON.stringify(
            payload,
          )}`,
        );
      }
      _.set(
        state.current.display[payload.stackEntry],
        payload.path,
        payload.value,
      );
    },
    setNodePosition(state, payload: { id: string; position: NodePosition }) {
      const node = _.find(state.nodes, ["id", payload.id]);
      if (node) {
        node.position = payload.position;
      }
      if (state.current?.id == payload.id) {
        state.current.position = payload.position;
      }
    },
  },
  actions: {
    async sendAction(
      { getters, rootGetters, dispatch },
      payload: SendActionAction,
    ) {
      if (payload.action == "delete") {
        await dispatch("delete");
        return;
      }
      let currentNode = getters["current"];
      let currentChangeSet;
      try {
        currentChangeSet = rootGetters["changeSet/current"];
      } catch (err) {
        await dispatch("changeSet/createDefault", {}, { root: true });
        currentChangeSet = rootGetters["changeSet/current"];
      }
      let entity;
      if (currentNode.display[currentChangeSet.id]) {
        entity = currentNode.display[currentChangeSet.id];
      } else {
        entity = currentNode.display["saved"];
      }
      await graphqlMutation({
        typeName: entity.siStorable.typeName,
        methodName: payload.action,
        variables: {
          // How is this not a bug? If we haven't made a change already, we won't have an
          // object with the ID. I bet we still have to pass the changeSet ID through.
          id: entity.id,
          changeSetId: currentChangeSet.id,
        },
      });
      await dispatch(
        "changeSet/get",
        { changeSetId: currentChangeSet.id },
        { root: true },
      );
    },
    async setFieldValue(
      { commit, getters, rootState },
      payload: SetFieldValueAction,
    ) {
      let currentNode = getters["current"];
      let stackEntry = "saved";
      if (rootState.changeSet.current?.id) {
        let currentChangeSetId = rootState.changeSet.current.id;
        if (currentNode.display[currentChangeSetId]) {
          stackEntry = currentChangeSetId;
        }
      }
      commit("setFieldValue", { stackEntry, ...payload });
    },
    async create({ dispatch }, payload: CreateAction) {
      if (payload.nodeType == NodeType.Entity) {
        await dispatch(
          "entity/create",
          {
            typeName: payload.typeName,
          },
          { root: true },
        );
      }
    },
    current({ commit }, payload: CurrentAction) {
      commit("current", payload.node);
    },
    // This is a hack to implement mouse tracking on node creation. -- alex tmp --
    setMouseTrackSelection({ commit }, payload: SetMouseTrackSelectionAction) {
      commit("setMouseTrackSelection", payload.id);
    },
    // This is a hack to implement mouse tracking on node creation. -- alex tmp --
    unsetMouseTrackSelection({ commit }) {
      commit("unsetMouseTrackSelection");
    },
    setNodePosition(
      { commit },
      payload: { position: NodePosition; id: string },
    ) {
      commit("setNodePosition", payload);
    },
    async delete({ getters, dispatch, rootGetters }) {
      let currentNode = getters["current"];
      let changeSetId;
      try {
        changeSetId = rootGetters["changeSet/current"].id;
      } catch (err) {
        await dispatch("changeSet/createDefault", {}, { root: true });
        changeSetId = rootGetters["changeSet/current"].id;
      }

      let displayCurrentNode;
      if (currentNode.display[changeSetId]) {
        displayCurrentNode = currentNode.display[changeSetId];
      } else {
        displayCurrentNode = currentNode.display["saved"];
      }
      if (currentNode.nodeType == NodeType.Entity) {
        await dispatch(
          "entity/delete",
          {
            typeName: displayCurrentNode.siStorable?.typeName,
            id: displayCurrentNode.id,
          },
          { root: true },
        );
      }
    },
    async add(
      { commit, state, rootState, rootGetters, dispatch },
      payload: AddAction,
    ) {
      for (let item of payload.items) {
        let existingNode = _.cloneDeep(
          _.find(state.nodes, ["entityId", item.entityId]),
        );
        if (existingNode) {
          existingNode.stack = _.unionBy(
            [item.object],
            existingNode.stack,
            "id",
          );
          existingNode.name = item.name;
          commit("add", { nodes: [existingNode] });
          if (state.current?.entityId == item.entityId) {
            commit("current", existingNode);
          }
        } else {
          let workspace = rootGetters["workspace/current"];
          let profile = rootGetters["user/profile"];
          // TODO: You have to create a node, and you need to do loader? Tricky!
          //       probably finish the whole create/entity process.
          const result = await graphqlMutation({
            typeName: "node",
            methodName: "create",
            variables: {
              name: item.name,
              displayName: item.name,
              siProperties: {
                workspaceId: workspace.id,
                billingAccountId: profile.billingAccount?.id,
                organizationId: profile.organization?.id,
              },
              entityId: item.entityId,
              position: {
                x: 0,
                y: 0,
              },
              sockets: [
                { name: "input", kind: "INPUT" },
                { name: "output", kind: "OUTPUT" },
              ],
              nodeKind: "ENTITY",
            },
          });
          const newNode = result.item;
          newNode["stack"] = [item.object];
          commit("add", {
            nodes: [newNode],
          });
          if (state.current?.entityId == item.entityId) {
            commit("current", newNode);
          }
          if (item.object.siStorable?.typeName != "application_entity") {
            let application = rootState.application.current;
            if (application) {
              let applicationNode = _.find(state.nodes, [
                "entityId",
                application.id,
              ]);
              if (applicationNode) {
                await dispatch(
                  "edge/create",
                  {
                    tailVertex: {
                      id: applicationNode.id,
                      socket: "output",
                      typeName: applicationNode.stack[0].siStorable?.typeName,
                    },
                    headVertex: {
                      id: newNode.id,
                      socket: "input",
                      typeName: newNode.stack[0].siStorable?.typeName,
                    },
                    bidirectional: true,
                  },
                  { root: true },
                );
              }
            }
          } else {
            console.log("you are an application!", { item, newNode });
          }
        }
      }
    },
    async load({ commit }): Promise<void> {
      const nodes: GqlNode[] = await graphqlQueryListAll({
        typeName: "node",
      });
      if (nodes.length > 0) {
        commit("add", { nodes });
      }
    },
  },
};
