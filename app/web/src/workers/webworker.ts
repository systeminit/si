import * as Comlink from "comlink";

import sqlite3InitModule, { Database, Sqlite3Static, SqlValue } from '@sqlite.org/sqlite-wasm';
import ReconnectingWebSocket from "reconnecting-websocket";
import { UpsertPayload, PatchPayload, PayloadDelete, DBInterface } from "./types/dbinterface";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { WorkspacePk } from "@/store/workspaces.store";

const log = console.log;
const error = console.error;

let db: Database;

const start = (sqlite3: Sqlite3Static) => {
  log('Running SQLite3 version', sqlite3.version.libVersion);
  db =
    'opfs' in sqlite3
      ? new sqlite3.oo1.OpfsDb('/si.sqlite3')
      : new sqlite3.oo1.DB('/si.sqlite3', 'ct');
  log(
    'opfs' in sqlite3
      ? `OPFS is available, created persisted database at ${db.filename}`
      : `OPFS is not available, created transient database ${db.filename}`,
  );
  // Your SQLite code here.
};

const initializeSQLite = async () => {
  try {
    log('Loading and initializing SQLite3 module...');
    const sqlite3 = await sqlite3InitModule({ print: log, printErr: error });
    log('Done initializing. Running demo...');
    start(sqlite3);
  } catch (err) {
    if (err instanceof Error) 
      error('Initialization error:', err.name, err.message);
    else
      error('Initialization error:', err);
  }
};

// INTEGER is 8 bytes, not large enough to store ULIDs
// we'll go with string, though reading that putting the bytes as BLOBs would save space
const ensureTables = async () => {
  const sql = `
  CREATE TABLE IF NOT EXISTS changesets (
    change_set_id TEXT PRIMARY KEY,
    workspace_id TEXT
  ) WITHOUT ROWID;

  CREATE TABLE IF NOT EXISTS snapshots (
    change_set_id TEXT,
    checksum TEXT,
    PRIMARY KEY (change_set_id, checksum)
  );

  CREATE TABLE IF NOT EXISTS datablobs (
    snapshot_rowid INTEGER,
    kind TEXT,
    args TEXT,
    checksum TEXT,
    data BLOB,
    PRIMARY KEY (snapshot_rowid, kind, args, checksum)
  ) WITHOUT ROWID;
  `;

  return db.exec({sql});
}

// TODO
const patchJSON = () => {

}

let bustCacheFn;

// TODO
const handleBlob = (payload: {
  workspaceId: WorkspacePk,
  changeSetId: ChangeSetId,
  fromSnapshotChecksum: string,
  toSnapshotChecksum: string,
  kind: string,
  args: Record<string, string>,
  origChecksum: string,
  newChecksum: string,
  data: unknown, // TODO, giant enum
}) => {
  
};

let socket: ReconnectingWebSocket;

const dbInterface: DBInterface = {
  async initDB() {
    return initializeSQLite();
  },

  async migrate() {
    return ensureTables();
  },

  async initSocket (url: string, bearerToken: string) {
    socket = new ReconnectingWebSocket(
      () =>
        `${url}/bifrost?token=Bearer+${bearerToken}`,
      [],
      {
        // see options https://www.npmjs.com/package/reconnecting-websocket#available-options
        startClosed: true, // don't start connected - we'll watch auth to trigger
        // TODO: tweak settings around reconnection behaviour
      },
    );

    socket.addEventListener("message", (messageEvent) => {
      const messageEventData = JSON.parse(messageEvent.data);
    });
    socket.addEventListener("error", (errorEvent) => {
      /* eslint-disable-next-line no-console */
      console.log("ws error", errorEvent.error, errorEvent.message);
    });
  },

  async initBifrost(url: string, bearerToken: string) {
    await Promise.all([this.initDB(), ]); // this.initSocket(url, bearerToken)]);
    await this.migrate();
  },

  async bifrostClose () {
    // socket.close();
  },

  async bifrostReconnect() {
    // socket.reconnect();
  },

  async testRainbowBridge() {
    await db.exec({sql: "delete from snapshots; insert into snapshots (change_set_id, checksum) values ('foo', 'bar'), ('apple', 'orange');"});
    const columns: string[] = [];
    const rows = await db.exec({sql: "select rowid, change_set_id, checksum from snapshots;", returnValue: "resultRows", columnNames: columns});
    return {rows, columns};
  },

  async addListenerBustCache (cb: (key: string) => void) {
    bustCacheFn = cb;
    cb("proof");
  },

  bifrost: {
    pull(changeSetId: ChangeSetId, kind: string, args: Record<string, string>): SqlValue[][] {
      return [];
    }
  }
};


Comlink.expose(dbInterface);