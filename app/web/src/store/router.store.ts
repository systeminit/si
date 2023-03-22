import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { RouteLocationNormalizedLoaded } from "vue-router";

export const useRouterStore = defineStore("router", {
  state: () => ({
    currentRoute: null as RouteLocationNormalizedLoaded | null,
  }),
});
