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
  db.exec({ sql: 'PRAGMA foreign_keys = ON;'});
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
  DROP TABLE IF EXISTS changesets;
  CREATE TABLE IF NOT EXISTS changesets (
    change_set_id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    PRIMARY KEY (change_set_id, workspace_id)
  ) WITHOUT ROWID;

  DROP TABLE IF EXISTS snapshots;
  CREATE TABLE IF NOT EXISTS snapshots (
    change_set_id TEXT NOT NULL,
    checksum TEXT NOT NULL,
    PRIMARY KEY (change_set_id, checksum),
    FOREIGN KEY (change_set_id) REFERENCES changesets(change_set_id) ON DELETE CASCADE
  );

  DROP TABLE IF EXISTS datablobs;
  DROP TABLE IF EXISTS atoms;
  CREATE TABLE IF NOT EXISTS atoms (
    kind TEXT,
    args TEXT,
    checksum TEXT,
    data BLOB,
    CONSTRAINT uniqueness UNIQUE (kind, args, checksum)
  );

  DROP TABLE IF EXISTS snapshots_mtm_atoms;
  CREATE TABLE IF NOT EXISTS snapshots_mtm_atoms (
    snapshot_rowid INTEGER,
    atom_rowid INTEGER,
    PRIMARY KEY (snapshot_rowid, atom_rowid),
    FOREIGN KEY (snapshot_rowid) REFERENCES snapshots(rowid) ON DELETE CASCADE,
    FOREIGN KEY (atom_rowid) REFERENCES atoms(rowid) ON DELETE CASCADE
  );
  `;

  return db.exec({sql});
}


type Checksum = string;
type ROWID = number;
const NOROW = Symbol("NOROW");
interface Atom {
  workspaceId: WorkspacePk,
  changeSetId: ChangeSetId,
  fromSnapshotChecksum: string,
  toSnapshotChecksum: string,
  kind: string,
  args: Record<string, string>,
  origChecksum: string,
  newChecksum: string,
  data: string, // this is a string of JSON we're not parsing
}

// CONSTRAINT: right now there are either zero args (e.g. just workspace & changeset) or 1 (i.e. "the thing", ComponentId, ViewId, et. al)
const argsToString = (args: Record<string, string>): string => {
  const entries = Object.entries(args);
  const entry = entries.pop();
  if (!entry) return "";
  return entry.join("|");
}

const oneInOne = (rows: SqlValue[][]): SqlValue | typeof NOROW => {
  const first = rows[0];
  if (first) {
    const id = first[0];
    if (id) return id;
  }
  return NOROW;
}

const findSnapshotRowId = async (checksum: Checksum, changeSetId: ChangeSetId): Promise<ROWID | typeof NOROW> => {
  const rows = await db.exec({
    sql: `select
      rowid
    from
      snapshots
    where
      checksum='?' and
      change_set_id='?'
    ;`,
    bind: [checksum, changeSetId],
    returnValue: "resultRows",
  });
  const maybeRowId = oneInOne(rows);
  if (maybeRowId !== NOROW) return maybeRowId as ROWID;
  else return maybeRowId; // NOROW
}

const atomExists = async (atom: Atom, rowid: ROWID): Promise<boolean> => {
  const args = argsToString(atom.args)
  const rows = await db.exec({
    sql: `
    select
      snapshot_rowid
    from atoms
    inner join snapshots_mtm_atoms
      ON atoms.rowid = snapshots_mtm_atoms.atom_rowid
    where
      snapshot_rowid='?' and
      kind='?' and checksum='?' and
      args='?'
    ;`,
    bind: [rowid, atom.kind, atom.newChecksum, args],
    returnValue: "resultRows",
  });
  return rows.length > 0;
};

const newSnapshot = (atom: Atom): ROWID => {

};

const removeAtom = async (atom: Atom) => {
  const args = argsToString(atom.args);
  await db.exec({
    sql: `delete
    from atoms
    where
      kind='?' and
      checksum='?' and
      args='?'
    ;
    `,
    bind: [atom.kind, atom.origChecksum, args],
  }); // CASCADES to the mtm table
};

const newAtom = async (atom: Atom, snapshot_rowid: ROWID) => {
  const args = argsToString(atom.args);
  const data = await new Blob([atom.data]).arrayBuffer();
  const rows = await db.exec({
    sql: `insert into atoms
      (kind, checksum, args, data)
        VALUES
      (?, ?, ?, ?)
    returning rowid;
    `,
    bind: [atom.kind, atom.newChecksum, args, data],
    returnValue: "resultRows",
  });
  const atom_rowid = oneInOne(rows);
  if (atom_rowid === NOROW) throw new Error("NOROW when inserting");
  else {
    await db.exec({
      sql: `insert into snapshots_mtm_atoms
        (atom_rowid, snapshot_rowid)
          VALUES
        (?, ?);`,
      bind: [atom_rowid, snapshot_rowid],
      returnValue: "resultRows",
    });
  }
};


const partialKeyFromKindAndArgs = async (kind: string, args: Record<string, string>): Promise<string> => {
  return `${kind}|${argsToString(args)}`;
};

const handleAtom = async (atom: Atom) => {
  let rowid = await findSnapshotRowId(atom.toSnapshotChecksum, atom.changeSetId)
  if (rowid === NOROW)
    rowid = await newSnapshot(atom);

  const exists = await atomExists(atom, rowid);
  if (atom.fromSnapshotChecksum === "0") {
    if (!exists)  // if i already have it, this is a NOOP
      await newAtom(atom, rowid);
  } else if (atom.toSnapshotChecksum === "0")
    // if i've already removed it, this is a NOOP
    if (exists) await removeAtom(atom);
  else {
    // patch it if I can
    if (exists) await patchAtom(atom);
    // otherwise, fire the small hammer to get the full object
    else mjolnir(await partialKeyFromKindAndArgs(atom.kind, atom.args));
  }
};

const patchAtom = async (atom: Atom) => {
  // FUTURE: JSON Patch
  const args = argsToString(atom.args);
  const data = await new Blob([atom.data]).arrayBuffer();
  await db.exec({
    sql: `
    update atoms set
      data = '?',
      checksum = '?'
    where
      kind = '?' AND
      checksum = '?' AND
      args = '?'
    ;
    `,
    bind: [data, atom.newChecksum, atom.kind, atom.origChecksum, args],
  });
};

const mjolnir = async (key: string) => {
  // TODO: we're missing a key, fire a small hammer to get it
};

// FUTURE: when we have more than atom payloads over the wire
const pruneAtomsForClosedChangeSet = async (workspaceId: WorkspacePk, changeSetId: ChangeSetId) => {
  await db.exec({
    sql: "delete from changesets where workspace_id='?' and change_set_id='?';",
    bind: [workspaceId, changeSetId],
  });  // CASCADE deletions to all snapshots from that changeset and all atoms
};

const ragnarok = () => {
  // FUTURE: drop the DB, rebuild it, and enter keys from empty
};

let bustCacheFn;

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
      const messageEventData = JSON.parse(messageEvent.data) as Atom;
      // FUTURE: not only atoms
      handleAtom(messageEventData);
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
    // TODO test blob in the atoms table!
    await db.exec({sql: "delete from snapshots; insert into snapshots (change_set_id, checksum) values ('foo', 'bar'), ('apple', 'orange');"});
    const columns: string[] = [];
    const rows = await db.exec({sql: "select rowid, change_set_id, checksum from snapshots;", returnValue: "resultRows", columnNames: columns});
    return {rows, columns};
  },

  async addListenerBustCache (cb: (queryKey: string, latestChecksum: string) => void) {
    bustCacheFn = cb;
    bustCacheFn("foo", "bar2");
  },

  async get(key: string): Promise<unknown> {
    // TODO: parse json string data from the results
    return {};
  },

  partialKeyFromKindAndArgs,
  mjolnir, 

  async bootstrapChecksums(): Promise<Record<string, Checksum>> {
    // TODO: read the full list of atom checksums by queryKey
    return {};
  },
};


Comlink.expose(dbInterface);