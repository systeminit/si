import Vue from "vue";
import Vuex, { Payload } from "vuex";
import { Store } from "vuex";
import localforage from "localforage";
import VuexPersistence from "vuex-persist";
//import Cookies from "js-cookie";
import { editor, EditorStore } from "./modules/editor";
import { application, ApplicationStore } from "./modules/application";
import { edge, EdgeStore } from "./modules/edge";
import { system, SystemStore } from "./modules/system";
import { entity, EntityStore } from "./modules/entity";
import { editSession, EditSessionStore } from "./modules/editSession";
import { user, UserStore } from "./modules/user";
import { changeSet, ChangeSetStore } from "./modules/changeSet";
import { persistEdits } from "./plugins/persistEdits";
import { persistNodes } from "./plugins/persistNodes";
import { workspace, WorkspaceStore } from "./modules/workspace";
import { billingAccount, BillingAccountStore } from "./modules/billingAccount";
import { node, NodeStore } from "./modules/node";
import { eventLog, EventLogStore } from "./modules/eventLog";
import { loader, LoaderStore } from "./modules/loader";
import { resource, ResourceStore } from "./modules/resource";

Vue.use(Vuex);

const debug = process.env.NODE_ENV !== "production";

const localForage = localforage.createInstance({
  driver: localforage.INDEXEDDB,
  name: "si-store",
  storeName: "vuex",
  description: "the vuex state, stored",
});

// @ts-ignore types?
const vuexPersist = new VuexPersistence<any>({
  storage: localForage,
  asyncStorage: true,
});

//const vuexCookie = new VuexPersistence({
//  restoreState: (key, _storage) => Cookies.getJSON(key),
//  saveState: async (key, state, _storage): Promise<void> => {
//    Cookies.set(key, state, {
//      expires: 3,
//    });
//  },
//});
//
//const vuexLocal = new VuexPersistence({
//  storage: window.localStorage,
//  reducer: (state: any) => ({ applications: state.applications }),
//});

export interface RootStore {
  editor: EditorStore;
  user: UserStore;
  entity: EntityStore;
  changeSet: ChangeSetStore;
  workspace: WorkspaceStore;
  billingAccount: BillingAccountStore;
  node: NodeStore;
  loader: LoaderStore;
  application: ApplicationStore;
  edge: EdgeStore;
  system: SystemStore;
  eventLog: EventLogStore;
  resource: ResourceStore;
  editSession: EditSessionStore;
  version: string;
}

const store: Store<RootStore> = new Vuex.Store({
  // @ts-ignore - we know its incomplete, but it isn't really
  state: {
    version: "1",
  },
  modules: {
    edge,
    application,
    system,
    editor,
    user,
    entity,
    changeSet,
    workspace,
    billingAccount,
    node,
    eventLog,
    resource,
    editSession,
    loader,
  },
  strict: debug,
  plugins: [persistEdits, persistNodes, vuexPersist.plugin],
});

export default store;
