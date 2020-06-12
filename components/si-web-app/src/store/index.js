import Vue from 'vue'
import Vuex from 'vuex'
import editor from './modules/editor'
import applications from './modules/applications'

Vue.use(Vuex)

const debug = process.env.NODE_ENV !== 'production'

export default new Vuex.Store({
  modules: {
    applications,
    editor,
  },
  strict: debug,
})