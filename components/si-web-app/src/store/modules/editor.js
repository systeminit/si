// NodeEditor Cache

// initial state
const state = () => ({
  selectedNodeId: String
})

// getters
const getters = {}

const actions = {
  selectNode ({ state, commit }, nodeId) {
    console.log("store triggered")
    commit('setSelectedNodeId', nodeId)
  }
}

// mutations
const mutations = {
  setSelectedNodeId (state, nodeId) {
    console.log("mutation triggered")
    state.selectedNodeId = nodeId
  }
}

export default {
  namespaced: true,
  state,
  getters,
  actions,
  mutations
}