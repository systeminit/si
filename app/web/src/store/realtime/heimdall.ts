import * as Comlink from "comlink";
import {
  computed,
  reactive,
  Reactive,
  inject,
  ComputedRef,
  ref,
  watch,
  onScopeDispose,
  getCurrentScope,
  MaybeRefOrGetter,
  toValue,
} from "vue";
import { QueryClient } from "@tanstack/vue-query";
import { monotonicFactory } from "ulid";
import PQueue from "p-queue";
import {
  TabDBInterface,
  SharedDBInterface,
  Id,
  BustCacheFn,
  LobbyExitFn,
  SHARED_BROADCAST_CHANNEL_NAME,
  Listable,
  Gettable,
  AtomDocument,
  UpdateFn,
  QueryAttributesTerm,
} from "@/workers/types/dbinterface";
import {
  Connection,
  EntityKind,
  SchemaMembers,
} from "@/workers/types/entity_kind_types";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { Context } from "@/newhotness/types";
import { DefaultMap } from "@/utils/defaultmap";
import * as rainbow from "@/newhotness/logic_composables/rainbow_counter";
import { sdfApiInstance as sdf } from "@/store/apis.web";
import { WorkspaceMetadata } from "@/api/sdf/dal/workspace";
import { useChangeSetsStore } from "../change_sets.store";
import { useWorkspacesStore } from "../workspaces.store";
import {
  cachedAppEmitter,
  SHOW_CACHED_APP_NOTIFICATION_EVENT,
} from "./cached_app_emitter";

// We want an id right away, not later. But ulid fails if run in this context
// (something about crypto randomValues).  we do not need crypto-secure ulids.
// We just want every tab to have a different one. Which this will get us.
const ulid = monotonicFactory(() => Math.random());

const ranInit = ref<boolean>(false);
let queryClient: QueryClient;
const tabDbId = ulid();
const lockAcquired = ref(false);

const lockAcquiredBroadcastChannel = new BroadcastChannel("DB_LOCK_ACQUIRED");
lockAcquiredBroadcastChannel.onmessage = (message) => {
  if (message.data !== tabDbId) {
    // eslint-disable-next-line no-console
    console.log("🌈 lock acquired by another tab");
    lockAcquired.value = true;
  }
};

const SHARED_WEB_WORKER_URL =
  import.meta.env.VITE_SI_ENV === "local"
    ? "../../workers/shared_webworker.ts"
    : "shared_webworker.js";

// Shared workers are unique per *name*, not per code URL.
const spawnSharedWorker = (name: string) =>
  new SharedWorker(new URL(SHARED_WEB_WORKER_URL, import.meta.url), {
    type: "module",
    name,
  });

let sharedWebWorkerName = `si-db-multiplexer-${__SHARED_WORKER_HASH__}`;
let sharedWorker = spawnSharedWorker(sharedWebWorkerName);
let db: Comlink.Remote<SharedDBInterface> = Comlink.wrap(sharedWorker.port);

const WORKER_URL =
  import.meta.env.VITE_SI_ENV === "local"
    ? "../../workers/webworker.ts"
    : "webworker.js";

const tabWorker = new Worker(new URL(WORKER_URL, import.meta.url), {
  type: "module",
});
const tabDb: Comlink.Remote<TabDBInterface> = Comlink.wrap(tabWorker);

const onSharedWorkerBootBroadcastChannel = new BroadcastChannel(
  SHARED_BROADCAST_CHANNEL_NAME,
);

onSharedWorkerBootBroadcastChannel.onmessage = async (msg) => {
  const name = msg.data as string;
  if (name !== sharedWebWorkerName) {
    // This will ensure that the new shared worker is the one we use to
    // communicate with the various remotes if a new version of the shared
    // webworker code is detected. But, note that if the interface changes, this
    // tab will still have to be reloaded for that communication to work.

    // eslint-disable-next-line no-console
    console.log("🌈 new shared worker detected, reconnecting");
    const currentBearers = await db.getBearers();
    db.unregisterRemote(tabDbId);
    sharedWorker = spawnSharedWorker(name);
    sharedWebWorkerName = name;
    db = Comlink.wrap(sharedWorker.port);
    db.registerRemote(tabDbId, Comlink.proxy(tabDb));
    if (await tabDb.hasDbLock()) {
      await db.setRemote(tabDbId);
    }
    db.addBearers(currentBearers);
    showCachedAppNotification();
  } else {
    db.registerRemote(tabDbId, Comlink.proxy(tabDb));
  }
};

window.onbeforeunload = () => {
  db.unregisterRemote(tabDbId);
};

const showCachedAppNotification = () => {
  cachedAppEmitter.emit(SHOW_CACHED_APP_NOTIFICATION_EVENT);
};

export const init = async (
  workspaceId: string,
  bearerToken: string,
  _queryClient: QueryClient,
) => {
  if (!ranInit.value) {
    // eslint-disable-next-line no-console
    console.log("🌈 calling init...");
    await db.setBearer(workspaceId, bearerToken);

    const { port1, port2 } = new MessageChannel();
    // This message fires when the lock has been acquired for this tab
    port1.onmessage = () => {
      db.setRemote(tabDbId);
      // eslint-disable-next-line no-console
      console.log("🌈 lock acquired by this tab");
      lockAcquired.value = true;
      lockAcquiredBroadcastChannel.postMessage(tabDbId);
    };

    // We are deliberately not awaiting this promise, since it blocks forever on
    // the tabs that do not get the lock
    tabDb.initBifrost(Comlink.proxy(port2));

    ranInit.value = true;
    queryClient = _queryClient;
  }

  // If both tabs are refreshed at the same time, this can falsely indicate that
  // a tab has the lock, but that tab has actually been refreshed just *after*
  // this call, so *we* now have the lock.  adding 2.5 second timeout here
  // ensures that there is enough time for the lock to be resolved in a multitab
  // scenario before we begin cold start. (This only matters if 2+ tabs are
  // refreshed at more or less the same time, in the normal scenario we will
  // indicate lock acquisition via the broadcast channel)
  setTimeout(async () => {
    if ((await db.hasRemote()) && !(await tabDb.hasDbLock())) {
      // eslint-disable-next-line no-console
      console.log("🌈 lock acquired by another tab, detected in timeout");
      lockAcquired.value = true;
    }
  }, 2500);
};

export const initCompleted = computed(
  () => ranInit.value && lockAcquired.value,
);

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

const updateCache = (
  queryKey: string[],
  id: string,
  // there is always more data attached, but we only care about accessing the ID
  // so thats all we need to type!
  // TODO if we're being told to add an undefined value, that seems like an upstream error
  data: { id: string } | undefined,
  removed = false,
) => {
  if (!removed && !data) return;

  queryClient.setQueryData(queryKey, (cachedData: { id: string }[]) => {
    if (!cachedData) {
      return cachedData;
    }

    if (removed) {
      // Filter out the item if it is removed
      return cachedData.filter((d) => d?.id !== id);
    } else {
      // If the data is already in the map, replace the existing entry
      if (cachedData.some((d) => d?.id === id)) {
        return cachedData.map((d) => (d?.id === id ? data : d));
      } else {
        // If the data is not already in the map, add it to the end
        return [...cachedData, data];
      }
    }
  });
};

const atomUpdated: UpdateFn = (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
  data: AtomDocument,
  listIds: string[],
  removed: boolean,
  noBroadcast?: boolean,
) => {
  if (kind === EntityKind.View) {
    const queryKey = [
      workspaceId,
      changeSetId,
      EntityKind.ViewList,
      workspaceId,
    ];
    updateCache(queryKey, id, data, removed);
  } else if (kind === EntityKind.IncomingConnections) {
    const queryKey = [
      workspaceId,
      changeSetId,
      EntityKind.IncomingConnectionsList,
      workspaceId,
    ];
    updateCache(queryKey, id, data, removed);
  } else if (kind === EntityKind.ComponentInList) {
    const queryKey = [
      workspaceId,
      changeSetId,
      EntityKind.ComponentList,
      workspaceId,
    ];
    updateCache(queryKey, id, data, removed);
    if (listIds.length > 0) {
      listIds.forEach((viewId) => {
        const queryKey = [
          workspaceId,
          changeSetId,
          EntityKind.ViewComponentList,
          viewId,
        ];
        updateCache(queryKey, id, data, removed);
      });
    }
  }
  if (!noBroadcast) {
    db.broadcastMessage({
      messageKind: "atomUpdated",
      arguments: { workspaceId, changeSetId, kind, id, data, listIds, removed },
    });
  }
};

const lobbyExit: LobbyExitFn = async (
  workspaceId: string,
  changeSetId: string,
  noBroadcast?: boolean,
) => {
  // Only navigate away from lobby if user is currently in the lobby
  // for this change set
  if (muspelheimStatuses.value[changeSetId] === true) {
    return;
  }

  if (!noBroadcast) {
    db.broadcastMessage({
      messageKind: "lobbyExit",
      arguments: { workspaceId, changeSetId },
    });
  }

  await niflheim(workspaceId, changeSetId, true);
  muspelheimStatuses.value[changeSetId] = true;
};

tabDb.addListenerBustCache(Comlink.proxy(bustTanStackCache));
tabDb.addListenerInFlight(Comlink.proxy(inFlight));
tabDb.addListenerReturned(Comlink.proxy(returned));
tabDb.addListenerLobbyExit(Comlink.proxy(lobbyExit));
tabDb.addAtomUpdated(Comlink.proxy(atomUpdated));

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
  kind: Gettable;
  id: Id;
}): Promise<Reactive<T> | null> => {
  if (!initCompleted.value) throw new Error("You must wait for initialization");

  const start = performance.now();
  const maybeAtomDoc = await db.get(
    args.workspaceId,
    args.changeSetId,
    args.kind,
    args.id,
  );
  const end = performance.now();
  // eslint-disable-next-line no-console
  console.log("🌈 bifrost query", args.kind, args.id, end - start, "ms");
  if (maybeAtomDoc === -1) return null;
  return reactive(maybeAtomDoc);
};

export const bifrostList = async <T>(args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
  kind: Listable;
  id: Id;
}): Promise<Reactive<T> | null> => {
  if (!initCompleted.value) throw new Error("You must wait for initialization");

  const start = performance.now();
  const maybeAtomDoc = await db.getList(
    args.workspaceId,
    args.changeSetId,
    args.kind,
    args.id,
  );
  const end = performance.now();
  // eslint-disable-next-line no-console
  console.log("🌈 bifrost queryList", args.kind, args.id, end - start, "ms");
  if (!maybeAtomDoc) return null;
  return reactive(JSON.parse(maybeAtomDoc));
};

/**
 * Query AttributeTree MVs in a changeset, looking for components that match the given terms.
 *
 * @param args.workspaceId The workspace ID to query.
 * @param args.changeSetId The changeset ID to query.
 * @param args.terms The key/value pairs to match. e.g. { key: "vpcId", value: "vpc-123" } or { key: "/domain/vpcId", value: "vpc-123" }
 * @returns the list of component IDs that match the given terms.
 */
export const bifrostQueryAttributes = async (args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
  terms: QueryAttributesTerm[];
}) => {
  if (!initCompleted.value) throw new Error("You must wait for initialization");

  const start = performance.now();
  const components = await db.queryAttributes(
    args.workspaceId,
    args.changeSetId,
    args.terms,
  );
  const end = performance.now();
  // eslint-disable-next-line no-console
  console.log("🌈 bifrost queryAttributes", end - start, "ms");
  return reactive(components);
};

export const getPossibleConnections = async (args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
}) => {
  return await db.getPossibleConnections(args.workspaceId, args.changeSetId);
};

export const linkNewChangeset = async (
  workspaceId: string,
  changeSetId: string,
  headChangeSetId: string,
) => {
  await db.linkNewChangeset(workspaceId, headChangeSetId, changeSetId);
};

export const getOutgoingConnectionsCounts = async (args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
}) => {
  if (!initCompleted.value) throw new Error("You must wait for initialization");

  const start = performance.now();
  const connectionsCounts = await db.getOutgoingConnectionsCounts(
    args.workspaceId,
    args.changeSetId,
  );
  const end = performance.now();
  // eslint-disable-next-line no-console
  console.log(
    "🌈 bifrost query getOutgoingConnectionsCounts",
    end - start,
    "ms",
  );
  if (connectionsCounts) return reactive(connectionsCounts);
  else return {};
};

export const getComponentDetails = async (args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
}) => {
  if (!initCompleted.value) throw new Error("You must wait for initialization");

  const start = performance.now();
  const componentNames = await db.getComponentDetails(
    args.workspaceId,
    args.changeSetId,
  );
  const end = performance.now();
  // eslint-disable-next-line no-console
  console.log("🌈 bifrost query componentNames", end - start, "ms");
  if (componentNames) return reactive(componentNames);
  else return {};
};

export const getSchemaMembers = async (args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
}): Promise<SchemaMembers[]> => {
  if (!initCompleted.value) throw new Error("You must wait for initialization");

  const start = performance.now();
  const schemaMembers = await db.getSchemaMembers(
    args.workspaceId,
    args.changeSetId,
  );
  const end = performance.now();
  // eslint-disable-next-line no-console
  console.log("🌈 bifrost query getSchemaMembers", end - start, "ms");
  if (schemaMembers) return reactive(JSON.parse(schemaMembers));
  else return [];
};

export const getOutgoingConnections = async (args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
}) => {
  if (!initCompleted.value) throw new Error("You must wait for initialization");

  const connectionsById = await db.getOutgoingConnectionsByComponentId(
    args.workspaceId,
    args.changeSetId,
  );
  if (connectionsById) return reactive(connectionsById);
  return new DefaultMap<string, Record<string, Connection>>(() => ({}));
};

export const getIncomingManagement = async (args: {
  workspaceId: string;
  changeSetId: ChangeSetId;
}) => {
  if (!initCompleted.value) throw new Error("You must wait for initialization");

  const connectionsById = await db.getIncomingManagementByComponentId(
    args.workspaceId,
    args.changeSetId,
  );
  if (connectionsById) return reactive(connectionsById);
  return new DefaultMap<string, Record<string, Connection>>(() => ({}));
};

const waitForInitCompletion = (): Promise<void> => {
  return new Promise((resolve) => {
    if (initCompleted.value) {
      // eslint-disable-next-line no-console
      console.debug("init already completed");
      resolve();
      return;
    }

    // eslint-disable-next-line no-console
    console.debug("waiting for init completion");
    const unwatch = watch(initCompleted, (newValue) => {
      if (newValue) {
        // eslint-disable-next-line no-console
        console.debug("init completed in watcher");
        unwatch();
        resolve();
      }
    });
    // If this happens in a disposable scope, we want to warn if the scope gets cancelled
    // (because the watch will be cancelled as well)
    if (getCurrentScope()) {
      onScopeDispose(() => {
        if (!initCompleted.value) {
          // eslint-disable-next-line no-console
          console.warn("waiting for init cancelled");
        }
      });
    }
  });
};

const MUSPELHEIM_CONCURRENCY = 1;

export const muspelheimStatuses = ref<{ [key: string]: boolean }>({});

const fetchOpenChangeSets = async (
  workspaceId: string,
): Promise<WorkspaceMetadata> => {
  const resp = await sdf<WorkspaceMetadata>({
    method: "GET",
    url: `v2/workspaces/${workspaceId}/change-sets`,
  });
  return resp.data;
};

export const muspelheim = async (workspaceId: string, force?: boolean) => {
  await waitForInitCompletion();
  const start = performance.now();
  // eslint-disable-next-line no-console
  console.log("🔥 MUSPELHEIM 🔥");
  const niflheimQueue = new PQueue({ concurrency: MUSPELHEIM_CONCURRENCY });
  const { changeSets: openChangeSets, defaultChangeSetId: baseChangeSetId } =
    await fetchOpenChangeSets(workspaceId);
  if (!baseChangeSetId) {
    throw new Error("No HEAD changeset found");
  }

  // Mark as pending in advance
  for (const changeSet of openChangeSets) {
    muspelheimStatuses.value[changeSet.id] = false;
  }

  await niflheim(workspaceId, baseChangeSetId, force);

  for (const changeSet of openChangeSets) {
    if (changeSet.id === baseChangeSetId) {
      continue;
    }

    niflheimQueue.add(async () => {
      await niflheim(workspaceId, changeSet.id, force);
    });
  }

  await niflheimQueue.onEmpty();

  // eslint-disable-next-line no-console
  console.log("🔥 DONE 🔥", performance.now() - start);
  return true;
};

export const registerBearerToken = async (
  workspaceId: string,
  bearerToken: string,
) => {
  await db.setBearer(workspaceId, bearerToken);
};

// cold start
export const niflheim = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  force = false,
  lobbyOnFailure = true,
): Promise<boolean> => {
  await waitForInitCompletion();
  const start = performance.now();
  const changeSetExists = await db.changeSetExists(workspaceId, changeSetId);
  if (!changeSetExists || force) {
    // eslint-disable-next-line no-console
    console.log("❄️ NIFLHEIM ❄️", changeSetId);
    const success = await db.niflheim(workspaceId, changeSetId);
    // eslint-disable-next-line no-console
    console.log("❄️ DONE ❄️", performance.now() - start);

    // If niflheim returned false (202 response), navigate to lobby
    // Index is being rebuilt and is not ready yet.
    if (!success && lobbyOnFailure) {
      muspelheimStatuses.value[changeSetId] = false;
    } else if (success) {
      muspelheimStatuses.value[changeSetId] = true;
    }
    return success;
  }

  return true;
};

// deprecated
export const changeSetId = computed(() => {
  const changeSetsStore = useChangeSetsStore();
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return changeSetsStore.selectedChangeSetId!;
});
// deprecated
const workspaceId = computed(() => {
  const workspaceStore = useWorkspacesStore();
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return workspaceStore.selectedWorkspacePk!;
});

// deprecated
// this is for the old world!
export const makeKey = (kind: string, id?: string) => {
  return [workspaceId.value, changeSetId.value, kind, id ?? workspaceId.value];
};

export const prune = async (workspaceId: string, changeSetId: string) => {
  delete muspelheimStatuses.value[changeSetId];
  await db.pruneAtomsForClosedChangeSet(workspaceId, changeSetId);
};

// this is for the old world!
export const makeArgs = (kind: string, id?: string) => {
  return {
    workspaceId: workspaceId.value,
    changeSetId: changeSetId.value,
    kind: kind as Gettable,
    id: id ?? changeSetId.value,
  };
};

export const useMakeArgs = () => {
  const ctx: Context | undefined = inject("CONTEXT");

  return <K = Gettable>(kind: EntityKind, id?: string) => {
    return {
      workspaceId: ctx?.workspacePk.value ?? "",
      changeSetId: ctx?.changeSetId.value ?? "",
      kind: kind as K,
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
export const useMakeKey = () => {
  const ctx: Context | undefined = inject("CONTEXT");

  return <K = Gettable>(
    kind: MaybeRefOrGetter<K>,
    id?: MaybeRefOrGetter<string>,
  ) =>
    computed<[string?, string?, (ComputedRef<K> | K)?, string?]>(() => [
      ctx?.workspacePk.value,
      ctx?.changeSetId.value,
      toValue(kind),
      toValue(id ?? ctx?.workspacePk),
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
