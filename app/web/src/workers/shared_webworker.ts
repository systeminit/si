import * as Comlink from "comlink";
import {
  ExecBaseOptions,
  ExecReturnResultRowsOptions,
  ExecRowModeArrayOptions,
  FlexibleString,
  SqlValue,
} from "@sqlite.org/sqlite-wasm";
import { Span } from "@opentelemetry/api";
import {
  BustCacheFn,
  DBInterface,
  LobbyExitFn,
  RainbowFn,
  NOROW,
  PatchBatch,
  AtomMessage,
  AtomDocument,
  BroadcastMessage,
  SHARED_BROADCAST_CHANNEL_NAME,
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
let currentRemote: Comlink.Remote<DBInterface> | undefined;
let currentRemoteId: string | undefined;
const remotes: { [key: string]: Comlink.Remote<DBInterface> } = {};

const hasRemoteChannel = new MessageChannel();

const getRemote = (): Promise<Comlink.Remote<DBInterface>> => {
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
  cb: (remote: Comlink.Remote<DBInterface>) => Promise<R>,
): Promise<R> {
  const remote = await getRemote();
  return cb(remote);
}

const dbInterface: DBInterface = {
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
  async receiveBroadcast(_message) {
    debug("shared worker received a broadcast?");
  },
  unregisterRemote(id: string) {
    debug("unregister remote in shared", id);
    delete remotes[id];
  },
  registerRemote(id: string, remote: Comlink.Remote<DBInterface>) {
    debug("register remote in shared", id);
    remotes[id] = remote;
  },
  async setRemote(remoteId: string) {
    debug("setting remote in shared web worker");

    currentRemote = remotes[remoteId];
    if (!currentRemote) {
      throw new Error(`remote {$remoteId} not registered`);
    }

    currentRemoteId = remoteId;
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
    kind: EntityKind,
    id: string,
  ): Promise<typeof NOROW | AtomDocument> {
    return await withRemote(
      async (remote) => await remote.get(workspaceId, changeSetId, kind, id),
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

  async mjolnirBulk(
    workspaceId: string,
    changeSetId: string,
    objs: Array<{ kind: string; id: string; checksum?: string }>,
    indexChecksum: string,
  ) {
    await withRemote(
      async (remote) =>
        await remote.mjolnirBulk(workspaceId, changeSetId, objs, indexChecksum),
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

  partialKeyFromKindAndId(kind: EntityKind, id: string): string {
    return `${kind}|${id}`;
  },

  kindAndIdFromKey(key: string): { kind: EntityKind; id: string } {
    const pieces = key.split("|", 2);
    if (pieces.length !== 2) throw new Error(`Bad key ${key} -> ${pieces}`);
    if (!pieces[0] || !pieces[1])
      throw new Error(`Missing key ${key} -> ${pieces}`);
    const kind = pieces[0] as EntityKind;
    const id = pieces[1];
    return { kind, id };
  },

  addListenerBustCache(_fn: BustCacheFn): void {},

  addListenerInFlight(_fn: RainbowFn): void {},

  addListenerReturned(_fn: RainbowFn): void {},

  addListenerLobbyExit(_fn: LobbyExitFn): void {},

  async atomChecksumsFor(changeSetId: string): Promise<Record<string, string>> {
    return await withRemote(
      async (remote) => await remote.atomChecksumsFor(changeSetId),
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

  oneInOne(rows: SqlValue[][]): SqlValue | typeof NOROW {
    const first = rows[0];
    if (first) {
      const id = first[0];
      if (id || id === 0) return id;
    }
    return NOROW;
  },

  async encodeDocumentForDB(doc: object): Promise<ArrayBuffer> {
    return await new Blob([JSON.stringify(doc)]).arrayBuffer();
  },

  decodeDocumentFromDB(doc: ArrayBuffer): AtomDocument {
    const s = new TextDecoder().decode(doc);
    const j = JSON.parse(s);
    return j;
  },

  async handlePatchMessage(data: PatchBatch, span?: Span): Promise<void> {
    await withRemote(
      async (remote) => await remote.handlePatchMessage(data, span),
    );
  },

  async handleHammer(msg: AtomMessage, span?: Span): Promise<void> {
    await withRemote(async (remote) => await remote.handleHammer(msg, span));
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
