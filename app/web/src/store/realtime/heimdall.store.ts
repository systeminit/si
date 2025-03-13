import { defineStore } from "pinia";
import * as Comlink from "comlink";
import { watch, computed, reactive, Reactive } from "vue";
import { useQueryClient } from "@tanstack/vue-query";
import { DBInterface, Id, BustCacheFn } from "@/workers/types/dbinterface";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { useAuthStore } from "../auth.store";
import { useChangeSetsStore } from "../change_sets.store";
import { useWorkspacesStore } from "../workspaces.store";

export const useHeimdall = defineStore("heimdall", () => {
  const authStore = useAuthStore();

  const coldStarts: Record<string, boolean> = reactive({});

  if (!authStore.selectedOrDefaultAuthToken)
    throw new Error("Missing Auth Token");

  const queryClient = useQueryClient();
  const bustTanStackCache: BustCacheFn = (
    workspaceId: string,
    changeSetId: string,
    kind: string,
    id: string,
  ) => {
    const queryKey = [workspaceId, changeSetId, kind, id];
    console.log("💥 bust cache for", queryKey);
    queryClient.invalidateQueries({ queryKey });
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

  const bifrost = async <T>(args: {
    workspaceId: string;
    changeSetId: ChangeSetId;
    kind: string;
    id: Id;
  }): Promise<Reactive<T> | null> => {
    // if i haven't cold started yet, dont try and query, temporary
    const key = `${args.workspaceId}:${args.changeSetId}`;
    if (!coldStarts[key]) return null;

    console.log("🌈 bifrost query", args.kind, args.id);
    const maybeAtomDoc = await db.get(args.changeSetId, args.kind, args.id);
    if (maybeAtomDoc === -1) {
      db.mjolnir(args.workspaceId, args.changeSetId, args.kind, args.id);
      return null;
    }
    return reactive(maybeAtomDoc);
  };

  // cold start
  const niflheim = async (
    workspaceId: string,
    changeSetId: ChangeSetId,
    force?: boolean,
  ) => {
    const key = `${workspaceId}:${changeSetId}`;
    if (!coldStarts[key] && force !== true) {
      console.log("❄️ NIFLHEIM ❄️");
      await db.niflheim(workspaceId, changeSetId);
      coldStarts[key] = true;
    }
  };

  const changeSetId = computed(() => {
    const changeSetsStore = useChangeSetsStore();
    return changeSetsStore.selectedChangeSetId;
  });
  const workspaceId = computed(() => {
    const workspaceStore = useWorkspacesStore();
    return workspaceStore.selectedWorkspacePk;
  });

  const makeKey = (kind: string, id?: string) => {
    return [
      workspaceId.value!,
      changeSetId.value!,
      kind,
      id ?? changeSetId.value!,
    ];
  };

  const makeArgs = (kind: string, id?: string) => {
    return {
      workspaceId: workspaceId.value!,
      changeSetId: changeSetId.value!,
      kind,
      id: id ?? changeSetId.value!,
    };
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
    changeSetId,
    workspaceId,
    makeKey,
    makeArgs,
    coldStarts,
    odin,
    niflheim,
    bifrost,
    fullDiagnosticTest,
    connectionShouldBeEnabled,
  };
});
