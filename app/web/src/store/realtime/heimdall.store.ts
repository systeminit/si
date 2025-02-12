import { defineStore } from 'pinia'
import * as Comlink from "comlink";
import { DBInterface, interpolate } from "@/workers/types/dbinterface";
import { watch, computed, reactive, readonly } from 'vue';
import { useAuthStore } from '../auth.store';
import { ChangeSetId } from '@/api/sdf/dal/change_set';

export const useHeimdall = defineStore('heimdall', async () => {
  const authStore = useAuthStore();

  const bustTanStackCache = (queryKey: string, latestChecksum: string) => {
    console.log("BUST", queryKey)
    frigg[queryKey] = latestChecksum;
    // TODO
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

  type AtomChecksumByKey = Record<string, string>;
  const frigg: AtomChecksumByKey = reactive({});

  const _bifrost = await db.bifrost;

  const bifrost = async (queryKey: string): Promise<unknown> => {
    const checksum = frigg[queryKey];
    return await _bifrost.get(`${queryKey}|${checksum}`);
  }

  return {
    bifrost,
    frigg: readonly(frigg),
  }

});