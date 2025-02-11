import * as Comlink from "comlink";

import sqlite3InitModule, { Sqlite3Static } from '@sqlite.org/sqlite-wasm';
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

const start = (sqlite3: Sqlite3Static) => {
  log('Running SQLite3 version', sqlite3.version.libVersion);
  const db =
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


const db: DBInterface = {
  hello()  {
    log("Hello world?");
    return "From inner";
  },

  async init() {
    return initializeSQLite();
  }
}
Comlink.expose(db);