import { defineStore } from 'pinia'
import * as Comlink from "comlink";
import {  AtomDocument, Checksum, DBInterface, interpolate, NOROW, QueryKey, Id } from "@/workers/types/dbinterface";
import { watch, computed  } from 'vue';
import { useAuthStore } from '../auth.store';
import { ChangeSetId } from '@/api/sdf/dal/change_set';

export const useHeimdall = defineStore('heimdall', () => {
  const authStore = useAuthStore();
  if (!authStore.selectedOrDefaultAuthToken) throw new Error("Missing Auth Token");

  const bustTanStackCache = (queryKey: QueryKey, latestChecksum: Checksum) => {
    console.log("BUST", queryKey)
    // TODO bust tanstack once we have it
  };

  const worker = new Worker(new URL("../../workers/webworker.ts", import.meta.url), { type: 'module' });
  const db: Comlink.Remote<DBInterface> = Comlink.wrap(worker)

  // PSA: these are not await'd
  // but stuff happens in here we do need to wait for
  // figure that out :sweat:
  db.addListenerBustCache(Comlink.proxy(bustTanStackCache));
  db.initBifrost("", "");
  db.setBearer(authStore.selectedOrDefaultAuthToken)

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

  const bifrost = async (workspaceId: string, changeSetId: ChangeSetId, kind: string, id: Id): Promise<typeof NOROW | AtomDocument> => {
    const maybeAtomDoc = await db.get(changeSetId, kind, id);
    if (maybeAtomDoc === NOROW)
      db.mjolnir(workspaceId, changeSetId, kind, id);
    return maybeAtomDoc
  };

  // cold start
  const niflheim = async (workspaceId: string, changeSetId: ChangeSetId) => {
    await db.niflheim(workspaceId, changeSetId);
  };

  const fullDiagnosticTest = async () => {
    await db.fullDiagnosticTest();
  }

  return {
    niflheim,
    bifrost,
    fullDiagnosticTest,
    connectionShouldBeEnabled,
  }

});