import { defineStore } from "pinia";
import _ from "lodash";
import { RouteLocationNormalizedLoaded } from "vue-router";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function castNumericParam(val: any) {
  if (_.isNil(val)) return val;
  const valToCast = _.isArray(val) ? val[0] : val;
  if (parseInt(valToCast).toString() === val) return parseInt(valToCast);
  return valToCast;
}

export const useRouterStore = defineStore("router", {
  state: () => ({
    currentRoute: null as RouteLocationNormalizedLoaded | null,
  }),
  getters: {
    urlSelectedWorkspacePk: (state) => {
      return castNumericParam(state.currentRoute?.params?.workspacePk);
    },
    urlSelectedChangeSetId: (state) => {
      return castNumericParam(state.currentRoute?.params?.changeSetId);
    },
  },
});
