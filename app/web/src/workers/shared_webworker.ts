import * as Comlink from "comlink";
import {
  ExecBaseOptions,
  ExecReturnResultRowsOptions,
  ExecRowModeArrayOptions,
  FlexibleString,
  SqlValue,
} from "@sqlite.org/sqlite-wasm";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import { WorkspacePk } from "@/api/sdf/dal/workspace";
import { EntityKind } from "./types/entity_kind_types";
import {
  SharedDBInterface,
  TabDBInterface,
  NOROW,
  AtomDocument,
  BroadcastMessage,
  SHARED_BROADCAST_CHANNEL_NAME,
  DB_NOT_INIT_ERR,
  Gettable,
  Listable,
  QueryAttributesTerm,
} from "./types/dbinterface";

// Wait 5 seconds after we no longer have any remotes before terminating ourselves
const SHUTDOWN_DELAY_MS = 5000;

declare global {
  interface Window {
    onconnect?: (event: MessageEvent) => void;
  }
}

const _DEBUG = true; // import.meta.env.VITE_SI_ENV === "local";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function debug(...args: any | any[]) {
  // eslint-disable-next-line no-console
  if (_DEBUG) console.debug(args);
}

let currentLeader: Comlink.Remote<TabDBInterface> | undefined;
let currentLeaderId: string | undefined;
const remotes: { [key: string]: Comlink.Remote<TabDBInterface> } = {};
const bearerTokens: { [key: string]: string } = {};

const hasLeaderChannel = new MessageChannel();
const leaderChangedChannel = new MessageChannel();

const LEADER_CHANGED = "LEADER_CHANGED";

const failOnLeaderChange = (): Promise<never> => {
  return new Promise((_, reject) => {
    const onMessage = () => {
      reject(LEADER_CHANGED);
    };
    leaderChangedChannel.port1.addEventListener("message", onMessage, {
      capture: false,
      passive: true,
      once: true,
    });
    leaderChangedChannel.port1.start();
  });
};

const getLeader = (): Promise<Comlink.Remote<TabDBInterface>> => {
  return new Promise((resolve, reject) => {
    if (currentLeader) {
      resolve(currentLeader);
    }
    hasLeaderChannel.port1.onmessage = () => {
      if (currentLeader) {
        resolve(currentLeader);
      } else {
        reject(new Error("Got remote message but no remote set"));
      }
    };
  });
};

const MAX_RETRIES = 250;

const sleep = (ms: number) =>
  new Promise((resolve) => {
    setTimeout(resolve, ms);
  });

async function withLeader<R>(
  cb: (remote: Comlink.Remote<TabDBInterface>) => Promise<R>,
  retry?: number,
): Promise<R> {
  const remote = await getLeader();
  const retries = retry ?? 0;

  if (retries >= MAX_RETRIES) {
    throw new Error(
      "Retries exceeded attempting to perform query against database leader. Please refresh this tab.",
    );
  }

  // If the leader with the remote changes while a call is in progress, we need
  // to retry the call, now with the new remote. Otherwise, we will likely hang
  // forever.
  try {
    const result = await Promise.race([cb(remote), failOnLeaderChange()]);
    return result;
  } catch (err) {
    if (typeof err === "string" && err === LEADER_CHANGED) {
      debug("LEADER CHANGED MID REQUEST, rerunning callback", retries);
      return withLeader(cb, retries + 1);
    }
    if (err instanceof Error && err.message === DB_NOT_INIT_ERR) {
      debug("DB NOT INITIALIZED?", retries);
      await sleep(100);
      return withLeader(cb, retries + 1);
    }

    throw err;
  }
}

const dbInterface: SharedDBInterface = {
  async broadcastMessage(message: BroadcastMessage) {
    Object.keys(remotes).forEach((remoteId) => {
      if (remoteId !== currentLeaderId) {
        try {
          remotes[remoteId]?.receiveBroadcast(message);
        } catch (err) {
          debug("failed to send to remote", remoteId, err);
        }
      }
    });
  },

  unregisterRemote(id: string) {
    debug("unregister remote in shared", id);
    if (currentLeaderId === id) {
      debug("tab with lock unregistered. no remote set");
      currentLeader = undefined;
    }
    delete remotes[id];

    if (Object.keys(remotes).length === 0) {
      // Just in case there is a race between closing a tab and a new one
      // loading, and this worker gets a new remote, don't shut down right away.
      // Double check after a few seconds.
      setTimeout(async () => {
        if (Object.keys(remotes).length === 0) {
          shutDownWebWorker();
        }
      }, SHUTDOWN_DELAY_MS);
    }
  },
  async registerRemote(id: string, remote: Comlink.Remote<TabDBInterface>) {
    if (!remotes[id]) {
      debug("register remote in shared", id);
      remotes[id] = remote;
    }
    if (await remote.hasDbLock()) {
      await this.setLeader(id);
    }
  },

  async hasLeader() {
    return !!currentLeader;
  },

  async currentLeaderId() {
    return currentLeaderId;
  },

  async setLeader(remoteId: string) {
    debug("setting remote in shared web worker to", remoteId);

    currentLeader = remotes[remoteId];
    if (!currentLeader) {
      throw new Error(`remote ${remoteId} not registered`);
    }
    const leaderChanged = currentLeaderId !== remoteId;
    currentLeaderId = remoteId;

    for (const [workspaceId, workspaceToken] of Object.entries(bearerTokens)) {
      await currentLeader.setBearer(workspaceId, workspaceToken);
      await currentLeader.initSocket(workspaceId);
    }

    currentLeader.bifrostReconnect();

    hasLeaderChannel.port2.postMessage("got leader");
    if (leaderChanged) {
      debug("follow the leader");
      leaderChangedChannel.port2.postMessage("leader changed");
    }
  },

  async initDB(_testing: boolean) {
    debug("init db called in shared webworker");
  },

  async migrate(testing: boolean) {
    return withLeader(async (remote) => await remote.migrate(testing));
  },

  async setBearer(workspaceId, token): Promise<void> {
    bearerTokens[workspaceId] = token;
    const updateRemote = async () => {
      await currentLeader?.setBearer(workspaceId, token);
      currentLeader?.initSocket(workspaceId);
    };
    updateRemote();
  },

  async getBearers(): Promise<{ [key: string]: string }> {
    return bearerTokens;
  },

  async addBearers(bearers) {
    for (const [workspaceId, bearerToken] of Object.entries(bearers)) {
      bearerTokens[workspaceId] = bearerToken;
      await currentLeader?.setBearer(workspaceId, bearerToken);
      currentLeader?.initSocket(workspaceId);
    }
  },

  async initSocket(workspaceId: string): Promise<void> {
    await withLeader(async (remote) => await remote.initSocket(workspaceId));
  },

  async initBifrost(_gotlockPort: MessagePort) {
    debug("init bifrost in shared worker called");
  },

  async bifrostClose() {
    await withLeader(async (remote) => await remote.bifrostClose());
  },

  async bifrostReconnect() {
    await withLeader(async (remote) => await remote.bifrostReconnect());
  },

  async linkNewChangeset(
    workspaceId: string,
    headChangeSetId: string,
    changeSetId: string,
  ): Promise<void> {
    await withLeader(
      async (remote) =>
        await remote.linkNewChangeset(
          workspaceId,
          headChangeSetId,
          changeSetId,
        ),
    );
  },

  async getOutgoingConnectionsByComponentId(
    workspaceId: string,
    changeSetId: string,
  ) {
    return await withLeader(
      async (remote) =>
        await remote.getOutgoingConnectionsByComponentId(
          workspaceId,
          changeSetId,
        ),
    );
  },

  async getIncomingManagementByComponentId(
    workspaceId: string,
    changeSetId: string,
  ) {
    return await withLeader(
      async (remote) =>
        await remote.getIncomingManagementByComponentId(
          workspaceId,
          changeSetId,
        ),
    );
  },

  async get(
    workspaceId: string,
    changeSetId: string,
    kind: Gettable,
    id: string,
  ): Promise<typeof NOROW | AtomDocument> {
    return await withLeader(
      async (remote) => await remote.get(workspaceId, changeSetId, kind, id),
    );
  },

  async getList(
    workspaceId: string,
    changeSetId: string,
    kind: Listable,
    id: string,
  ): Promise<string> {
    return await withLeader(
      async (remote) =>
        await remote.getList(workspaceId, changeSetId, kind, id),
    );
  },

  async queryAttributes(
    workspaceId: WorkspacePk,
    changeSetId: ChangeSetId,
    terms: QueryAttributesTerm[],
  ): Promise<ComponentId[]> {
    return await withLeader(
      async (remote) =>
        await remote.queryAttributes(workspaceId, changeSetId, terms),
    );
  },

  async getPossibleConnections(workspaceId, changeSetId) {
    return await withLeader(
      async (remote) =>
        await remote.getPossibleConnections(workspaceId, changeSetId),
    );
  },

  async getOutgoingConnectionsCounts(workspaceId: string, changeSetId: string) {
    return await withLeader(
      async (remote) =>
        await remote.getOutgoingConnectionsCounts(workspaceId, changeSetId),
    );
  },

  async getComponentDetails(workspaceId: string, changeSetId: string) {
    return await withLeader(
      async (remote) =>
        await remote.getComponentDetails(workspaceId, changeSetId),
    );
  },

  async getComponentsInViews(workspaceId: string, changeSetId: string) {
    return await withLeader(
      async (remote) =>
        await remote.getComponentsInViews(workspaceId, changeSetId),
    );
  },

  async getComponentsInOnlyOneView(workspaceId: string, changeSetId: string) {
    return await withLeader(
      async (remote) =>
        await remote.getComponentsInOnlyOneView(workspaceId, changeSetId),
    );
  },

  async getSchemaMembers(workspaceId: string, changeSetId: string) {
    return await withLeader(
      async (remote) => await remote.getSchemaMembers(workspaceId, changeSetId),
    );
  },

  async getDefaultSubscriptions(workspaceId: string, changeSetId: string) {
    return await withLeader(
      async (remote) =>
        await remote.getDefaultSubscriptions(workspaceId, changeSetId),
    );
  },

  async mjolnir(
    workspaceId: string,
    changeSetId: string,
    kind: EntityKind,
    id: string,
    checksum?: string,
  ) {
    await withLeader(
      async (remote) =>
        await remote.mjolnir(workspaceId, changeSetId, kind, id, checksum),
    );
  },

  async changeSetExists(
    workspaceId: string,
    changeSetId: string,
  ): Promise<boolean> {
    return await withLeader(
      async (remote) => await remote.changeSetExists(workspaceId, changeSetId),
    );
  },

  async niflheim(workspaceId: string, changeSetId: string): Promise<boolean> {
    return await withLeader(
      async (remote) => await remote.niflheim(workspaceId, changeSetId),
    );
  },

  async pruneAtomsForClosedChangeSet(workspaceId: string, changeSetId: string) {
    return await withLeader(
      async (remote) =>
        await remote.pruneAtomsForClosedChangeSet(workspaceId, changeSetId),
    );
  },

  async exec(
    opts: ExecBaseOptions &
      ExecRowModeArrayOptions &
      ExecReturnResultRowsOptions & {
        sql: FlexibleString;
      },
  ): Promise<SqlValue[][]> {
    return await withLeader(async (remote) => await remote.exec(opts));
  },

  async bobby(): Promise<void> {
    await withLeader(async (remote) => await remote.bobby());
  },

  async ragnarok(
    workspaceId: string,
    changeSetId: string,
    noColdStart?: boolean,
  ): Promise<void> {
    await withLeader(
      async (remote) =>
        await remote.ragnarok(workspaceId, changeSetId, noColdStart),
    );
  },

  async odin(changeSetId: string): Promise<object> {
    return await withLeader(async (remote) => await remote.odin(changeSetId));
  },
};

const onConnectBroadcast = new BroadcastChannel(SHARED_BROADCAST_CHANNEL_NAME);

// eslint-disable-next-line no-restricted-globals
const name = self.name;

// eslint-disable-next-line no-restricted-globals
const shutDownWebWorker = () => self.close();

// eslint-disable-next-line no-restricted-globals
self.onconnect = (event: MessageEvent) => {
  Comlink.expose(dbInterface, event.ports[0]);
  onConnectBroadcast.postMessage(name);
};
