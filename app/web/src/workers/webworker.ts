import * as Comlink from "comlink";

import sqlite3InitModule, { Database, Sqlite3Static, SqlValue } from '@sqlite.org/sqlite-wasm';
import ReconnectingWebSocket from "reconnecting-websocket";
import { UpsertPayload, PatchPayload, PayloadDelete, DBInterface } from "./types/dbinterface";

const log = console.log;
const error = console.error;


/*
class DBInterface extends EventTarget {
  #ws;
  #promiser;
  #dbId;

  constructor (url: string) {
    super();
    this.#ws = new ReconnectingWebSocket(url);
    try {
      log('Loading and initializing SQLite3 module...');

      this.#promiser = await new Promise((resolve) => {
        const _promiser = sqlite3Worker1Promiser({
          onready: () => resolve(_promiser),
        });
      });

      log('Done initializing. Running demo...');

      const configResponse = await this.#promiser('config-get', {});
      log('Running SQLite3 version', configResponse.result.version.libVersion);

      const openResponse = await this.#promiser('open', {
        filename: 'file:mydb.sqlite3?vfs=opfs',
      });
      const { dbId } = openResponse;
      this.#dbId = dbId;

      log(
        'OPFS is available, created persisted database at',
        openResponse.result.filename.replace(/^file:(.*?)\?vfs=opfs$/, '$1'),
      );
    
    } catch (err: Error) {
      error(err.name, err.message);
    }

  };

  activate() {
    this.#ws.addEventListener("message", (msg: any) => {
      if ("data" in msg) return
      const payload = JSON.parse(msg.data) as UpsertPayload | PatchPayload | PayloadDelete;

      this.bustKey(payload)

      this.savePayload(payload);
    });
  }

  const bustKey = (payload: UpsertPayload | PatchPayload | PayloadDelete) => {
    const mainQueryKey = [payload.kind, payload.changeSetId];
    if (payload.args) mainQueryKey.push(...Object.values(payload.args));
  }

  const savePayload = (payload: UpsertPayload | PatchPayload | PayloadDelete) => {

  }

  const readKey = () => {

  }

  #read = async () => {
    await this.#promiser('exec', {
      dbId: this.#dbId,
      sql: 'INSERT INTO t(a,b) VALUES (?,?)',
      bind: [i, i * 2],
    });
  };

  #write = async () => {
    await this.#promiser('exec', {
      dbId: this.#dbId,
      sql: 'INSERT INTO t(a,b) VALUES (?,?)',
      bind: [i, i * 2],
    });
  };

}

const url = "" // from config
const interface = new DBInterface(url);

*/

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


const dbInterface: DBInterface = {
  hello()  {
    log("Hello world?");
    return "From inner";
  },

  async init() {
    return initializeSQLite();
  },

  async migrate() {
    return ensureTables();
  },

  async smokeTest() {
    console.log("called")
    await db.exec({sql: "delete from snapshots; insert into snapshots (change_set_id, checksum) values ('foo', 'bar'), ('apple', 'orange');"});
    const columns: string[] = [];
    const rows = await db.exec({sql: "select rowid, change_set_id, checksum from snapshots;", returnValue: "resultRows", columnNames: columns});
    return {rows, columns};
  }
}
Comlink.expose(dbInterface);