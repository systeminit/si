import Vue from "vue";
import Vuex from "vuex";
import { Store } from "vuex";
import VuexPersistence from "vuex-persist";
import Cookies from "js-cookie";
// @ts-ignore
import editor from "./modules/editor";
// @ts-ignore
import applications from "./modules/applications";
import { user, Authentication } from "./modules/user";

Vue.use(Vuex);

const debug = process.env.NODE_ENV !== "production";

const vuexCookie = new VuexPersistence({
  restoreState: (key, _storage) => Cookies.getJSON(key),
  saveState: async (key, state, _storage): Promise<void> => {
    Cookies.set(key, state, {
      expires: 3,
    });
  },
  modules: ["applications"],
});

const vuexLocal = new VuexPersistence({
  storage: window.localStorage,
  reducer: (state: any) => ({ applications: state.applications }),
});

export interface RootStore {
  user: {
    auth: Authentication;
  };
  applications: {
    id: string;
    name: string;
  };
  editor: {
    selectedNodeId: string;
    selectedNode: Record<string, any>;
    nodeList: Record<string, any>[];
  };
}

const store: Store<RootStore> = new Vuex.Store({
  modules: {
    applications,
    editor,
    user,
  },
  strict: debug,
  plugins: [vuexCookie.plugin, vuexLocal.plugin],
});

export default store;
