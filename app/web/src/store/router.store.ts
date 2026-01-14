import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { RouteLocationNormalizedLoaded, RouteLocationAsRelativeGeneric } from "vue-router";
import router from "@/router";
import { ChangeSetId } from "@/api/sdf/dal/change_set";

/**
 * PSA: we are using this `currentRoute` to denote the route we are navigating TO!
 *
 * When you do `router.push` with new parameters, all the components in the new route must mount prior
 * to the route (and URL) _actually_ becoming the new `currentRoute`
 *
 * This means that a first router.push/replace with a new changeset param will be overriden
 * by a second router.push/replace that happens in beforeMount/onMount that omit `params`,
 * because vue-router grabs the data from `currentRoute` which is "old" because we haven't
 * landed on the "new" route yet.
 */
export const useRouterStore = defineStore("router", {
  state: () => ({
    currentRoute: null as RouteLocationNormalizedLoaded | RouteLocationAsRelativeGeneric | null,
  }),
  actions: {
    replace(originChangeSetId: ChangeSetId | undefined, location: RouteLocationAsRelativeGeneric) {
      // if you're not operating on the same change set we are viewing, you can't change the router/URL
      if (!location.name && this.currentRoute) location.name = this.currentRoute.name;
      if (this.currentRoute?.params?.changeSetId === originChangeSetId) {
        router.replace(location);
        this.currentRoute = location;
      }
    },
    push(originChangeSetId: ChangeSetId | undefined, location: RouteLocationAsRelativeGeneric) {
      // if you're not operating on the same change set we are viewing, you can't change the router/URL
      if (this.currentRoute?.params?.changeSetId === originChangeSetId) {
        router.push(location);
        this.currentRoute = location;
      }
    },
  },
});
