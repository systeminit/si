class Application {
  constructor(id, name) {
    this.id = id
    this.name = name
  }
}

// initial state
const state = () => ({
  applicationList: [],
})

// getters
const getters = {
  list (state) {
    return state.applicationList
  }
}

const actions = {
  initializeStore({ state, commit }) {
    let applicationObject = new Application("tmp", "tmp");
    commit('addtoApplicationList', applicationObject)
  },
  addApplication({ state, commit }, payload) {
    let applicationObject = new Application(payload.id, payload.name);
    commit('addtoApplicationList', applicationObject)
  }
}

// mutations
const mutations = {
  addtoApplicationList(state, applicationObject) {
    state.applicationList.push(applicationObject)
  }
}

export default {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
}