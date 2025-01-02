import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { RouteLocationNormalizedLoaded } from "vue-router";

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
    currentRoute: null as RouteLocationNormalizedLoaded | null,
  }),
});
