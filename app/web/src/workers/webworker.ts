import * as Comlink from "comlink";
import { applyOperation } from 'fast-json-patch';
import sqlite3InitModule, { Database, Sqlite3Static, SqlValue } from '@sqlite.org/sqlite-wasm';
import ReconnectingWebSocket from "reconnecting-websocket";
import { DBInterface, NOROW, Checksum, ROWID, Atom, QueryKey, Args, RawArgs, AtomOperation, interpolate,  RowWithColumnsAndId, AtomMessage, AtomMeta, AtomDocument } from "./types/dbinterface";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { WorkspacePk } from "@/store/workspaces.store";
import { trace, Span, } from '@opentelemetry/api';
import { WebTracerProvider } from '@opentelemetry/sdk-trace-web';
import { BatchSpanProcessor, ConsoleSpanExporter } from '@opentelemetry/sdk-trace-base';
import { FetchInstrumentation } from '@opentelemetry/instrumentation-fetch';
import { registerInstrumentations } from '@opentelemetry/instrumentation';
// import { OTLPTraceExporter } from '@opentelemetry/exporter-otlp-http';
import { Resource } from '@opentelemetry/resources';
import { ATTR_SERVICE_NAME, ATTR_SERVICE_VERSION } from '@opentelemetry/semantic-conventions';
import { getProjectEnvVariables } from "../shared/dynamicEnvVars";

const { envVariables } = getProjectEnvVariables();

let otelEndpoint =
  envVariables.VITE_OTEL_EXPORTER_OTLP_ENDPOINT ??
  import.meta.env.VITE_OTEL_EXPORTER_OTLP_ENDPOINT;
if (!otelEndpoint) otelEndpoint = window.location.host;

const exporter = new ConsoleSpanExporter();
/* const exporter = new OTLPTraceExporter({
  url: `${otelEndpoint}/v1/traces`, 
}); */

const processor = new BatchSpanProcessor(exporter);

const provider = new WebTracerProvider({
  resource: new Resource({
      [ATTR_SERVICE_NAME]: 'bifrost',
      [ATTR_SERVICE_VERSION]: '0.1',
  }),
  spanProcessors: [
    processor
  ],
});

provider.register();

registerInstrumentations({
  instrumentations: [
      new FetchInstrumentation(),
  ],
});


const tracer = trace.getTracer('bifrost');
const log = console.log;
const error = console.error;

let db: Database;

const start = async (sqlite3: Sqlite3Static) => {
  // log('Running SQLite3 version', sqlite3.version.libVersion);
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
  // const result = await db.exec({ sql: 'PRAGMA foreign_keys', returnValue: "resultRows" })
  // log("PRAGMA foreign_keys: ", oneInOne(result), "?");
};

const initializeSQLite = async () => {
  try {
    // log('Loading and initializing SQLite3 module...');
    const sqlite3 = await sqlite3InitModule({ print: log, printErr: error });
    // log('Done initializing. Running demo...');
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
   *  - INSERT INTO snapshots_mtm_atoms SELECT <rowid>, atom_id WHERE checksum="<old_checksum>" AND change_set_id=<this_changeSetId>
   *  - UPDATE changesets SET snapshot_id = rowid
   *  - For each patch data
   *    - fromChecksum = 0, this is net new, insert atom
   *    - toChecksum = 0, this is a deletion, remove atom
   *    - nonzero checksums:
   *      - select * from atoms where kind=<kind>, args=<args>, checksum=<old_checksum>
   *        - if data doesn't exist throw mjolnir
   *      - apply patch data
   *      - atom_id = insert into atoms data=<blob>, kind=<kind>, args=<args>, checksum=<new_checksum>
   *      - insert into snapshots_mtm_atoms atom_id = atom_id, snapshot_id = rowid
   *  - DELETE FROM snapshots WHERE change_set_id=<this_changeSetId> AND checksum=<old_checksum>
   */

  return db.exec({sql});
}

const encodeDocumentForDB = async (doc: object) => {
  return await new Blob([JSON.stringify(doc)]).arrayBuffer();
};

const decodeDocumentFromDB = (doc: ArrayBuffer): AtomDocument => {
  const s = new TextDecoder().decode(doc);
  const j = JSON.parse(s);
  return j
};

const oneInOne = (rows: SqlValue[][]): SqlValue | typeof NOROW => {
  const first = rows[0];
  if (first) {
    const id = first[0];
    if (id) return id;
  }
  return NOROW;
}

const findSnapshotRowId = async (checksum: Checksum): Promise<RowWithColumnsAndId | typeof NOROW> => {
  const columns: string[] = [];
  const maybeRow = await db.exec({
    sql: `select
      id, change_set_id
    from
      snapshots
    where
      checksum=?
    ;`,
    bind: [checksum],
    returnValue: "resultRows",
    columnNames: columns,
  });
  const rows = interpolate(columns, maybeRow);
  if (rows.length === 0) return NOROW;
  else {
    const row = rows[0];
    if (row && "id" in row && row.id)
      return row as RowWithColumnsAndId; // shouldnt need the cast but its not picking it up
    else return NOROW;
  };
}

const atomExists = async (atom: Atom, isNew: boolean): Promise<boolean> => {
  const rows = await db.exec({
    sql: `
    select
      snapshot_id
    from atoms
    inner join snapshots_mtm_atoms
      ON atoms.id = snapshots_mtm_atoms.atom_id
    where
      kind=? and
      args=? and
      checksum = ?
    ;
    `,
    bind: [atom.kind, atom.args.toString(), isNew ? atom.kindToChecksum : atom.kindFromChecksum],
    returnValue: "resultRows",
  });
  return rows.length > 0;
};

// SEE PATCH WORKFLOW
const newSnapshot = async (meta: AtomMeta, fromId: number): Promise<ROWID> => {
  const changeSet = await db.exec({
    sql: `SELECT change_set_id FROM snapshots WHERE change_set_id = ? LIMIT 1;`,
    bind: [meta.changeSetId],
    returnValue: "resultRows",
  });
  const changeSetId = oneInOne(changeSet);
  if (changeSetId === NOROW) {
    await db.exec({
      sql: `INSERT INTO changesets (change_set_id, workspace_id) VALUES (?, ?);`,
      bind: [meta.changeSetId, meta.workspaceId],
    });
  }

  const created = await db.exec({
    sql: `INSERT INTO snapshots (change_set_id, checksum) VALUES (?, ?) RETURNING id;`,
    bind: [meta.changeSetId, meta.snapshotToChecksum],
    returnValue: "resultRows",
  });
  const snapshot_id = oneInOne(created);
  if (snapshot_id === NOROW) throw new Error("Insertion Failed");

  await db.exec({
    sql: `INSERT INTO snapshots_mtm_atoms
      SELECT 
        ?, atom_id
      FROM snapshots_mtm_atoms
      WHERE
        snapshot_id = ?
      `,
    bind: [snapshot_id, fromId]
  });
  return snapshot_id as number;
};

const removeOldSnapshots = async(changeSetId: ChangeSetId, fromSnapshotChecksum: Checksum) => {
  await db.exec({
    sql: "DELETE FROM snapshots WHERE change_set_id = ? AND checksum = ?",
    bind: [changeSetId, fromSnapshotChecksum],
  })
};

const removeAtom = async (atom: Atom) => {
  await db.exec({
    sql: `delete
    from atoms
    where
      kind=? and
      checksum=? and
      args=?
    ;
    `,
    bind: [atom.kind, atom.kindFromChecksum, atom.args.toString()],
  }); // CASCADES to the mtm table
};

const createAtom = async (atom: Atom): Promise<ROWID> => {
  const doc = {};
  atom.operations.forEach((op) => {
    applyOperation(doc, op, false, true);
  })
  const rows = await db.exec({
    sql: `insert into atoms
      (kind, checksum, args, data)
        VALUES
      (?, ?, ?, ?)
    returning id;
    `,
    bind: [
      atom.kind,
      atom.kindToChecksum,
      atom.args.toString(),
      await encodeDocumentForDB(doc),
    ],
    returnValue: "resultRows",
  });
  const atom_id = oneInOne(rows);
  if (atom_id === NOROW) throw new Error("NOROW when inserting");
  return atom_id as ROWID;
};

const partialKeyFromKindAndArgs = async (kind: string, args: Args): Promise<QueryKey> => {
  return `${kind}|${args.toString()}`;
};

// FUTURE: maybe not only atoms come over the wire?
const handleEvent = async (messageEvent: MessageEvent<any>, span?: Span ) => {
  const data = JSON.parse(messageEvent.data) as AtomMessage;
  span?.setAttribute("numRawAtoms", data.atoms.length);
  if (data.atoms.length === 0) return;
  // Assumption: every patch is working on the same workspace and changeset
  // (e.g. we're not bundling messages across workspaces somehow)
  const { changeSetId, workspaceId, snapshotFromChecksum: fromSnapshotChecksum, snapshotToChecksum: toSnapshotChecksum } = { ...data.meta };
  span?.setAttributes({ changeSetId, workspaceId, fromSnapshotChecksum, toSnapshotChecksum });

  if (!changeSetId) throw new Error("Expected changeSetId")

  const snapshots: Record<Checksum, ROWID> = {};
  const oldSnapshots: Checksum[] = [];

  if (!snapshots[fromSnapshotChecksum]) {
    const fromSnapshot = await findSnapshotRowId(fromSnapshotChecksum);
    if (fromSnapshot === NOROW) throw new Error("RAGNAROK!") // TODO
    snapshots[fromSnapshotChecksum] = fromSnapshot.id;
  }

  if (!snapshots[toSnapshotChecksum]) {
    const toSnapshot = await findSnapshotRowId(toSnapshotChecksum);
    if (toSnapshot !== NOROW) {
      snapshots[toSnapshotChecksum] = toSnapshot.id;
    } else {
      const fromSnapshotId = snapshots[fromSnapshotChecksum]
      if (!fromSnapshotId) throw new Error("Missing fromSnapshotId");
      const toSnapshotId = await newSnapshot(data.meta, fromSnapshotId);
      oldSnapshots.push(fromSnapshotChecksum);
      snapshots[toSnapshotChecksum] = toSnapshotId;
    }
  }

  const atoms = data.atoms.map((rawAtom) => {
    const atom: Atom = {
      ...rawAtom,
      args: new Args(rawAtom.args as RawArgs),
      operations: JSON.parse(rawAtom.operations),
      workspaceId,
      changeSetId,
      snapshotFromChecksum: fromSnapshotChecksum,
      snapshotToChecksum: toSnapshotChecksum,
    };
    return atom;
  });

  span?.setAttribute("numAtoms", atoms.length);
  await Promise.all(atoms.map(async (atom) => {
    const toSnapshotId = snapshots[atom.snapshotToChecksum];
    if (!toSnapshotId) throw new Error(`Expected snapshot ROWID for ${atom}`);
    await handleAtom(atom, toSnapshotId);
  }));

  span?.setAttribute("numSnapshotsRemoved", oldSnapshots.length);
  oldSnapshots.forEach((checksum) => {
    removeOldSnapshots(changeSetId, checksum)
  })
};

const handleAtom = async (atom: Atom, toSnapshotId: ROWID) => {
  await tracer.startActiveSpan("handleAtom", async (span) => {
    // if we have the change already don't do anything
    span.setAttribute("atom", JSON.stringify(atom));
    const noop = await atomExists(atom, true);
    if (noop) {
      span.addEvent("noop");
      span.end();
      return;
    }

    // otherwise, find the old record
    const exists = await atomExists(atom, false);
    span.setAttribute("exists", exists);
    let atomid: ROWID | undefined;
    if (atom.kindFromChecksum === "0") {
      if (!exists)  // if i already have it, this is a NOOP
        atomid = await createAtom(atom);
    } else if (atom.kindToChecksum === "0") {
      // if i've already removed it, this is a NOOP
      if (exists) await removeAtom(atom);
    } else {
      // patch it if I can
      if (exists)
          atomid = await patchAtom(atom, toSnapshotId);
      // otherwise, fire the small hammer to get the full object
      else {
        span.addEvent("mjolnir", { atom: JSON.stringify(atom) });
        mjolnir(atom.changeSetId, atom.kind, atom.args);
      }
    }

    if (atomid)
      await db.exec({
        sql: `insert into snapshots_mtm_atoms
          (atom_id, snapshot_id)
            VALUES
          (?, ?);`,
        bind: [atomid, toSnapshotId],
      });

    span.end()
  })
};

const patchAtom = async (atom: Atom, snapshotId: ROWID): Promise<ROWID> => {
  const atomRow = await db.exec({
    sql: `SELECT id, kind, args, checksum, data
      FROM atoms 
      INNER JOIN snapshots_mtm_atoms ON atoms.id = snapshots_mtm_atoms.atom_id
      WHERE
        snapshots_mtm_atoms.snapshot_id = ? and
        kind = ? and
        args = ? and
        checksum = ?
      `,
      bind: [snapshotId, atom.kind, atom.args.toString(), atom.kindFromChecksum],
      returnValue: "resultRows",
  });
  const atomId = oneInOne(atomRow);
  if (atomId === NOROW) throw new Error("Cannot find atom");
  // delete the MTM row that exists for the current snapshot, it will get replaced
  await db.exec({
    sql: `
    delete from snapshots_mtm_atoms
    where atom_id = ? and snapshot_id = ?
    ;`,
    bind: [atomId, snapshotId],
    returnValue: "resultRows",
  });

  // FUTURE: JSON Patch, where we select the old data, and patch it
  // just inserting right now
  const _doc = atomRow[0]?.[4] as ArrayBuffer;
  const doc = decodeDocumentFromDB(_doc);
  atom.operations.forEach((op) => {
    applyOperation(doc, op, false, true);
  })

  const rows = await db.exec({
    sql: `
    insert into atoms
      (kind, args, checksum, data)
    values
      (?, ?, ?, ?)
    returning id;
    ;`,
    bind: [
      atom.kind,
      atom.args.toString(),
      atom.kindToChecksum,
      await encodeDocumentForDB(doc),
    ],
    returnValue: "resultRows",
  });
  const atom_id = oneInOne(rows);
  if (atom_id === NOROW) throw new Error("NOROW when inserting");
  return atom_id as ROWID;
};

const mjolnir = async (changeSetId: ChangeSetId, kind: string, args: Args) => {
  // TODO: we're missing a key, fire a small hammer to get it
  log("MJOLNIR!")
};

// FUTURE: when we have changeset data
const pruneAtomsForClosedChangeSet = async (workspaceId: WorkspacePk, changeSetId: ChangeSetId) => {
  await tracer.startActiveSpan("prune", async (span) => {
    span.setAttributes({ workspaceId, changeSetId });
    await db.exec({
      sql: `
        DELETE FROM snapshots WHERE change_set_id = ?;
      `,
      bind: [changeSetId],
    });
    await db.exec({
      sql: `
        DELETE FROM changesets WHERE change_set_id = ?;
      `,
      bind: [changeSetId],
    });
    await db.exec({
      sql: `
        DELETE FROM atoms
        WHERE id IN (
          SELECT id FROM atoms
          LEFT JOIN snapshots_mtm_atoms ON snapshots_mtm_atoms.atom_id = atoms.id
          WHERE snapshots_mtm_atoms.atom_id IS NULL
        );
      `
    });
    span.end();
  });
};

const ragnarok = () => {
  // FUTURE: drop the DB data, rebuild it, and enter keys from empty
};

let socket: ReconnectingWebSocket;
let bustCacheFn;
const dbInterface: DBInterface = {
  async initDB() {
    return initializeSQLite();
  },

  async migrate() {
    const result = ensureTables();
    // log("Migration completed");
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
      const root = tracer.startActiveSpan("handleEvent", async (span) => {
        await handleEvent(messageEvent, span);
        span.end();
      })
    });
  
    socket.addEventListener("error", (errorEvent) => {
      error("ws error", errorEvent.error, errorEvent.message);
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

  async get(changeSetId: ChangeSetId, kind: string, args: Args): Promise<typeof NOROW | object> {
    const atomData = await db.exec({
      sql: `
      select
        data
      from
        atoms
        inner join snapshots_mtm_atoms mtm ON atoms.id = mtm.atom_id
        inner join snapshots ON mtm.snapshot_id = snapshots.id
      where
        snapshots.change_set_id = ? AND
        kind = ? AND
        args = ?
      ;`,
      bind: [changeSetId, kind, args.toString()],
      returnValue: "resultRows",
    });
    const data = oneInOne(atomData);
    if (data === NOROW) return NOROW
    const atomDoc = decodeDocumentFromDB(data as ArrayBuffer);
    return atomDoc
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
      where snapshots.change_set_id = ?
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

  async fullDiagnosticTest() {
    log("~~ DIAGNOSTIC STARTED ~~")
    const head = "HEAD";
    const workspace = "W";
    await db.exec({sql: `
        INSERT INTO changesets (change_set_id, workspace_id)
        VALUES (?, ?);
      `,
      bind: [head, workspace],
    });
    const checksum = 'HEAD';
    const snapshot = await db.exec({sql: `
        INSERT INTO snapshots (change_set_id, checksum)
        VALUES (?, ?) RETURNING id;
      `,
      bind: [head, checksum],
      returnValue: "resultRows"
    });
    const snapshot_id = oneInOne(snapshot);
    if (snapshot_id === NOROW) throw new Error(`Failed id`)

    const testRecord = "testRecord";
    const atom1 = await db.exec({sql: `
        INSERT INTO atoms (kind, args, checksum, data)
        VALUES (?, ?, ?, ?) RETURNING id;
      `,
      bind: [
        testRecord,
        new Args({testId: "1"}).toString(),
        "tr1",
        await encodeDocumentForDB({id: 1, name: "test record 1"})
      ],
      returnValue: "resultRows"
    });
    const atom1_id = oneInOne(atom1);
    if (atom1_id === NOROW) throw new Error(`Failed id`)

    await db.exec({sql: `
        INSERT INTO snapshots_mtm_atoms (snapshot_id, atom_id)
        VALUES (?, ?);
      `,
      bind: [snapshot_id, atom1_id],
    });

    const atom2 = await db.exec({sql: `
        INSERT INTO atoms (kind, args, checksum, data)
        VALUES (?, ?, ?, ?) RETURNING id;
      `,
      bind: [
        testRecord,
        new Args({testId: "2"}).toString(),
        "tr2",
        await encodeDocumentForDB({id: 2, name: "test record 2"})
      ],
      returnValue: "resultRows"
    });
    const atom2_id = oneInOne(atom2);
    if (atom2_id === NOROW) throw new Error(`Failed id`)

    await db.exec({sql: `
        INSERT INTO snapshots_mtm_atoms (snapshot_id, atom_id)
        VALUES (?, ?);
      `,
      bind: [snapshot_id, atom2_id],
    });

    const testList = "testList"
    const atom3 = await db.exec({sql: `
        INSERT INTO atoms (kind, args, checksum, data)
        VALUES (?, ?, ?, ?) RETURNING id;
      `,
      bind: [
        testList,
        new Args({}).toString(),
        "tl1",
        await encodeDocumentForDB({ list: [
          `${testRecord}:1:tr1`,
          `${testRecord}:2:tr2`,
        ]})
      ],
      returnValue: "resultRows"
    });
    const atom3_id = oneInOne(atom3);
    if (atom3_id === NOROW) throw new Error(`Failed id`)

    await db.exec({sql: `
        INSERT INTO snapshots_mtm_atoms (snapshot_id, atom_id)
        VALUES (?, ?);
      `,
      bind: [snapshot_id, atom3_id],
    });
    log("~~ FIXTURE COMPLETED ~~")

    /**
     * OK, the above code gives us 3 atoms that represent a list and two items within it
     * all hooked up to the snapshot and changeset tables
     * 
     * Let's craft expected payloads over the web socket wire, and only call handle event
     * and assert we have the rows we expect to have!
     * 
     * First payload is changing the name of a view
     */
    const payload1: AtomMessage = {
      meta: {
        workspaceId: "W",
        changeSetId: "new_change_set",
        snapshotFromChecksum: "HEAD",
        snapshotToChecksum: "new_change_set",
      },
      atoms: [{
        kind: testRecord,
        kindFromChecksum: "tr1",
        kindToChecksum: "tr1-new-name",
        operations: JSON.stringify([{op: "replace", path: "/name", value: "new name"}]),
        args: {testId: "1"},
      }],
    };
    const event1 = { data: JSON.stringify(payload1) } as MessageEvent;
    await tracer.startActiveSpan("handleEvent", async (span) => {
      await handleEvent(event1, span);
      span.end()
    });

    const confirm1 = await db.exec({
      sql: `SELECT count(snapshot_id) FROM snapshots_mtm_atoms WHERE snapshot_id = ?;`,
      bind: [snapshot_id],
      returnValue: "resultRows",
    });
    const count_old_snapshot_atoms = oneInOne(confirm1);
    // one for each original atom
    console.assert(count_old_snapshot_atoms === 3, `old snapshots ${String(count_old_snapshot_atoms)} === 3`);

    const confirm2 = await db.exec({
      sql: `SELECT count(snapshot_id) FROM snapshots_mtm_atoms WHERE snapshot_id != ?;`,
      bind: [snapshot_id],
      returnValue: "resultRows",
    });
    const count_new_snapshot_atoms = oneInOne(confirm2);
    // copied mtm & the patched atom
    console.assert(count_new_snapshot_atoms === 3, `new snapshots ${String(count_new_snapshot_atoms)} === 1`);

    const confirm3 = await db.exec({
      sql: `SELECT count(rowid) FROM atoms;`,
      returnValue: "resultRows",
    });
    const count_atoms = oneInOne(confirm3);
    // three original atoms, plus the new patched atom
    console.assert(count_atoms === 4, `atoms ${String(count_atoms)} === 4`);

    const new_atom_data = await db.exec({
      sql: `SELECT data FROM atoms WHERE checksum = ?`,
      bind: ["tr1-new-name"],
      returnValue: "resultRows",
    });
    const data = oneInOne(new_atom_data);
    if (data === NOROW) throw new Error("Expected data, got nothing")
    const doc = decodeDocumentFromDB(data as ArrayBuffer);
    console.assert(doc.id === 1 && doc.name === "new name", `Document doesn't match (${JSON.stringify(doc)})`);

    log("~~ FIRST PAYLOAD SUCCESS ~~")

    /**
     * Second payload is merging that change to HEAD
     */
    const payload2: AtomMessage = {
      meta: {
        workspaceId: "W",
        changeSetId: "HEAD",
        snapshotFromChecksum: "HEAD",
        snapshotToChecksum: "new_change_set_on_head",  // will this be different?? if not, my UNIQUE on checksum is bad
      },
      atoms: [{
        kind: testRecord,
        kindFromChecksum: "tr1",
        kindToChecksum: "tr1-new-name",
        operations: JSON.stringify([{op: "replace", path: "/name", value: "new name"}]),
        args: {testId: "1"},
      }]
    };
    const event2 = { data: JSON.stringify(payload2) } as MessageEvent;
    await tracer.startActiveSpan("handleEvent", async (span) => {
      await handleEvent(event2);
      span.end()
    });

    const confirm4 = await db.exec({
      sql: `SELECT count(snapshot_id) FROM snapshots_mtm_atoms WHERE snapshot_id = ?;`,
      bind: [snapshot_id],
      returnValue: "resultRows",
    });
    const count_old_head_snapshot_atoms = oneInOne(confirm4);
    // one for each original atom
    console.assert(count_old_head_snapshot_atoms === NOROW, `old head snapshots ${String(count_old_head_snapshot_atoms)} === 0`);

    const confirm5 = await db.exec({
      sql: `SELECT count(snapshot_id) FROM snapshots_mtm_atoms WHERE snapshot_id != ?;`,
      bind: [snapshot_id],
      returnValue: "resultRows",
    });
    const count_new_snapshot_atoms_again = oneInOne(confirm5);
    // copied mtm & the patched atom, 3 for the changeset, 3 for HEAD
    console.assert(count_new_snapshot_atoms_again === 3*2, `new snapshots ${String(count_new_snapshot_atoms_again)} === 3*2`);

    const confirm6 = await db.exec({
      sql: `SELECT count(rowid) FROM atoms;`,
      returnValue: "resultRows",
    });
    const count_atoms_no_change = oneInOne(confirm6);
    // same number of atoms no change
    console.assert(count_atoms_no_change === 4, `atoms ${String(count_atoms_no_change)} === 4`);

    log("~~ SECOND PAYLOAD SUCCESS ~~")

    /**
     * Third thing that happens, closing out that changeSet
     * WE NEED AN EVENT TO TELL US THIS
     */
    await pruneAtomsForClosedChangeSet("W", "new_change_set");
    const confirm7 = await db.exec({
      sql: `SELECT count(snapshot_id) FROM snapshots_mtm_atoms WHERE snapshot_id != ?;`,
      bind: [snapshot_id],
      returnValue: "resultRows",
    });
    const count_snapshots_after_purge = oneInOne(confirm7);
    // 3 for HEAD
    console.assert(count_snapshots_after_purge === 3, `new snapshots ${String(count_snapshots_after_purge)} === 3`);

    const confirm8 = await db.exec({
      sql: `SELECT count(rowid) FROM atoms;`,
      returnValue: "resultRows",
    });
    const count_atoms_after_purge = oneInOne(confirm8);
    // back to 3 atoms, like original
    console.assert(count_atoms_after_purge === 3, `atoms ${String(count_atoms_after_purge)} === 3`);

    log("~~ PURGE SUCCESS ~~")

    /**
     * Fourth thing that happens, add a new view, remove an existing view
     * TODO
     */

    const payload3: AtomMessage = {
      meta: {
        workspaceId: "W",
        changeSetId: "add_remove",
        snapshotFromChecksum: "new_change_set_on_head",
        snapshotToChecksum: "add_remove_1",
      },
      atoms: [
        {
          kind: testRecord,
          kindFromChecksum: "0",
          kindToChecksum: "tr3-add",
          operations: JSON.stringify([{op: "add", path: "/name", value: "record 3"}, {op: "add", path: "/id", value: 3}]),
          args: {testId: "3"},
        },
        {
          kind: testRecord,
          kindFromChecksum: "tr1-new-name",
          kindToChecksum: "0",
          operations: JSON.stringify([]),
          args: {testId: "1"},
        },
        {
          kind: testList,
          kindFromChecksum: "tl1",
          kindToChecksum: "tl1-add-remove",
          operations: JSON.stringify([{op: "remove", path: "/list/0"}, {op: "add", path:"/list/2", value: `${testRecord}:3:tr3-add`}]),
          args: {},
        },
      ]
    };
    const event3 = { data: JSON.stringify(payload3) } as MessageEvent;
    await tracer.startActiveSpan("handleEvent", async (span) => {
      await handleEvent(event3);
      span.end()
    });

    const added_record = await db.exec({
      sql: `SELECT data FROM atoms WHERE checksum = ?`,
      bind: ["tr3-add"],
      returnValue: "resultRows",
    });
    const added = oneInOne(added_record);
    if (added === NOROW) throw new Error("Expected new record, got nothing")
    const added_doc = decodeDocumentFromDB(added as ArrayBuffer);
    console.assert(added_doc.id === 3 && added_doc.name === "record 3", `Added document doesn't match (${JSON.stringify(added_doc)})`);

    const removed_record = await db.exec({
      sql: `SELECT data FROM atoms WHERE checksum = ?`,
      bind: ["tr1-new-name"],
      returnValue: "resultRows",
    });
    const removed = oneInOne(removed_record);
    console.assert(removed === NOROW, "Expected removed record gone")

    const modlist = await db.exec({
      sql: `SELECT data FROM atoms WHERE checksum = ?`,
      bind: ["tl1-add-remove"],
      returnValue: "resultRows",
    });
    const list = oneInOne(modlist);
    if (list === NOROW) throw new Error("Expected list, got nothing")
    const list_doc = decodeDocumentFromDB(list as ArrayBuffer);
    console.assert(list_doc.list[0] === `${testRecord}:2:tr2`, `List item 1 is wrong (${JSON.stringify(list_doc)})`);
    console.assert(list_doc.list[1] === `${testRecord}:3:tr3-add`, `List item 2 is wrong (${JSON.stringify(list_doc)})`);

    log("~~ ADD / REMOVE COMPLETED ~~")

    log("~~ DIAGNOSTIC COMPLETED ~~")
  }
};


Comlink.expose(dbInterface);