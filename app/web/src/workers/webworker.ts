/*
// In `main.js`.
const worker = new Worker('worker.js', { type: 'module' });
*/

// In `worker.js`.
import sqlite3Worker1Promiser from '@sqlite.org/sqlite-wasm';

const log = console.log;
const error = console.error;

import ReconnectingWebSocket from "reconnecting-websocket";
import { UpsertPayload, PatchPayload, PayloadDelete } from "./types";


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

onmessage = (e: MessageEvent) => {

};