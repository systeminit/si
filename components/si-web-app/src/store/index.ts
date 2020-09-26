import Vue from "vue";
import Vuex from "vuex";
import { Store } from "vuex";
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
import { organization, OrganizationStore } from "./modules/organization";

Vue.use(Vuex);

const vuexLocal = new VuexPersistence<RootStore>({
  storage: window.localStorage,
});

const debug = process.env.NODE_ENV !== "production";

export class GetCurrentError extends Error {
  constructor(modelName: string) {
    let message = `no current ${modelName}`;
    super(message);
    this.name = "GetCurrentError";
  }
}

export interface AddMutation<T> {
  items: T[];
}

export interface RootStore {
  editor: EditorStore;
  user: UserStore;
  entity: EntityStore;
  changeSet: ChangeSetStore;
  organization: OrganizationStore;
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
    organization,
    workspace,
    billingAccount,
    node,
    eventLog,
    resource,
    editSession,
    loader,
  },
  strict: debug,
  plugins: [persistEdits, persistNodes, vuexLocal.plugin],
});

export default store;
