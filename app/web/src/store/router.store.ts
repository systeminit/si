import { defineStore } from "pinia";
import _ from "lodash";
import { RouteLocationNormalizedLoaded } from "vue-router";

export const useRouterStore = defineStore("router", {
  state: () => ({
    currentRoute: null as RouteLocationNormalizedLoaded | null,
  }),
});
