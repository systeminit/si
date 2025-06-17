import * as Comlink from "comlink";
import {
  computed,
  reactive,
  Reactive,
  inject,
  ComputedRef,
  unref,
  toRaw,
  ref,
} from "vue";
import { QueryClient } from "@tanstack/vue-query";
import { monotonicFactory } from "ulid";
import {
  DBInterface,
  Id,
  BustCacheFn,
  LobbyExitFn,
  SHARED_BROADCAST_CHANNEL_NAME,
} from "@/workers/types/dbinterface";
import {
  BifrostConnection,
  EntityKind,
  Prop,
} from "@/workers/types/entity_kind_types";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { Context } from "@/newhotness/types";
import { DefaultMap } from "@/utils/defaultmap";
import * as rainbow from "@/newhotness/logic_composables/rainbow_counter";
import router from "@/router";
import { useChangeSetsStore } from "../change_sets.store";
import { useWorkspacesStore } from "../workspaces.store";

// We want an id right away, not later. But ulid fails if run in this context (something about crypto randomValues).
// we do not need crypto-secure ulids. We just want every tab to have a different one. Which this will get us.
const ulid = monotonicFactory(() => Math.random());

let token: string | undefined;
let queryClient: QueryClient;
const tabDbId = ulid();
const lockAcquired = ref(false);

export const init = async (bearerToken: string, _queryClient: QueryClient) => {
  if (!token) {
    // eslint-disable-next-line no-console
    console.log("🌈 initializing bifrost...");
    const start = Date.now();
    await tabDb.setBearer(bearerToken);

    const { port1, port2 } = new MessageChannel();
    // This message fires when the lock has been acquired for this tab
    port1.onmessage = () => {
      db.setRemote(tabDbId);
      lockAcquired.value = true;
    };

    // We are deliberately not awaiting this promise, since it blocks forever on the tabs that do not get the lock
    tabDb.initBifrost(Comlink.proxy(port2));

    const end = Date.now();
    token = bearerToken;
    queryClient = _queryClient;
    // eslint-disable-next-line no-console
    console.log(`...initialization completed [${end - start}ms] 🌈`);
  }
};

export const initCompleted = computed(() => !!token && lockAcquired.value);

const bustTanStackCache: BustCacheFn = (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
  noBroadcast?: boolean,
) => {
  const queryKey = [workspaceId, changeSetId, kind, id];
  // eslint-disable-next-line no-console
  console.log("💥 bust tanstack cache for", queryKey);
  queryClient.invalidateQueries({ queryKey });
  if (!noBroadcast) {
    db.broadcastMessage({
      messageKind: "cacheBust",
      arguments: { workspaceId, changeSetId, kind, id },
    });
  }
};

const sharedWebWorkerUrl =
  import.meta.env.VITE_SI_ENV === "local"
    ? "../../workers/shared_webworker.ts"
    : "shared_webworker.js";

const sharedWorker = new SharedWorker(
  new URL(sharedWebWorkerUrl, import.meta.url),
  { type: "module", name: "si-db-multiplexer" },
);

const db: Comlink.Remote<DBInterface> = Comlink.wrap(sharedWorker.port);

const workerUrl =
  import.meta.env.VITE_SI_ENV === "local"
    ? "../../workers/webworker.ts"
    : "webworker.js";

const tabWorker = new Worker(new URL(workerUrl, import.meta.url), {
  type: "module",
});
const tabDb: Comlink.Remote<DBInterface> = Comlink.wrap(tabWorker);

const onSharedWorkerBootBroadcastChannel = new BroadcastChannel(
  SHARED_BROADCAST_CHANNEL_NAME,
);
onSharedWorkerBootBroadcastChannel.onmessage = () => {
  db.registerRemote(tabDbId, Comlink.proxy(tabDb));
};
window.onbeforeunload = () => {
  db.unregisterRemote(tabDbId);
};

const inFlight = (
  changeSetId: ChangeSetId,
  label: string,
  noBroadcast?: boolean,
) => {
  rainbow.add(changeSetId, label);
  if (!noBroadcast) {
    db.broadcastMessage({
      messageKind: "listenerInFlight",
      arguments: { changeSetId, label },
    });
  }
};

const returned = (
  changeSetId: ChangeSetId,
  label: string,
  noBroadcast?: boolean,
) => {
  rainbow.remove(changeSetId, label);

  if (!noBroadcast) {
    db.broadcastMessage({
      messageKind: "listenerReturned",
      arguments: { changeSetId, label },
    });
  }
};

const lobbyExit: LobbyExitFn = async (
  workspaceId: string,
  changeSetId: string,
  noBroadcast?: boolean,
) => {
  // Only navigate away from lobby if user is currently in the lobby
  // for this workspace and change set
  if (router.currentRoute.value.name !== "new-hotness-lobby") {
    return;
  } else {
    const params = router.currentRoute.value.params;
    if (!params || Object.keys(params).length === 0)
      throw new Error("Params expected");
    if (
      params.workspaceId !== workspaceId ||
      params.changeSetId !== changeSetId
    )
      return;
  }

  if (!noBroadcast) {
    db.broadcastMessage({
      messageKind: "lobbyExit",
      arguments: { workspaceId, changeSetId },
    });
  }

  await niflheim(workspaceId, changeSetId, true);
  router.push({
    name: "new-hotness",
    params: {
      workspacePk: workspaceId,
      changeSetId,
    },
  });
};

tabDb.addListenerBustCache(Comlink.proxy(bustTanStackCache));
tabDb.addListenerInFlight(Comlink.proxy(inFlight));
tabDb.addListenerReturned(Comlink.proxy(returned));
tabDb.addListenerLobbyExit(Comlink.proxy(lobbyExit));

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
  kind: EntityKind;
  id: Id;
}): Promise<Reactive<T> | null> => {
  if (!initCompleted.value) throw new Error("bifrost not initiated");
  const start = Date.now();
  const maybeAtomDoc = await db.get(
    args.workspaceId,
    args.changeSetId,
    args.kind,
    args.id,
  );
  const end = Date.now();
  // eslint-disable-next-line no-console
  console.log("🌈 bifrost query", args.kind, args.id, end - start, "ms");
  if (maybeAtomDoc === -1) return null;
  return reactive(maybeAtomDoc);
};

export const getPossibleConnections = async (args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
  destSchemaName: string;
  dest: Prop;
}) => {
  return reactive(
    await db.getPossibleConnections(
      args.workspaceId,
      args.changeSetId,
      args.destSchemaName,
      toRaw(args.dest), // Can't send reactive stuff across the boundary, silently fails
    ),
  );
};

export const linkNewChangeset = async (
  workspaceId: string,
  changeSetId: string,
  headChangeSetId: string,
) => {
  await db.linkNewChangeset(workspaceId, headChangeSetId, changeSetId);
};

export const getOutgoingConnections = async (args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
}) => {
  if (!initCompleted.value) throw new Error("bifrost not initiated");

  const connectionsById = await db.getOutgoingConnectionsByComponentId(
    args.workspaceId,
    args.changeSetId,
  );
  if (connectionsById) return reactive(connectionsById);
  return new DefaultMap<string, Record<string, BifrostConnection>>(() => ({}));
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
    const success = await db.niflheim(workspaceId, changeSetId);
    // eslint-disable-next-line no-console
    console.log("❄️ DONE ❄️");

    // If niflheim returned false (202 response), navigate to lobby
    // Index is being rebuilt and is not ready yet.
    if (!success) {
      router.push({
        name: "new-hotness-lobby",
        params: {
          workspacePk: workspaceId,
          changeSetId,
        },
      });
    } else return true;
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

// this is for the old world!
export const makeKey = (kind: string, id?: string) => {
  return [
    workspaceId.value,
    changeSetId.value,
    kind as EntityKind,
    id ?? workspaceId.value,
  ];
};

export const prune = async (workspaceId: string, changeSetId: string) => {
  await db.pruneAtomsForClosedChangeSet(workspaceId, changeSetId);
};

// this is for the old world!
export const makeArgs = (kind: string, id?: string) => {
  return {
    workspaceId: workspaceId.value,
    changeSetId: changeSetId.value,
    kind: kind as EntityKind,
    id: id ?? changeSetId.value,
  };
};

export const useMakeArgs = () => {
  const ctx: Context | undefined = inject("CONTEXT");

  return (kind: EntityKind, id?: string) => {
    return {
      workspaceId: ctx?.workspacePk.value ?? "",
      changeSetId: ctx?.changeSetId.value ?? "",
      kind,
      id: id ?? ctx?.workspacePk.value ?? "",
    };
  };
};

export const changeSetExists = async (
  workspaceId: string,
  changeSetId: string,
) => await db.changeSetExists(workspaceId, changeSetId);

/// Make a reactive query key that includes the workspace, changeSet, EntityKind and entity ID
/// (if any).
///
/// @returns A computed reactive key suitable for use with tanstack useQuery() or useQueryClient().
///
/// @example
/// const componentId = ref<ComponentId>();
/// const makeKey = useMakeKey();
/// const query = useQuery({ queryKey: makeKey(EntityKind.Component, componentId), ... });
///
/// You may also specify other reactive values that will be included in the key, so that the query
/// will restart when those other values change:
///
/// @example
/// const currentProp = ref<Prop>();
/// const makeKey = useMakeKey();
/// const query = useQuery({ queryKey: makeKey(EntityKind.PossibleConnections, undefined, currentProp), ... });
///
export const useMakeKey = () => {
  const ctx: Context | undefined = inject("CONTEXT");

  return <T extends unknown[]>(
    kind: ComputedRef<EntityKind> | EntityKind,
    id?: ComputedRef<string> | string,
    ...extra: [...T]
  ) =>
    computed<
      [string?, string?, (ComputedRef<EntityKind> | EntityKind)?, string?, ...T]
    >(() => [
      ctx?.workspacePk.value,
      ctx?.changeSetId.value,
      kind,
      unref(id) ?? ctx?.workspacePk.value,
      ...extra,
    ]);
};

export const odin = async (changeSetId: string) => {
  const allData = await db.odin(changeSetId);
  // eslint-disable-next-line no-console
  console.log("⚡ ODIN ⚡");
  // eslint-disable-next-line no-console
  console.log(allData);
};

export const bobby = async () => {
  await db.bobby();
  // eslint-disable-next-line no-console
  console.log("🗑️ BOBBY DROP TABLE 🗑️");
};
export const ragnarok = async (workspaceId: string, changeSetId: string) => {
  await db.ragnarok(workspaceId, changeSetId);
  // eslint-disable-next-line no-console
  console.log("🗑️ RAGNAROK 🗑️");
};

export const mjolnir = async (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
) => {
  await db.mjolnir(workspaceId, changeSetId, kind, id);
};
