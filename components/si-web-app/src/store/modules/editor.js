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
  nodeList: []
})

// getters
const getters = {}

const actions = {
  selectNode({ state, commit }, nodeId) {
    commit('setSelectedNodeId', nodeId)
  },
  addNode({ state, commit }, payload) {
    let nodeObject = new Node(payload.id, payload.name, payload.isEntity);
    commit('addtoNodeList', nodeObject)
  },
  removeNode({ state, commit }, nodeId) {
    commit('removeNodeFromList', nodeId)
  },

}

// mutations
const mutations = {
  setSelectedNodeId(state, nodeId) {
    state.selectedNodeId = nodeId
  },
  addtoNodeList(state, nodeObject) {
    state.nodeList.push(nodeObject)
  },
  removeNodeFromList(state, nodeObject) {
    array.remove(state.nodeList, nodeObject => removeItem.includes(nodeObject.id))
  }
}

export default {
  namespaced: true,
  state,
  getters,
  actions,
  mutations
}