import { defineStore } from 'pinia'
import * as Comlink from "comlink";
import { Args, AtomDocument, Checksum, DBInterface, interpolate, NOROW, QueryKey, RawArgs } from "@/workers/types/dbinterface";
import { watch, computed  } from 'vue';
import { useAuthStore } from '../auth.store';
import { ChangeSetId } from '@/api/sdf/dal/change_set';

export const useHeimdall = defineStore('heimdall', async () => {
  const authStore = useAuthStore();

  type AtomChecksumByKey = Record<Checksum, QueryKey>;

  const bustTanStackCache = (queryKey: QueryKey, latestChecksum: Checksum) => {
    console.log("BUST", queryKey)
    // TODO bust tanstack once we have it
  };

  const worker = new Worker(new URL("../../workers/webworker.ts", import.meta.url), { type: 'module' });
  const db: Comlink.Remote<DBInterface> = Comlink.wrap(worker)

  await db.addListenerBustCache(Comlink.proxy(bustTanStackCache));
  await db.initBifrost("", "");

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

  const bifrost = async (changeSetId: ChangeSetId, kind: string, rawArgs: RawArgs): Promise<typeof NOROW | AtomDocument> => {
    const args = new Args(rawArgs);
    const maybeAtomDoc = await db.get(changeSetId, kind, args);
    if (maybeAtomDoc === NOROW)
      db.mjolnir(changeSetId, kind, args);
    return maybeAtomDoc
  };

  // cold start
  const niflheim = async (changeSetId: ChangeSetId) => {
    const frigg = await fetch("newarch/frigg");
    const [localChecksums, remoteChecksums] = await Promise.all([
      await db.atomChecksumsFor(changeSetId),
      await frigg.json() as Record<QueryKey, Checksum>,
    ]);
    Object.entries(remoteChecksums).map(async ([key, checksum]) => {
      const local = localChecksums[key];
      if (!local || local !== checksum) {
        const { kind, args } = await db.kindAndArgsFromKey(key);
        db.mjolnir(changeSetId, kind, args);
      }
    });
  };

  await db.fullDiagnosticTest();

  return {
    niflheim,
    bifrost,
    connectionShouldBeEnabled,
  }

});