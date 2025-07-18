import * as Comlink from "comlink";
import {
  ExecBaseOptions,
  ExecReturnResultRowsOptions,
  ExecRowModeArrayOptions,
  FlexibleString,
  SqlValue,
} from "@sqlite.org/sqlite-wasm";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { WorkspacePk } from "@/store/workspaces.store";
import { ComponentId } from "@/api/sdf/dal/component";
import { EntityKind } from "./types/entity_kind_types";
import {
  SharedDBInterface,
  TabDBInterface,
  NOROW,
  AtomDocument,
  BroadcastMessage,
  SHARED_BROADCAST_CHANNEL_NAME,
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

let currentRemote: Comlink.Remote<TabDBInterface> | undefined;
let currentRemoteId: string | undefined;
const remotes: { [key: string]: Comlink.Remote<TabDBInterface> } = {};
const bearerTokens: { [key: string]: string } = {};

const hasRemoteChannel = new MessageChannel();

const getRemote = (): Promise<Comlink.Remote<TabDBInterface>> => {
  return new Promise((resolve, reject) => {
    if (currentRemote) {
      resolve(currentRemote);
    }
    hasRemoteChannel.port1.onmessage = () => {
      if (currentRemote) {
        resolve(currentRemote);
      } else {
        reject(new Error("Got remote message but no remote set"));
      }
    };
  });
};

async function withRemote<R>(
  cb: (remote: Comlink.Remote<TabDBInterface>) => Promise<R>,
): Promise<R> {
  const remote = await getRemote();
  return cb(remote);
}

const dbInterface: SharedDBInterface = {
  async broadcastMessage(message: BroadcastMessage) {
    Object.keys(remotes).forEach((remoteId) => {
      if (remoteId !== currentRemoteId) {
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
    if (currentRemoteId === id) {
      debug("tab with lock unregistered. no remote set");
      currentRemote = undefined;
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
      await this.setRemote(id);
    }
  },
  async hasRemote() {
    return !!currentRemote;
  },
  async currentRemoteId() {
    return currentRemoteId;
  },
  async setRemote(remoteId: string) {
    debug("setting remote in shared web worker to", remoteId);

    currentRemote = remotes[remoteId];
    if (!currentRemote) {
      throw new Error(`remote ${remoteId} not registered`);
    }
    currentRemoteId = remoteId;

    for (const [workspaceId, workspaceToken] of Object.entries(bearerTokens)) {
      await currentRemote.setBearer(workspaceId, workspaceToken);
      await currentRemote.initSocket(workspaceId);
    }

    currentRemote.bifrostReconnect();

    hasRemoteChannel.port2.postMessage("got remote");
  },

  async initDB(_testing: boolean) {
    debug("init db called in shared webworker");
  },

  async migrate(testing: boolean) {
    return withRemote(async (remote) => await remote.migrate(testing));
  },

  async setBearer(workspaceId, token): Promise<void> {
    bearerTokens[workspaceId] = token;
    const updateRemote = async () => {
      await currentRemote?.setBearer(workspaceId, token);
      currentRemote?.initSocket(workspaceId);
    };
    updateRemote();
  },

  async getBearers(): Promise<{ [key: string]: string }> {
    return bearerTokens;
  },

  async addBearers(bearers) {
    for (const [workspaceId, bearerToken] of Object.entries(bearers)) {
      bearerTokens[workspaceId] = bearerToken;
      await currentRemote?.setBearer(workspaceId, bearerToken);
      currentRemote?.initSocket(workspaceId);
    }
  },

  async initSocket(workspaceId: string): Promise<void> {
    await withRemote(async (remote) => await remote.initSocket(workspaceId));
  },

  async initBifrost(_gotlockPort: MessagePort) {
    debug("init bifrost in shared worker called");
  },

  async bifrostClose() {
    await withRemote(async (remote) => await remote.bifrostClose());
  },

  async bifrostReconnect() {
    await withRemote(async (remote) => await remote.bifrostReconnect());
  },

  async linkNewChangeset(
    workspaceId: string,
    headChangeSetId: string,
    changeSetId: string,
  ): Promise<void> {
    await withRemote(
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
    return await withRemote(
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
    return await withRemote(
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
    return await withRemote(
      async (remote) => await remote.get(workspaceId, changeSetId, kind, id),
    );
  },

  async getList(
    workspaceId: string,
    changeSetId: string,
    kind: Listable,
    id: string,
  ): Promise<string> {
    return await withRemote(
      async (remote) =>
        await remote.getList(workspaceId, changeSetId, kind, id),
    );
  },

  async queryAttributes(
    workspaceId: WorkspacePk,
    changeSetId: ChangeSetId,
    terms: QueryAttributesTerm[],
  ): Promise<ComponentId[]> {
    return await withRemote(
      async (remote) =>
        await remote.queryAttributes(workspaceId, changeSetId, terms),
    );
  },

  async getPossibleConnections(workspaceId, changeSetId) {
    return await withRemote(
      async (remote) =>
        await remote.getPossibleConnections(workspaceId, changeSetId),
    );
  },

  async getOutgoingConnectionsCounts(workspaceId: string, changeSetId: string) {
    return await withRemote(
      async (remote) =>
        await remote.getOutgoingConnectionsCounts(workspaceId, changeSetId),
    );
  },

  async getComponentDetails(workspaceId: string, changeSetId: string) {
    return await withRemote(
      async (remote) =>
        await remote.getComponentDetails(workspaceId, changeSetId),
    );
  },

  async getSchemaMembers(workspaceId: string, changeSetId: string) {
    return await withRemote(
      async (remote) => await remote.getSchemaMembers(workspaceId, changeSetId),
    );
  },

  async mjolnir(
    workspaceId: string,
    changeSetId: string,
    kind: EntityKind,
    id: string,
    checksum?: string,
  ) {
    await withRemote(
      async (remote) =>
        await remote.mjolnir(workspaceId, changeSetId, kind, id, checksum),
    );
  },

  async changeSetExists(
    workspaceId: string,
    changeSetId: string,
  ): Promise<boolean> {
    return await withRemote(
      async (remote) => await remote.changeSetExists(workspaceId, changeSetId),
    );
  },

  async niflheim(workspaceId: string, changeSetId: string): Promise<boolean> {
    return await withRemote(
      async (remote) => await remote.niflheim(workspaceId, changeSetId),
    );
  },

  async pruneAtomsForClosedChangeSet(workspaceId: string, changeSetId: string) {
    return await withRemote(
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
    return await withRemote(async (remote) => await remote.exec(opts));
  },

  async bobby(): Promise<void> {
    await withRemote(async (remote) => await remote.bobby());
  },

  async ragnarok(
    workspaceId: string,
    changeSetId: string,
    noColdStart?: boolean,
  ): Promise<void> {
    await withRemote(
      async (remote) =>
        await remote.ragnarok(workspaceId, changeSetId, noColdStart),
    );
  },

  async odin(changeSetId: string): Promise<object> {
    return await withRemote(async (remote) => await remote.odin(changeSetId));
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
