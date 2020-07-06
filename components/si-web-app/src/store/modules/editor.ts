import { Module, Store } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";
import { EntityProperty } from "./entity";

export interface EditorStore {
  selectedNodeId: undefined | String;
  selectedNode: undefined | Node;
  nodeList: Node[];
  mode: "view" | "edit";
  editTree: Record<string, any>;
  isSaving: boolean;
  editSaveError: undefined | Error;
}

export class Node {
  id: string;
  name: string;
  status: boolean;
  changeSetId: string;

  constructor(
    nodeId: Node["id"],
    nodeName: Node["name"],
    isEntity: Node["status"],
    changeSetId: Node["changeSetId"],
  ) {
    this.id = nodeId;
    this.name = nodeName;
    this.status = isEntity; // bool
    this.changeSetId = changeSetId;
  }
}

export const debouncedFieldValueSet = _.debounce(async function({
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
  await store.dispatch("editor/setEditValue", {
    path,
    value,
    map,
  });
},
100);

export const editor: Module<EditorStore, RootStore> = {
  namespaced: true,
  state: {
    selectedNodeId: undefined,
    selectedNode: undefined,
    nodeList: [],
    editTree: {},
    mode: "view",
    isSaving: false,
    editSaveError: undefined,
  },
  getters: {
    propertiesList(state, _getters, _rootState, rootGetters): EntityProperty[] {
      return rootGetters["entity/propertiesList"](state.selectedNode?.id);
    },
    editEntry(state): Record<string, any> {
      const selectedNodeId = state.selectedNode?.id;
      if (selectedNodeId) {
        let editTree = state.editTree;
        let result = editTree[selectedNodeId];
        return result;
      } else {
        throw new Error(
          "Cannot get edit tree entry, because no node was selected first",
        );
      }
    },
    getEditValue: state => (path: string[]): any => {
      const selectedNodeId = state.selectedNode?.id;
      if (!selectedNodeId) {
        throw new Error("Cannot get edit value; node is not selected");
      }
      return _.get(state.editTree[selectedNodeId], path);
    },
    nodeList(state, _getters, _rootState, rootGetters) {
      let currentChangeSetId: undefined | string = undefined;
      try {
        currentChangeSetId = rootGetters["changeSet/currentId"];
      } catch (err) {
        // Not logged in, or not in a changeSet!
        return state.nodeList;
      }
      if (currentChangeSetId) {
        // Collect all the nodes in our changeset, keeping only the one with the latest change set entry
        let latestChangeSetNodes: Record<
          string,
          { changeSetEntryCount: Number; node: Node }
        > = {};
        const nodeList: Node[] = [];
        for (const node of state.nodeList) {
          if (node.changeSetId == currentChangeSetId) {
            let matchResults = /^change_set:.+:(\d+):(.+:.+)$/.exec(node.id);
            if (matchResults) {
              const realId = matchResults[2];
              const changeSetEntryCount = parseInt(matchResults[1]);
              if (
                latestChangeSetNodes[realId] == undefined ||
                changeSetEntryCount >
                  latestChangeSetNodes[realId]?.changeSetEntryCount
              ) {
                latestChangeSetNodes[realId] = { changeSetEntryCount, node };
              }
            } else {
              throw new Error(
                "Node is in a change set, but its ID is malformed! This is a bug!",
              );
            }
          }
        }
        for (const meta of Object.values(latestChangeSetNodes)) {
          nodeList.push(meta.node);
        }
        for (const node of state.nodeList) {
          if (!latestChangeSetNodes[node.id] && !node.changeSetId) {
            nodeList.push(node);
          }
        }
        return nodeList;
      } else {
        return state.nodeList;
      }
    },
  },
  actions: {
    async editNode({ commit, state, rootGetters, dispatch }) {
      let entity = rootGetters["entity/get"](state.selectedNode?.id);
      // This is a dirty hack - it ensures we always have the latest data.
      await dispatch(
        "entity/get",
        { id: entity["id"], typeName: entity["siStorable"]["typeName"] },
        { root: true },
      );
      entity = rootGetters["entity/get"](state.selectedNode?.id);
      commit("addEditNode", _.cloneDeep(entity));
    },
    async addEditNode({ commit }, entity) {
      commit("addEditNode", _.cloneDeep(entity));
    },
    async setEditValue(
      { commit, state },
      { path, value, map }: { path: string[]; value: any; map?: boolean },
    ) {
      const selectedNodeId = state.selectedNode?.id;
      if (!selectedNodeId) {
        throw new Error("Cannot set edit value; node is not selected");
      }
      commit("setEditValue", { path, value, map });
    },
    async selectNode({ commit, dispatch }, nodeObject) {
      commit("setSelectedNode", nodeObject);
      // TODO: this should make sure its properties are loaded up
      await dispatch("editNode");
    },
    addNode({ state, commit }, payload) {
      if (state.nodeList.some(node => node.id === payload.id)) {
        commit("removeNodeFromList", payload);
      }
      let nodeObject = new Node(
        payload.id,
        payload.name,
        payload.isEntity,
        payload.changeSetId,
      );
      commit("addToNodeList", nodeObject);
    },
    removeNode({ commit }, nodeObject) {
      commit("removeNodeFromList", nodeObject);
    },
    modeSwitch({ commit, state, rootState, dispatch }) {
      if (state.mode == "view") {
        if (rootState.changeSet.current) {
          commit("setMode", "edit");
        } else {
          dispatch("changeSet/createDefault", {}, { root: true }).then(() => {
            commit("setMode", "edit");
          });
        }
      } else {
        commit("setMode", "view");
      }
    },
  },
  mutations: {
    setIsSaving(state, saving: boolean) {
      state.isSaving = saving;
    },
    setEditSaveError(state, error: Error) {
      state.editSaveError = error;
    },
    setMode(state, mode: EditorStore["mode"]) {
      state.mode = mode;
    },
    addEditNode(state, entity) {
      let id = `${entity.id}`;
      state.editTree[id] = entity;
    },
    setEditValue(
      state,
      { path, value }: { path: string[]; value: any; map?: boolean },
    ) {
      const selectedNodeId = state.selectedNode?.id;
      if (!selectedNodeId) {
        throw new Error(
          "Cannot set edit value in mutation; node is not selected",
        );
      }
      _.set(state.editTree[selectedNodeId], path, value);
    },
    setSelectedNodeId(state, nodeId) {
      state.selectedNodeId = nodeId;
    },
    setSelectedNode(state, nodeObject) {
      state.selectedNode = nodeObject;
    },
    addToNodeList(state, nodeObject) {
      state.nodeList.push(nodeObject);
    },
    removeNodeFromList(state, nodeObject) {
      let index = state.nodeList.findIndex(obj => obj.id === nodeObject.id);
      if (index > -1) {
        state.nodeList.splice(index, 1);
      }
    },
  },
};
