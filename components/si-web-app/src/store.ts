import Vue from "vue";
import Vuex from "vuex";
import { Store } from "vuex";
import { session } from "@/store/modules/session";

export type SiVuexStore = Store<RootStore>;

Vue.use(Vuex);

const debug = process.env.NODE_ENV !== "production";

export interface RootStore {
  version: string;
}

export const storeData = {
  state: {
    version: "1",
  },
  modules: {
    session,
  },
  strict: debug,
};
