import array from 'lodash/array'

// NodeEditor Cache

class Node {
  constructor(nodeId, nodeName, isEntity) {
    this.id = nodeId
    this.name = nodeName
    this.status = isEntity // bool
  }
}

// initial state
const state = () => ({
  selectedNodeId: String,
  selectedNode: {},
  nodeList: []
})

// getters
const getters = {}

const actions = {
  // selectNode({ state, commit }, nodeId) {
  //   commit('setSelectedNodeId', nodeId)
  // },
  selectNode({ state, commit }, nodeObject) {
    commit('setSelectedNode', nodeObject)
  },
  addNode({ state, commit }, payload) {
    if(state.nodeList.some(node => node.id === payload.id)) {
      commit('removeNodeFromList', payload)
    }
    let nodeObject = new Node(payload.id, payload.name, payload.isEntity);
    commit('addtoNodeList', nodeObject)
  },
  removeNode({ state, commit }, nodeObject) {
    commit('removeNodeFromList', nodeObject)
  },
}

// mutations
const mutations = {
  setSelectedNodeId(state, nodeId) {
    state.selectedNodeId = nodeId
  },
  setSelectedNode(state, nodeObject) {
    state.selectedNode = nodeObject
  },
  addtoNodeList(state, nodeObject) {
    state.nodeList.push(nodeObject)
  },
  removeNodeFromList(state, nodeObject) {
    let index = state.nodeList.findIndex(obj => obj.id === nodeObject.id)
    if (index > -1) {
      state.nodeList.splice(index, 1);
    }
  }
}

export default {
  namespaced: true,
  state,
  getters,
  actions,
  mutations
}