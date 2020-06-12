import Vue from "vue"
import Vuex from "vuex"
import VuexPersistence from 'vuex-persist'
import Cookies from 'js-cookie'
import editor from "./modules/editor"
import applications from "./modules/applications"

Vue.use(Vuex)

const debug = process.env.NODE_ENV !== "production"


const vuexCookie = new VuexPersistence({
  restoreState: (key, storage) => Cookies.getJSON(key),
  saveState: (key, state, storage) =>
    Cookies.set(key, state, {
      expires: 3
    }),
    modules: ['applications'], 
})

const vuexLocal = new VuexPersistence({
  storage: window.localStorage,
  reducer: (state) => ({ applications: state.applications }),
})

const store = new Vuex.Store({
  modules: {
    applications,
    editor,
  },
  strict: debug,
  plugins: [vuexCookie.plugin, vuexLocal.plugin]
})

export default store