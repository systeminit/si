import * as Comlink from "comlink";
import {
  ExecBaseOptions,
  ExecReturnResultRowsOptions,
  ExecRowModeArrayOptions,
  FlexibleString,
  SqlValue,
} from "@sqlite.org/sqlite-wasm";
import {
  SharedDBInterface,
  TabDBInterface,
  NOROW,
  AtomDocument,
  BroadcastMessage,
  SHARED_BROADCAST_CHANNEL_NAME,
  Gettable,
  Listable,
} from "./types/dbinterface";
import { EntityKind } from "./types/entity_kind_types";

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

let bearerToken: string;
let currentRemote: Comlink.Remote<TabDBInterface> | undefined;
let currentRemoteId: string | undefined;
const remotes: { [key: string]: Comlink.Remote<TabDBInterface> } = {};

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
    delete remotes[id];
  },
  registerRemote(id: string, remote: Comlink.Remote<TabDBInterface>) {
    if (!remotes[id]) {
      debug("register remote in shared", id);
      remotes[id] = remote;
    }
  },
  async hasRemote() {
    return !!currentRemote;
  },
  async setRemote(remoteId: string) {
    debug("setting remote in shared web worker to", remoteId);

    const wasConnected = !!currentRemote;

    currentRemote = remotes[remoteId];
    if (!currentRemote) {
      throw new Error(`remote {$remoteId} not registered`);
    }
    currentRemoteId = remoteId;

    // Ensure we reconnect the websocket if we already had a remote
    // (otherwise we let heimdall decide when to connect the websocket)
    if (wasConnected) {
      currentRemote.bifrostReconnect();
    }

    hasRemoteChannel.port2.postMessage("got remote");
    if (bearerToken) {
      currentRemote.setBearer(bearerToken);
    }
  },

  async initDB(_testing: boolean) {
    debug("init db called in shared webworker");
  },

  async migrate(testing: boolean) {
    return withRemote(async (remote) => await remote.migrate(testing));
  },

  setBearer(token: string): void {
    bearerToken = token;
    currentRemote?.setBearer(bearerToken);
  },

  async initSocket(): Promise<void> {
    await withRemote(async (remote) => await remote.initSocket());
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

  async getPossibleConnections(
    workspaceId,
    changeSetId,
    destSchemaName,
    destProp,
  ) {
    return await withRemote(
      async (remote) =>
        await remote.getPossibleConnections(
          workspaceId,
          changeSetId,
          destSchemaName,
          destProp,
        ),
    );
  },

  async getOutgoingConnectionsCounts(workspaceId: string, changeSetId: string) {
    return await withRemote(
      async (remote) =>
        await remote.getOutgoingConnectionsCounts(workspaceId, changeSetId),
    );
  },

  async getComponentNames(workspaceId: string, changeSetId: string) {
    return await withRemote(
      async (remote) =>
        await remote.getComponentNames(workspaceId, changeSetId),
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
self.onconnect = (event: MessageEvent) => {
  Comlink.expose(dbInterface, event.ports[0]);
  onConnectBroadcast.postMessage("booted");
};
