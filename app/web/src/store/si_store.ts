import { addStoreHooks, TracingApi } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { reactive, Reactive } from "vue";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useSdfApi } from "./apis";

export function defineSiStore<
  Id extends string,
  SS extends Omit<object, "activated">,
>(id: Id, setupStore: (reactiveSiState: ReactiveSiState) => SS) {
  function realSetupStore() {
    // Create the reactive state that will be used by the store to get resources when activated
    const reactiveSiState = reactive({ isActive: false }) as ReactiveSiState;
    const store = setupStore(reactiveSiState);
    return Object.assign(store, { reactiveSiState });
  }
  const store = defineStore(id, realSetupStore, {
    async onActivated() {
      const realtimeStore = useRealtimeStore();

      // On activate, give the store access to the SDF API
      this.reactiveSiState = {
        isActive: true,
        sdf: useSdfApi({}),
        subscribe: (topic, subscriptions) =>
          realtimeStore.subscribe(this.$id, topic, subscriptions),
      };

      // On deactivate, unsubscribe and remove access to the SDF API
      return () => {
        realtimeStore.unsubscribe(this.$id);
        this.reactiveSiState = { isActive: false };
      };
    },
  });
  return addStoreHooks(undefined, undefined, store);
}

export type ActiveSiState = {
  isActive: true;
  sdf: TracingApi;
  subscribe: (
    topic: Parameters<SubscribeFn>[1],
    subscriptions: Parameters<SubscribeFn>[2],
  ) => ReturnType<SubscribeFn>;
};
export type InactiveSiState = {
  isActive: false;
  sdf?: undefined;
  subscribe?: undefined;
};
export type ReactiveSiState = Reactive<ActiveSiState | InactiveSiState>;

type SubscribeFn = ReturnType<typeof useRealtimeStore>["subscribe"];
