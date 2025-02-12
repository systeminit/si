import { defineStore } from 'pinia'
import * as Comlink from "comlink";
import { DBInterface, interpolate } from "@/workers/types/dbinterface";
import { watch, computed } from 'vue';
import { useAuthStore } from '../auth.store';

export const useHeimdall = defineStore('heimdall', async () => {
  const authStore = useAuthStore();

  const bustTanStackCache = (key: string) => {
    // TODO
    console.log("BUST", key)
  };

  const worker = new Worker(new URL("../../workers/webworker.ts", import.meta.url), { type: 'module' });
  const dbInterface: Comlink.Remote<unknown> = Comlink.wrap(worker)
  const db = dbInterface as Comlink.Remote<DBInterface>;

  await db.addListenerBustCache(Comlink.proxy(bustTanStackCache));
  await db.initBifrost("", "");

  const { rows, columns } = await db.testRainbowBridge();
  console.log("SMOKE RESULTS", interpolate(columns, rows));

  const connectionShouldBeEnabled = computed(
    () =>
      authStore.userIsLoggedInAndInitialized &&
      authStore.selectedWorkspaceToken,
  );

  watch(
    connectionShouldBeEnabled,
    () => {
      if (connectionShouldBeEnabled.value) {
        db.bifrostReconnect();
      } else {
        db.bifrostClose();
      }
    },
    { immediate: true },
  );

  return {
    bifrost: db.bifrost,
  }

});