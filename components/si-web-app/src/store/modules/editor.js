import _ from "lodash";

// NodeEditor Cache

class Node {
  constructor(nodeId, nodeName, isEntity, changeSetId) {
    console.log("changesetid", changeSetId);
    this.id = nodeId;
    this.name = nodeName;
    this.status = isEntity; // bool
    this.changeSetId = changeSetId;
  }
}

// initial state
const state = () => ({
  selectedNodeId: String,
  selectedNode: {},
  nodeList: [],
});

// getters
const getters = {
  nodeList(state, _getters, _rootState, rootGetters) {
    let currentChangeSetId = undefined;
    try {
      currentChangeSetId = rootGetters["changeSet/currentId"];
    } catch (err) {
      // Not logged in, or not in a changeSet!
      return state.nodeList;
    }
    if (currentChangeSetId) {
      let nodeList = _.filter(state.nodeList, function(node) {
        if (
          node.changeSetId == "" ||
          node.changeSetId == undefined ||
          node.changeSetId == currentChangeSetId
        ) {
          return true;
        } else {
          return false;
        }
      });
      return nodeList;
    } else {
      return state.nodeList;
    }
  },
};

const actions = {
  // selectNode({ state, commit }, nodeId) {
  //   commit('setSelectedNodeId', nodeId)
  // },
  selectNode({ state, commit }, nodeObject) {
    commit("setSelectedNode", nodeObject);
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
  removeNode({ state, commit }, nodeObject) {
    commit("removeNodeFromList", nodeObject);
  },
};

// mutations
const mutations = {
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
};

export default {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};
