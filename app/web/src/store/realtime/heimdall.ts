import * as Comlink from "comlink";
import { computed, reactive, Reactive } from "vue";
import { QueryClient } from "@tanstack/vue-query";
import { DBInterface, Id, BustCacheFn } from "@/workers/types/dbinterface";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { useChangeSetsStore } from "../change_sets.store";
import { useWorkspacesStore } from "../workspaces.store";

let token: string | undefined;
let queryClient: QueryClient;
export const init = async (bearerToken: string, _queryClient: QueryClient) => {
  if (!token) {
    // eslint-disable-next-line no-console
    console.log("🌈 initializing bifrost...");
    const start = Date.now();
    await db.setBearer(bearerToken);
    await db.initBifrost();
    const end = Date.now();
    token = bearerToken;
    queryClient = _queryClient;
    // eslint-disable-next-line no-console
    console.log(`...initialization completed [${end - start}ms] 🌈`);
  }
};

export const initCompleted = computed(() => !!token);

const bustTanStackCache: BustCacheFn = (
  workspaceId: string,
  changeSetId: string,
  kind: string,
  id: string,
) => {
  const queryKey = [workspaceId, changeSetId, kind, id];
  // eslint-disable-next-line no-console
  console.log("💥 bust cache for", queryKey);
  queryClient.invalidateQueries({ queryKey });

  // TODO: order matters, or code gen has to re-bust sources of references
  if (kind === "View") {
    queryClient.invalidateQueries({
      queryKey: [workspaceId, changeSetId, "ViewList", changeSetId],
    });
    // eslint-disable-next-line no-console
    console.log("💥 bust cache for", [
      workspaceId,
      changeSetId,
      "ViewList",
      changeSetId,
    ]);
  }
};

const workerUrl =
  import.meta.env.VITE_SI_ENV === "local"
    ? "../../workers/webworker.ts"
    : "webworker.js";

const worker = new Worker(new URL(workerUrl, import.meta.url), {
  type: "module",
});
const db: Comlink.Remote<DBInterface> = Comlink.wrap(worker);

// PSA: these are not await'd
// but stuff happens in here we do need to wait for
// figure that out :sweat:
db.addListenerBustCache(Comlink.proxy(bustTanStackCache));

export const bifrostReconnect = async () => {
  await db.bifrostReconnect();
};

export const bifrostClose = async () => {
  await db.bifrostClose();
};

/**
 * PSA, comlink isn't able to serialize a symbol over the wire...
 * So we're using -1 as a replacement for NOROW on this side of the fence...
 */

export const bifrost = async <T>(args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
  kind: string;
  id: Id;
}): Promise<Reactive<T> | null> => {
  if (!initCompleted.value) throw new Error("bifrost not initiated");
  // eslint-disable-next-line no-console
  console.log("🌈 bifrost query", args.kind, args.id);
  const maybeAtomDoc = await db.get(
    args.workspaceId,
    args.changeSetId,
    args.kind,
    args.id,
  );
  if (maybeAtomDoc === -1) {
    db.mjolnir(args.workspaceId, args.changeSetId, args.kind, args.id);
    return null;
  }
  return reactive(maybeAtomDoc);
};

// cold start
export const niflheim = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  force?: boolean,
) => {
  if (!initCompleted.value) return null;
  const coldstart = !(await db.changeSetExists(workspaceId, changeSetId));
  if (coldstart || force) {
    // eslint-disable-next-line no-console
    console.log("❄️ NIFLHEIM ❄️");
    await db.niflheim(workspaceId, changeSetId);
  }
};

export const changeSetId = computed(() => {
  const changeSetsStore = useChangeSetsStore();
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return changeSetsStore.selectedChangeSetId!;
});
const workspaceId = computed(() => {
  const workspaceStore = useWorkspacesStore();
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return workspaceStore.selectedWorkspacePk!;
});

export const makeKey = (kind: string, id?: string) => {
  return [workspaceId.value, changeSetId.value, kind, id ?? changeSetId.value];
};

export const prune = async (workspaceId: string, changeSetId: string) => {
  await db.pruneAtomsForClosedChangeSet(workspaceId, changeSetId);
};

export const makeArgs = (kind: string, id?: string) => {
  return {
    workspaceId: workspaceId.value,
    changeSetId: changeSetId.value,
    kind,
    id: id ?? changeSetId.value,
  };
};

/*
const fullDiagnosticTest = async () => {
  await db.fullDiagnosticTest();
}; */

export const odin = async () => {
  const allData = await db.odin();
  // eslint-disable-next-line no-console
  console.log("⚡ ODIN ⚡");
  // eslint-disable-next-line no-console
  console.log(allData);
};
