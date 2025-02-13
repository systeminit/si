import * as Comlink from "comlink";

import sqlite3InitModule, { Database, Sqlite3Static, SqlValue } from '@sqlite.org/sqlite-wasm';
import ReconnectingWebSocket from "reconnecting-websocket";
import { UpsertPayload, PatchPayload, PayloadDelete, DBInterface, NOROW, Checksum, ROWID, Atom, QueryKey, Args, RawArgs, RawAtom } from "./types/dbinterface";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { WorkspacePk } from "@/store/workspaces.store";

const log = console.log;
const error = console.error;

let db: Database;

const start = async (sqlite3: Sqlite3Static) => {
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
  await db.exec({ sql: 'PRAGMA foreign_keys = ON;'});
  const result = db.exec({ sql: 'PRAGMA foreign_keys', returnValue: "resultRows" })
  log("PRAGMA foreign_keys: ", oneInOne(result), "?");
};

const initializeSQLite = async () => {
  try {
    log('Loading and initializing SQLite3 module...');
    const sqlite3 = await sqlite3InitModule({ print: log, printErr: error });
    log('Done initializing. Running demo...');
    await start(sqlite3);
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
  /**
   * GOAL: persist only data that is readable, once blob data is no longer viewable, get rid of it
   * PROBLEM: Objects exist across multiple changesets, so we cannot ever UPDATE atom
   * SOLUTION: We copy objects when we are given mutations
   * PROBLEM: We don't want to read every single blob and check internal references
   * SOLUTION Use snapshot checksums and FK snapshot_mtm relationships to delete
   */
  const sql = `
  DROP TABLE IF EXISTS datablobs;
  DROP TABLE IF EXISTS atoms;
  DROP TABLE IF EXISTS snapshots_mtm_atoms;
  DROP TABLE IF EXISTS snapshots;
  DROP TABLE IF EXISTS changesets;

  CREATE TABLE IF NOT EXISTS changesets (
    change_set_id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL
  );
  CREATE INDEX IF NOT EXISTS changeset_workspace_id ON changesets(workspace_id);

  CREATE TABLE IF NOT EXISTS snapshots (
    id INTEGER PRIMARY KEY,
    checksum TEXT UNIQUE NOT NULL,
    change_set_id TEXT NOT NULL,
    FOREIGN KEY (change_set_id) REFERENCES changesets(change_set_id) ON DELETE CASCADE
  );

  CREATE TABLE IF NOT EXISTS atoms (
    id INTEGER PRIMARY KEY,
    kind TEXT,
    args TEXT,
    checksum TEXT,
    data BLOB,
    CONSTRAINT uniqueness UNIQUE (kind, args, checksum)
  );

  CREATE TABLE IF NOT EXISTS snapshots_mtm_atoms (
    snapshot_id INTEGER,
    atom_id INTEGER,
    FOREIGN KEY (snapshot_id) REFERENCES snapshots(id) ON DELETE CASCADE,
    FOREIGN KEY (atom_id) REFERENCES atoms(id) ON DELETE CASCADE,
    PRIMARY KEY (snapshot_id, atom_id)
  ) WITHOUT ROWID;
  `;
  /**
   * RULES:
   * When an Atom is deleted, delete its MTM entry (CASCADE should take care of this)
   * When a Snapshot is deleted, delete its MTM entry, bot not its atoms (CASCADE should take care of this)
   * 
   * When a Changeset is closed/deleted:
   *  - delete atoms connected to its snapshot MTMs (We can not CASCADE atom deletion)
   *  - delete its record, CASCADE should delete its snapshots and MTMs
   * 
   * PATCH WORKFLOW:
   * When we are given a new snapshot along with patch data:
   *  - rowid = INSERT INTO snapshots <new_checksum>, <this_changeSetId>
   *  - INSERT INTO snapshots_mtm_atoms SELECT <rowid>, atom_rowid WHERE checksum="<old_checksum>" AND change_set_id=<this_changeSetId>
   *  - UPDATE changesets SET snapshot_id = rowid
   *  - For each patch data
   *    - fromChecksum = 0, this is net new, insert atom
   *    - toChecksum = 0, this is a deletion, remove atom
   *    - nonzero checksums:
   *      - select * from atoms where kind=<kind>, args=<args>, checksum=<old_checksum>
   *        - if data doesn't exist throw mjolnir
   *      - apply patch data
   *      - atom_rowid = insert into atoms data=<blob>, kind=<kind>, args=<args>, checksum=<new_checksum>
   *      - insert into snapshots_mtm_atoms atom_rowid = atom_rowid, snapshot_rowid = rowid
   *  - DELETE FROM snapshots WHERE change_set_id=<this_changeSetId> AND checksum=<old_checksum>
   */

  return db.exec({sql});
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
    bind: [rowid, atom.kind, atom.newChecksum, atom.args.toString()],
    returnValue: "resultRows",
  });
  return rows.length > 0;
};

// SEE PATCH WORKFLOW
const newSnapshot = async (atom: Atom): Promise<ROWID> => {
  const created = await db.exec({
    sql: `INSERT INTO snapshots (change_set_id, checksum) VALUES (?, ?);`,
    bind: [atom.changeSetId, atom.newChecksum],
    returnValue: "resultRows",
  });
  const snapshot_id = oneInOne(created);
  if (snapshot_id === NOROW) throw new Error("Insertion Failed");
  return snapshot_id as number;
};

const newSnapshotEnd = async(changeSetId: ChangeSetId, fromSnapshotChecksum: Checksum) => {
  await db.exec({
    sql: "DELETE FROM snapshots WHERE change_set_id = '?' AND checksum = '?'",
    bind: [changeSetId, fromSnapshotChecksum],
  })
};

const removeAtom = async (atom: Atom) => {
  await db.exec({
    sql: `delete
    from atoms
    where
      kind='?' and
      checksum='?' and
      args='?'
    ;
    `,
    bind: [atom.kind, atom.origChecksum, atom.args.toString()],
  }); // CASCADES to the mtm table
};

const createAtom = async (atom: Atom, snapshot_rowid: ROWID): Promise<ROWID> => {
  const data = await new Blob([atom.data]).arrayBuffer();
  const rows = await db.exec({
    sql: `insert into atoms
      (kind, checksum, args, data)
        VALUES
      (?, ?, ?, ?)
    returning rowid;
    `,
    bind: [atom.kind, atom.newChecksum, atom.args.toString(), data],
    returnValue: "resultRows",
  });
  const atom_rowid = oneInOne(rows);
  if (atom_rowid === NOROW) throw new Error("NOROW when inserting");
  return atom_rowid as ROWID;
};

const partialKeyFromKindAndArgs = async (kind: string, args: Args): Promise<QueryKey> => {
  return `${kind}|${args.toString()}`;
};

// FUTURE: maybe not only atoms come over the wire?
const handleEvent = async (messageEvent: MessageEvent<any>) => {
  const data = JSON.parse(messageEvent.data) as RawAtom[];
  if (data.length === 0) return;
  // Assumption: every patch is working on the same workspace and changeset
  // (e.g. we're not bundling messages across workspaces somehow)
  const changeSetId = data[0]?.changeSetId;
  if (!changeSetId) throw new Error("Expected changeSetId")

  const snapshots: Record<string, ROWID> = {};
  const oldSnapshots: string[] = [];
  const atoms = await Promise.all(data.map(async (rawAtom) => {
    const atom: Atom = {
      ...rawAtom,
      args: new Args(rawAtom.args as RawArgs),
    };

    let rowid = await findSnapshotRowId(
      atom.toSnapshotChecksum, atom.changeSetId
    );
    if (rowid === NOROW) {
      rowid = await newSnapshot(atom);
      oldSnapshots.push(atom.fromSnapshotChecksum);
    } 
    snapshots[atom.toSnapshotChecksum] = rowid;

    return atom;
  }));

  atoms.forEach((atom) => {
    const rowid = snapshots[atom.toSnapshotChecksum];
    if (!rowid) throw new Error(`Expected ROWID for ${atom.toSnapshotChecksum}`);
    handleAtom(atom, rowid);
  });

  oldSnapshots.forEach((checksum) => {
    newSnapshotEnd(changeSetId, checksum)
  })
};

const handleAtom = async (atom: Atom, rowid: ROWID) => {
  const exists = await atomExists(atom, rowid);
  let atomid: ROWID | undefined;
  if (atom.fromSnapshotChecksum === "0") {
    if (!exists)  // if i already have it, this is a NOOP
      atomid = await createAtom(atom, rowid);
  } else if (atom.toSnapshotChecksum === "0")
    // if i've already removed it, this is a NOOP
    if (exists) await removeAtom(atom);
  else {
    // patch it if I can
    if (exists)
        atomid = await patchAtom(atom);
    // otherwise, fire the small hammer to get the full object
    else mjolnir(atom.kind, atom.args);
  }

  if (atomid)
    await db.exec({
      sql: `insert into snapshots_mtm_atoms
        (atom_rowid, snapshot_rowid)
          VALUES
        (?, ?);`,
      bind: [atomid, rowid],
    });
};

const patchAtom = async (atom: Atom): Promise<ROWID> => {
  // FUTURE: JSON Patch, where we select the old data, and patch it
  // just inserting right now
  const data = await new Blob([atom.data]).arrayBuffer();
  const rows = await db.exec({
    sql: `
    insert into atoms
      (kind, args, checksum, data)
    values
      ('?', '?', '?', '?')
    ;
    `,
    bind: [atom.kind, atom.args.toString(), atom.newChecksum, data],
    returnValue: "resultRows",
  });
  const atom_rowid = oneInOne(rows);
  if (atom_rowid === NOROW) throw new Error("NOROW when inserting");
  return atom_rowid as ROWID;
};

const mjolnir = async (kind: string, args: Args) => {
  // TODO: we're missing a key, fire a small hammer to get it
};

// FUTURE: when we have changeset data
const pruneAtomsForClosedChangeSet = async (workspaceId: WorkspacePk, changeSetId: ChangeSetId) => {
};

const ragnarok = () => {
  // FUTURE: drop the DB, rebuild it, and enter keys from empty
};

let socket: ReconnectingWebSocket;
let bustCacheFn;
const dbInterface: DBInterface = {
  async initDB() {
    return initializeSQLite();
  },

  async migrate() {
    const result = ensureTables();
    log("Migration completed");
    return result;
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
      handleEvent(messageEvent);
    });
  
    socket.addEventListener("error", (errorEvent) => {
      log("ws error", errorEvent.error, errorEvent.message);
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

  async addListenerBustCache (cb: (queryKey: QueryKey, latestChecksum: Checksum) => void) {
    bustCacheFn = cb;
    bustCacheFn("foo", "bar2");
  },

  async get(kind: string, args: Args, checksum: Checksum): Promise<unknown> {
    // TODO: parse json string data from the results
    return {};
  },

  partialKeyFromKindAndArgs,
  mjolnir, 

  async bootstrapChecksums(changeSetId: ChangeSetId): Promise<Record<QueryKey, Checksum>> {
    const mapping: Record<QueryKey, Checksum> = {};
    const rows = await db.exec({sql: `
      select atoms.kind, atoms.args, atoms.checksum
      from atoms
      inner join snapshots_mtm_atoms mtm ON atoms.id = mtm.atom_id
      inner join snapshots ON mtm.snapshot_id = snapshots.id
      where snapshots.change_set_id = '?'
      ;
      `, 
      bind: [changeSetId],
      returnValue: "resultRows"});
    rows.forEach((row) => {
      const key = `${row[0]}|${row[1]}` as QueryKey;
      const checksum = row[3] as Checksum;
      mapping[key] = checksum;
    })
    return mapping;
  },

  async fullDiagnosticTest(changeSetId: ChangeSetId) {
    // TODO
  }
};


Comlink.expose(dbInterface);