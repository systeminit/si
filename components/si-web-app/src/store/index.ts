import Vue from "vue";
import Vuex from "vuex";
import { Store } from "vuex";
import VuexPersistence from "vuex-persist";
import Cookies from "js-cookie";
// @ts-ignore
import { editor, EditorStore } from "./modules/editor";
// @ts-ignore
import applications from "./modules/applications";
import { entity, EntityStore } from "./modules/entity";
import { user, UserStore } from "./modules/user";
import { changeSet, ChangeSetStore } from "./modules/changeSet";
import { persistEdits } from "./plugins/persistEdits";
import { workspace, WorkspaceStore } from "./modules/workspace";
import { billingAccount, BillingAccountStore } from "./modules/billingAccount";
import { node, NodeStore } from "./modules/node";
import { loader, LoaderStore } from "./modules/loader";

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
  editor: EditorStore;
  applications: {
    applicationList: any[];
  };
  user: UserStore;
  entity: EntityStore;
  changeSet: ChangeSetStore;
  workspace: WorkspaceStore;
  billingAccount: BillingAccountStore;
  node: NodeStore;
  loader: LoaderStore;
  version: string;
}

const store: Store<RootStore> = new Vuex.Store({
  // @ts-ignore - we know its incomplete, but it isn't really
  state: {
    version: "1",
  },
  modules: {
    applications,
    editor,
    user,
    entity,
    changeSet,
    workspace,
    billingAccount,
    node,
    loader,
  },
  strict: debug,
  plugins: [vuexCookie.plugin, vuexLocal.plugin, persistEdits],
});

export default store;
