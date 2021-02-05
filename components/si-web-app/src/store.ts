import Vue from "vue";
import Vuex from "vuex";
import { Store } from "vuex";
import { session } from "@/store/modules/session";
import { application } from "@/store/modules/application";

export type SiVuexStore = Store<RootStore>;

Vue.use(Vuex);

let debug = process.env.NODE_ENV !== "production";

export interface RootStore {
  version: string;
}

export const storeData = {
  state: {
    version: "1",
  },
  modules: {
    session,
    application,
  },
  strict: debug,
};
