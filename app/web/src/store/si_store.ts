import { addStoreHooks, TracingApi } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { reactive } from "vue";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useSdfApi } from "./apis";

type _ActiveSiStore = {
  sdf: TracingApi;
  subscribe(
    topic: Parameters<ReturnType<typeof useRealtimeStore>["subscribe"]>[1],
    subscriptions: Parameters<
      ReturnType<typeof useRealtimeStore>["subscribe"]
    >[2],
  ): ReturnType<ReturnType<typeof useRealtimeStore>["subscribe"]>;
};
export type ActiveSiStore =
  | _ActiveSiStore
  | { [K in keyof _ActiveSiStore]?: undefined };

export function defineSiStore<
  Id extends string,
  SS extends Omit<object, "activated">,
>(id: Id, setupStore: (activated: ActiveSiStore) => SS) {
  function realSetupStore() {
    const activated = reactive<ActiveSiStore>({});
    const store = setupStore(activated);
    return Object.assign(store, { activated });
  }
  const store = defineStore(id, realSetupStore, {
    // On activate, give the store access to the SDF API
    async onActivated() {
      const realtimeStore = useRealtimeStore();
      const sdf = useSdfApi({});
      const subscribe: ActiveSiStore["subscribe"] = (topic, subscriptions) =>
        realtimeStore.subscribe(this.$id, topic, subscriptions);
      Object.assign(this.activated, { sdf, subscribe });
      // On deactivate, unsubscribe and remove access to the SDF API
      return () => {
        realtimeStore.unsubscribe(this.$id);
        Object.assign(this.activated, { sdf: undefined, subscribe: undefined });
      };
    },
  });
  return addStoreHooks(undefined, undefined, store);
}
