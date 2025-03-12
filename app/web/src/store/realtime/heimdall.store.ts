import { defineStore } from "pinia";
import * as Comlink from "comlink";
import { watch, computed, reactive } from "vue";
import {
  DBInterface,
  Id,
  BustCacheFn,
} from "@/workers/types/dbinterface";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { useAuthStore } from "../auth.store";
import { useQueryClient } from "@tanstack/vue-query";

export const useHeimdall = defineStore("heimdall", () => {
  const authStore = useAuthStore();

  const coldStarts: Record<string, boolean> = reactive({});

  if (!authStore.selectedOrDefaultAuthToken)
    throw new Error("Missing Auth Token");

  const queryClient = useQueryClient();
  const bustTanStackCache: BustCacheFn = (workspaceId: string, changeSetId: string, kind: string, id: string) => {
    const queryKey = [workspaceId, changeSetId, kind, id];
    console.log("💥 bust cache for", queryKey);
    queryClient.invalidateQueries({ queryKey })
  };

  const worker = new Worker(
    new URL("../../workers/webworker.ts", import.meta.url),
    { type: "module" },
  );
  const db: Comlink.Remote<DBInterface> = Comlink.wrap(worker);

  // PSA: these are not await'd
  // but stuff happens in here we do need to wait for
  // figure that out :sweat:
  db.addListenerBustCache(Comlink.proxy(bustTanStackCache));
  db.setBearer(authStore.selectedOrDefaultAuthToken);

  db.initBifrost();

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

  /**
   * PSA, comlink isn't able to serialize a symbol over the wire... 
   * So we're using -1 as a replacement for NOROW on this side of the fence...
   */

  const bifrost = async <T>(
    workspaceId: string,
    changeSetId: ChangeSetId,
    kind: string,
    id: Id,
  ): Promise<-1 | T> => {
    // if i haven't cold started yet, dont try and query, temporary
    const key = `${workspaceId}:${changeSetId}`
    if (!coldStarts[key]) return -1;

    console.log("🌈 bifrost query", kind, id);
    const maybeAtomDoc = await db.get(changeSetId, kind, id);
    if (maybeAtomDoc === -1) {
      db.mjolnir(workspaceId, changeSetId, kind, id);
      return -1;
    }
    return maybeAtomDoc;
  };

  // cold start
  const niflheim = async (workspaceId: string, changeSetId: ChangeSetId, force?: boolean) => {
    const key = `${workspaceId}:${changeSetId}`
    if (!coldStarts[key] && force !== true) {
      console.log("❄️ NIFLHEIM ❄️");
      await db.niflheim(workspaceId, changeSetId);
      coldStarts[key] = true;
    }
  };

  const fullDiagnosticTest = async () => {
    await db.fullDiagnosticTest();
  };

  const odin = async () => {
    const allData = await db.odin();
    console.log("⚡ ODIN ⚡");
    console.log(allData);
  };

  return {
    coldStarts,
    odin,
    niflheim,
    bifrost,
    fullDiagnosticTest,
    connectionShouldBeEnabled,
  };
});
