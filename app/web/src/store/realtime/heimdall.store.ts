import { defineStore } from 'pinia'
import * as Comlink from "comlink";
import { Args, Checksum, DBInterface, interpolate, QueryKey, RawArgs } from "@/workers/types/dbinterface";
import { watch, computed, reactive, readonly } from 'vue';
import { useAuthStore } from '../auth.store';

export const useHeimdall = defineStore('heimdall', async () => {
  const authStore = useAuthStore();

  type AtomChecksumByKey = Record<Checksum, QueryKey>;
  const frigg: AtomChecksumByKey = reactive({});

  const bustTanStackCache = (queryKey: QueryKey, latestChecksum: Checksum) => {
    console.log("BUST", queryKey)
    frigg[queryKey] = latestChecksum;
    // TODO bust tanstack once we have it
  };

  const worker = new Worker(new URL("../../workers/webworker.ts", import.meta.url), { type: 'module' });
  const dbInterface: Comlink.Remote<unknown> = Comlink.wrap(worker)
  const db = dbInterface as Comlink.Remote<DBInterface>;

  await db.addListenerBustCache(Comlink.proxy(bustTanStackCache));
  await db.initBifrost("", "");

  const { rows, columns } = await db.testRainbowBridge();
  const data = interpolate(columns, rows);
  console.log("SMOKE RESULTS", data, typeof data[0]?.rowid);

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

  const bifrost = async (kind: string, rawArgs: RawArgs): Promise<unknown> => {
    const args = new Args(rawArgs);
    const queryKey = await db.partialKeyFromKindAndArgs(kind, args);
    const checksum = frigg[queryKey];
    if (!checksum) {
      db.mjolnir(kind, args);
      return {};
    } else
      return await db.get(kind, args, checksum);
  }

  return {
    bifrost,
    frigg: readonly(frigg),
    connectionShouldBeEnabled,
  }

});