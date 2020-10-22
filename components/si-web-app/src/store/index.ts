import Vue from "vue";
import Vuex from "vuex";
import { Store } from "vuex";
import { application, ApplicationStore } from "./modules/application";
import { secret, SecretStore } from "./modules/secret";
import { system, SystemStore } from "./modules/system";
import { user, UserStore } from "./modules/user";
import { workspace, WorkspaceStore } from "./modules/workspace";
import { billingAccount, BillingAccountStore } from "./modules/billingAccount";
import { loader, LoaderStore } from "./modules/loader";
import { organization, OrganizationStore } from "./modules/organization";
import { editor, EditorStore } from "./modules/editor";

Vue.use(Vuex);

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
  organization: OrganizationStore;
  workspace: WorkspaceStore;
  billingAccount: BillingAccountStore;
  loader: LoaderStore;
  application: ApplicationStore;
  system: SystemStore;
  version: string;
  secret: SecretStore;
}

const store: Store<RootStore> = new Vuex.Store({
  // @ts-ignore - we know its incomplete, but it isn't really
  state: {
    version: "1",
  },
  modules: {
    application,
    system,
    editor,
    user,
    organization,
    workspace,
    billingAccount,
    loader,
    secret,
  },
  strict: debug,
});

export default store;
