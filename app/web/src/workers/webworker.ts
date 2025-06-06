import * as Comlink from "comlink";
import { applyPatch as applyOperations } from "fast-json-patch";
import sqlite3InitModule, {
  Database,
  ExecBaseOptions,
  ExecReturnResultRowsOptions,
  ExecRowModeArrayOptions,
  FlexibleString,
  Sqlite3Static,
  SqlValue,
} from "@sqlite.org/sqlite-wasm";
import ReconnectingWebSocket from "reconnecting-websocket";
import { trace, Span } from "@opentelemetry/api";
import { WebTracerProvider } from "@opentelemetry/sdk-trace-web";
import {
  BatchSpanProcessor,
  // ConsoleSpanExporter,
} from "@opentelemetry/sdk-trace-base";
import { FetchInstrumentation } from "@opentelemetry/instrumentation-fetch";
import { registerInstrumentations } from "@opentelemetry/instrumentation";
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-http";
import { Resource } from "@opentelemetry/resources";
import {
  ATTR_SERVICE_NAME,
  ATTR_SERVICE_VERSION,
} from "@opentelemetry/semantic-conventions";
import { URLPattern, describePattern } from "@si/vue-lib";
import Axios, {
  AxiosInstance,
  AxiosResponse,
  InternalAxiosRequestConfig,
} from "axios";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { nonNullable } from "@/utils/typescriptLinter";
import { DefaultMap } from "@/utils/defaultmap";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  DBInterface,
  NOROW,
  Checksum,
  Atom,
  QueryKey,
  Id,
  PatchBatch,
  AtomMeta,
  AtomDocument,
  AtomMessage,
  MessageKind,
  IndexObjectMeta,
  BustCacheFn,
  Ragnarok,
  RainbowFn,
  IndexUpdate,
  LobbyExitFn,
  MjolnirBulk,
} from "./types/dbinterface";
import {
  BifrostViewList,
  RawViewList,
  View,
  EddaComponentList,
  EddaComponent,
  BifrostComponentList,
  EddaIncomingConnectionsList,
  BifrostIncomingConnectionsList,
  EddaIncomingConnections,
  BifrostComponentConnections,
  BifrostConnection,
  EddaConnection,
  BifrostComponent,
  SchemaVariant,
  PossibleConnection,
  EntityKind,
  EddaSchemaVariantCategories,
  BifrostSchemaVariantCategories,
  CategoryVariant,
  SchemaMembers,
} from "./types/entity_kind_types";
import {
  hasReturned,
  maybeMjolnir,
  bulkDone,
  bulkInflight,
  processPatchQueue,
  processMjolnirQueue,
  bustQueueAdd,
} from "./mjolnir_queue";

let otelEndpoint = import.meta.env.VITE_OTEL_EXPORTER_OTLP_ENDPOINT;
if (!otelEndpoint) otelEndpoint = "http://localhost:8080";
const exporter = new OTLPTraceExporter({
  url: `${otelEndpoint}/v1/traces`,
});
// const consoleExporter = new ConsoleSpanExporter();
const processor = new BatchSpanProcessor(exporter);

const provider = new WebTracerProvider({
  resource: new Resource({
    [ATTR_SERVICE_NAME]: "bifrost",
    [ATTR_SERVICE_VERSION]: "0.1",
  }),
  spanProcessors: [processor],
});

provider.register();

registerInstrumentations({
  instrumentations: [new FetchInstrumentation()],
});

const tracer = trace.getTracer("bifrost");
// eslint-disable-next-line no-console
const log = console.log;
// eslint-disable-next-line no-console
const error = console.error;
const _DEBUG = true; // import.meta.env.VITE_SI_ENV === "local";
const _START_FRESH = false;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function debug(...args: any | any[]) {
  // eslint-disable-next-line no-console
  if (_DEBUG) console.debug(args);
}

/**
 *
 *  INITIALIZATION FNS
 *
 */
let db: Database;
let sdf: AxiosInstance;

const getDbName = (testing: boolean) => {
  if (testing) return "sitest.sqlite3";
  switch (import.meta.env.VITE_SI_ENV) {
    case "production":
      return "si.sqlite3";
    case "staging":
      return "si.staging.sqlite3";
    default:
      return "si.local.sqlite3";
  }
};

const start = async (sqlite3: Sqlite3Static, testing: boolean) => {
  const dbname = getDbName(testing);
  db =
    "opfs" in sqlite3
      ? new sqlite3.oo1.OpfsDb(`/${dbname}`)
      : new sqlite3.oo1.DB(`/${dbname}`, "c");
  debug(
    "opfs" in sqlite3
      ? `OPFS is available, created persisted database at ${db.filename}`
      : `OPFS is not available, created transient database ${db.filename}`,
  );
  db.exec({ sql: "PRAGMA foreign_keys = ON;" });
};

const initializeSQLite = async (testing: boolean) => {
  try {
    const sqlite3 = await sqlite3InitModule({ print: log, printErr: error });
    await start(sqlite3, testing);
  } catch (err) {
    if (err instanceof Error)
      error("Initialization error:", err.name, err.message);
    else error("Initialization error:", err);
  }
};

const dropTables = async () => {
  const sql = `
  DROP TABLE IF EXISTS index_mtm_atoms;
  DROP TABLE IF EXISTS atoms;
  DROP TABLE IF EXISTS indexes;
  DROP TABLE IF EXISTS changesets;
  DROP TABLE IF EXISTS weak_references;
  `;
  db.exec({ sql });
};

// INTEGER is 8 bytes, not large enough to store ULIDs
// we'll go with string, though reading that putting the bytes as BLOBs would save space
const ensureTables = async (testing: boolean) => {
  if (_START_FRESH || testing) await dropTables();
  /**
   * GOAL: persist only data that is readable, once blob data is no longer viewable, get rid of it
   * PROBLEM: Objects exist across multiple changesets, so we cannot ever UPDATE atom
   * SOLUTION: We copy objects when we are given mutations
   * PROBLEM: We don't want to read every single blob and check internal references
   * SOLUTION: Use index checksums and FK index_mtm relationships to delete
   */
  const sql = `
  CREATE TABLE IF NOT EXISTS changesets (
    change_set_id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    index_checksum TEXT NOT NULL,
    FOREIGN KEY (index_checksum) REFERENCES indexes(checksum) ON DELETE CASCADE
  ) WITHOUT ROWID;
  CREATE INDEX IF NOT EXISTS changeset_workspace_id ON changesets(workspace_id);

  CREATE TABLE IF NOT EXISTS indexes (
    checksum TEXT PRIMARY KEY
  ) WITHOUT ROWID;

  CREATE TABLE IF NOT EXISTS atoms (
    kind TEXT NOT NULL,
    args TEXT NOT NULL,
    checksum TEXT NOT NULL,
    data BLOB,
    PRIMARY KEY (kind, args, checksum)
  ) WITHOUT ROWID;

  CREATE TABLE IF NOT EXISTS index_mtm_atoms (
    index_checksum TEXT NOT NULL,
    kind TEXT NOT NULL,
    args TEXT NOT NULL,
    checksum TEXT NOT NULL,
    PRIMARY KEY (index_checksum, kind, args, checksum),
    FOREIGN KEY (index_checksum) REFERENCES indexes(checksum) ON DELETE CASCADE,
    FOREIGN KEY (kind, args, checksum) REFERENCES atoms(kind, args, checksum) ON DELETE CASCADE,
    CONSTRAINT uniqueness UNIQUE (index_checksum, kind, args) ON CONFLICT REPLACE
  ) WITHOUT ROWID;

  CREATE TABLE IF NOT EXISTS weak_references (
    change_set_id TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_args TEXT NOT NULL,
    referrer_kind TEXT NOT NULL,
    referrer_args TEXT NOT NULL,
    PRIMARY KEY (change_set_id, target_kind, target_args, referrer_kind, referrer_args)
  ) WITHOUT ROWID;
  `;
  /**
   * RULES:
   * When an Atom is deleted, delete its MTM entry (CASCADE should take care of this)
   * When an Index is deleted, delete its MTM entry, but not its atoms (CASCADE should take care of this)
   *
   * When a Changeset is closed/deleted:
   *  - delete atoms connected to its index MTMs (We can not CASCADE atom deletion)
   *  - delete its record, CASCADE should delete its indexes and MTMs
   *
   * PATCH WORKFLOW:
   * When we are given a new index along with patch data:
   *  - INSERT INTO indexes <new_index_checksum>
   *  - INSERT INTO index_mtm_atoms SELECT <new_index_checksum>, kind, args, checksum WHERE index_checksum="<old_index_checksum>" AND change_set_id=<this_changeSetId>
   *  - UPDATE changesets SET index_checksum = <new_index_checksum>
   *  - For each patch data
   *    - fromChecksum = 0, this is net new, insert atom
   *    - toChecksum = 0, this is a deletion, remove atom
   *    - nonzero checksums:
   *      - select * from atoms where kind=<kind>, args=<args>, checksum=<old_checksum>
   *        - if data doesn't exist throw mjolnir
   *      - apply patch data
   *      - atom_id = insert into atoms data=<blob>, kind=<kind>, args=<args>, checksum=<new_checksum>
   *      - insert into index_mtm_atoms atom_id = atom_id, index_checksum = <new_index_checksum>
   *  - DELETE FROM indexes WHERE change_set_id=<this_changeSetId> AND checksum=<old_index_checksum>
   */

  return db.exec({ sql });
};

// NOTE: this is just for external test usage, do not use this within this file
const exec = (
  opts: ExecBaseOptions &
    ExecRowModeArrayOptions &
    ExecReturnResultRowsOptions & {
      sql: FlexibleString;
    },
): SqlValue[][] => db.exec(opts);

/**
 * A few small utilities
 */
const encodeDocumentForDB = async (doc: object) => {
  return await new Blob([JSON.stringify(doc)]).arrayBuffer();
};

const decodeDocumentFromDB = (doc: ArrayBuffer): AtomDocument => {
  const s = new TextDecoder().decode(doc);
  const j = JSON.parse(s);
  return j;
};

// When you just expect one column and one row
const oneInOne = (rows: SqlValue[][]): SqlValue | typeof NOROW => {
  const first = rows[0];
  if (first) {
    const id = first[0];
    if (id || id === 0) return id;
  }
  return NOROW;
};

/**
 *
 * INDEX LOGIC
 *
 */

const atomExistsOnIndexes = async (
  kind: EntityKind,
  id: string,
  checksum: Checksum,
): Promise<Checksum[]> => {
  const rows = db.exec({
    sql: `
    select
     index_checksum
    from index_mtm_atoms
    where
      kind=? and
      args=? and
      checksum = ?
    ;
    `,
    bind: [kind, id, checksum],
    returnValue: "resultRows",
  });
  return rows.flat().filter(nonNullable) as Checksum[];
};

const newIndex = async (
  meta: AtomMeta,
  fromIndexChecksum: string | undefined,
) => {
  db.exec({
    sql: `INSERT INTO indexes (checksum) VALUES (?);`,
    bind: [meta.toIndexChecksum],
  });

  const rows = db.exec({
    sql: `SELECT index_checksum FROM changesets WHERE change_set_id = ?`,
    bind: [meta.changeSetId],
    returnValue: "resultRows",
  });
  const lastKnownFromChecksum = oneInOne(rows) as
    | string
    | undefined
    | typeof NOROW;

  if (fromIndexChecksum && fromIndexChecksum !== meta.toIndexChecksum) {
    db.exec({
      sql: `INSERT INTO index_mtm_atoms
        SELECT
          ?, kind, args, checksum
        FROM index_mtm_atoms
        WHERE
          index_checksum = ?
        `,
      bind: [meta.toIndexChecksum, fromIndexChecksum],
    });
  } else if (lastKnownFromChecksum && lastKnownFromChecksum !== NOROW) {
    debug(`HIT ELSE BRANCH NEW FROM CHECKSUM SHIT`);
    db.exec({
      sql: `INSERT INTO index_mtm_atoms
        SELECT
          ?, kind, args, checksum
        FROM index_mtm_atoms
        WHERE
          index_checksum = ?
        `,
      bind: [meta.toIndexChecksum, lastKnownFromChecksum],
    });
  } else {
    // we have a new change set and a patch at the same time
    // which means that the change set record did not exist, no from in the DB
    // but we have the from in the payload
    db.exec({
      sql: `INSERT INTO index_mtm_atoms
        SELECT
          ?, kind, args, checksum
        FROM index_mtm_atoms
        WHERE
          index_checksum = ?
        `,
      bind: [meta.toIndexChecksum, meta.fromIndexChecksum],
    });
  }
};

const removeAtom = async (indexChecksum: Checksum, atom: Required<Atom>) => {
  db.exec({
    sql: `
    DELETE FROM index_mtm_atoms
    WHERE index_checksum = ? AND kind = ? AND args = ? AND checksum = ?
    `,
    bind: [indexChecksum, atom.kind, atom.id, atom.fromChecksum],
  });
};

const createAtomFromPatch = async (atom: Atom, span?: Span) => {
  const doc = {};
  let afterDoc = {};
  if (atom.operations) {
    const applied = applyOperations(doc, atom.operations);
    afterDoc = applied.newDocument;
  }
  await createAtom(atom, afterDoc, span);
  return afterDoc;
};

const createAtom = async (atom: Atom, doc: object, _span?: Span) => {
  debug("createAtom", atom, doc);

  const encodedDoc = await encodeDocumentForDB(doc);
  try {
    db.exec({
      sql: `insert into atoms
        (kind, checksum, args, data)
          VALUES
        (?, ?, ?, ?)
        ON CONFLICT (kind, checksum, args)
        DO UPDATE SET data=excluded.data
      ;`,
      bind: [atom.kind, atom.toChecksum, atom.id, encodedDoc],
    });

    debug("✅ createAtom successful:", atom.kind, atom.id, atom.toChecksum);
  } catch (err) {
    error("createAtom failed", atom, doc, err);
  }
};

const partialKeyFromKindAndArgs = (kind: EntityKind, id: Id): QueryKey => {
  return `${kind}|${id}`;
};

const kindAndArgsFromKey = (key: QueryKey): { kind: EntityKind; id: Id } => {
  const pieces = key.split("|", 2);
  if (pieces.length !== 2) throw new Error(`Bad key ${key} -> ${pieces}`);
  if (!pieces[0] || !pieces[1])
    throw new Error(`Missing key ${key} -> ${pieces}`);
  const kind = pieces[0] as EntityKind;
  const id = pieces[1];
  return { kind, id };
};

const bustOrQueue = (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
  skipQueue = false,
) => {
  if (skipQueue) bustCacheFn(workspaceId, changeSetId, kind, id);
  else bustQueueAdd(workspaceId, changeSetId, kind, id, bustCacheFn);
};

const bustCacheAndReferences = async (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
  skipQueue = false,
) => {
  // bust me
  bustOrQueue(workspaceId, changeSetId, kind, id, skipQueue);

  // bust everyone who refers to me
  const sql = `
    select referrer_kind, referrer_args from weak_references where target_kind = ? and target_args = ? and change_set_id = ?;
  `;
  const bind = [kind, id, changeSetId];
  const refs = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  refs.forEach(([ref_kind, ref_id]) => {
    if (ref_kind && ref_id)
      bustOrQueue(
        workspaceId,
        changeSetId,
        ref_kind as EntityKind,
        ref_id as string,
        skipQueue,
      );
  });
};

const handleHammer = async (msg: AtomMessage, span?: Span) => {
  debug(
    "🔨 HAMMER RECEIVED:",
    msg.atom.kind,
    msg.atom.id,
    "toChecksum:",
    msg.atom.toChecksum,
  );

  // Log index checksum for validation context
  if (msg.atom.toChecksum) {
    span?.setAttribute("indexChecksum", msg.atom.toIndexChecksum);
    debug("🔨 handling hammer with index checksum", msg.atom.toIndexChecksum);
  }

  // in between throwing a hammer and receiving it, i might already have written the atom
  const indexes = await atomExistsOnIndexes(
    msg.atom.kind,
    msg.atom.id,
    msg.atom.toChecksum,
  );
  if (indexes.length > 0) {
    if (indexes.includes(msg.atom.toIndexChecksum)) {
      debug(
        "🔨 HAMMER NOOP: Atom already exists in index:",
        msg.atom.kind,
        msg.atom.id,
        msg.atom.toChecksum,
        indexes,
      );
      return; // noop
    } else {
      debug("HAMMER: Atom exists, MTM needed");
      await insertAtomMTM(msg.atom, msg.atom.toIndexChecksum);
      return;
    }
  }

  const indexChecksum = await indexLogic(msg.atom, span);

  // if the atom exists, i just need the MTM
  if (indexes.length === 0) {
    debug(
      "🔨 HAMMER: Creating new atom:",
      msg.atom.kind,
      msg.atom.id,
      "checksum:",
      msg.atom.toChecksum,
    );
    await createAtom(msg.atom, msg.data, span);
    debug(
      "🔨 HAMMER: Atom created successfully:",
      msg.atom.kind,
      msg.atom.id,
      "checksum:",
      msg.atom.toChecksum,
    );
  } else {
    debug(
      "🔨 HAMMER: Atom exists, just need MTM:",
      msg.atom.kind,
      msg.atom.id,
      "existing indexes:",
      indexes,
    );
  }

  if (!indexChecksum)
    throw new Error(`Expected index checksum for ${msg.atom.toIndexChecksum}`);

  debug(
    "🔨 HAMMER: Inserting MTM for:",
    msg.atom.kind,
    msg.atom.id,
    "checksum:",
    msg.atom.toChecksum,
    "index:",
    indexChecksum,
  );
  await insertAtomMTM(msg.atom, indexChecksum);

  await updateChangeSetWithNewIndex(msg.atom);
  await removeOldIndex();

  if (COMPUTED_KINDS.includes(msg.atom.kind)) {
    debug("🔨 HAMMER: Updating computed for:", msg.atom.kind, msg.atom.id);
    await updateComputed(
      msg.atom.workspaceId,
      msg.atom.changeSetId,
      msg.atom.kind,
      msg.data,
      indexChecksum,
    );
  }

  debug(
    "🔨 HAMMER: Busting cache for:",
    msg.atom.kind,
    msg.atom.id,
    "checksum:",
    msg.atom.toChecksum,
  );
  await bustCacheAndReferences(
    msg.atom.workspaceId,
    msg.atom.changeSetId,
    msg.atom.kind,
    msg.atom.id,
  );
};

const insertAtomMTM = async (atom: Atom, indexChecksum: Checksum) => {
  try {
    const bind = [indexChecksum, atom.kind, atom.id, atom.toChecksum];
    const exists = db.exec({
      sql: `select index_checksum, kind, args, checksum from index_mtm_atoms
        where index_checksum = ? and kind = ? and args = ? and checksum = ?
      ;`,
      bind,
      returnValue: "resultRows",
    });
    if (exists.length > 0) {
      return false; // no-op
    }

    db.exec({
      sql: `insert into index_mtm_atoms
        (index_checksum, kind, args, checksum)
          VALUES
        (?, ?, ?, ?)
      ;`,
      bind,
    });
  } catch (err) {
    // should be resolved with the previous SELECT
    // even with the unique constraint ON CONFLICT REPLACE
    // if the checksum is identical, it will error
    error("createMTM failed", atom);
  }
  return true;
};

const indexLogic = async (meta: AtomMeta, span?: Span) => {
  const { changeSetId, workspaceId, toIndexChecksum } = {
    ...meta,
  };

  span?.setAttributes({
    changeSetId,
    workspaceId,
    toIndexChecksum,
  });

  const changeSetQuery = db.exec({
    sql: `select change_set_id, index_checksum from changesets where change_set_id = ?`,
    returnValue: "resultRows",
    bind: [meta.changeSetId],
  });
  let changeSetExists;
  let currentIndexChecksum;
  const changeSet = changeSetQuery[0] as string[];
  if (changeSet) {
    [changeSetExists, currentIndexChecksum] = [...changeSet];
  }

  const indexQuery = db.exec({
    sql: `select checksum from indexes where checksum = ?`,
    returnValue: "resultRows",
    bind: [toIndexChecksum],
  });
  const indexExists = oneInOne(indexQuery);

  if (changeSetExists && !currentIndexChecksum)
    throw new Error("Null value from SQL, impossible");

  if (
    changeSetExists &&
    meta.fromIndexChecksum &&
    meta.fromIndexChecksum !== currentIndexChecksum
  ) {
    debug("🔥🔥 RAGNAROK", meta.fromIndexChecksum, currentIndexChecksum);
    // throw new Ragnarok(
    //   "From Checksum Doesn't Exist",
    //   workspaceId,
    //   changeSetId,
    //   meta.fromIndexChecksum,
    //   currentIndexChecksum,
    // );
  }

  // Create index if needed - this is the new validation mechanism
  if (indexExists === NOROW) await newIndex(meta, currentIndexChecksum);

  if (!changeSetExists) {
    db.exec({
      sql: "insert into changesets (change_set_id, workspace_id, index_checksum) VALUES (?, ?, ?);",
      bind: [meta.changeSetId, meta.workspaceId, toIndexChecksum],
    });
  }

  // Index checksum provides validation - every time MVs are generated, there's a new index checksum
  debug("✓ Index checksum validation passed", toIndexChecksum);

  return toIndexChecksum;
};

const handlePatchMessage = async (data: PatchBatch, span?: Span) => {
  const batchId = `${data.meta.toIndexChecksum}-${data.patches.length}`;
  debug("📦 BATCH START:", batchId);

  span?.setAttribute("numRawPatches", data.patches.length);
  if (data.patches.length === 0) return;
  // Assumption: every patch is working on the same workspace and changeset
  // (e.g. we're not bundling messages across workspaces somehow)

  if (!data.meta.changeSetId) throw new Error("Expected changeSetId");
  if (!data.meta.toIndexChecksum) throw new Error("Expected indexChecksum");

  // Log index checksum for tracing - this provides validation at the index level
  span?.setAttribute("indexChecksum", data.meta.toIndexChecksum);
  debug("📦 Processing patches with index checksum", data.meta.toIndexChecksum);
  debug(
    "📦 Patch details:",
    data.patches.map(
      (p, i) =>
        `[${i}] ${p.kind}.${p.id}: ${p.fromChecksum} -> ${p.toChecksum}`,
    ),
  );

  // Check for duplicate patches in the same batch
  const patchKeys = data.patches.map(
    (p) => `${p.kind}.${p.id}.${p.fromChecksum}.${p.toChecksum}`,
  );
  const uniquePatchKeys = new Set(patchKeys);
  if (patchKeys.length !== uniquePatchKeys.size) {
    debug("📦 WARNING: Duplicate patches detected in batch!", {
      total: patchKeys.length,
      unique: uniquePatchKeys.size,
      duplicates: patchKeys.filter(
        (key, index) => patchKeys.indexOf(key) !== index,
      ),
    });
  }

  let indexChecksum: string;
  try {
    indexChecksum = await indexLogic(data.meta, span);
    debug("📦 Index logic completed, resolved checksum:", indexChecksum);
  } catch (err) {
    if (err instanceof Ragnarok) {
      span?.addEvent("ragnarok", {
        patchBatch: JSON.stringify(data),
        fromChecksumExpected: err.fromChecksumExpected,
        currentChecksum: err.currentChecksum,
      });
      await ragnarok(err.workspaceId, err.changeSetId);
      return;
    } else {
      throw err;
    }
  }

  /**
   * Patches are not coming over the wire in any meaningful
   * order, which means they can be inter-dependent e.g. an item in
   * a list can be _after_ the list that wants it.
   * This causes an unnecessary hammer by the list when it doesn't have
   * the item.
   *
   * We can at least do anything with "list" *after* everything else
   * Its the 20% that gets us 80% until patches can be ordered by
   * graph dependency.
   */
  const atoms = data.patches
    .map((rawAtom) => {
      const atom: Atom = {
        ...rawAtom,
        ...data.meta,
        operations: rawAtom.patch,
      };
      return atom;
    })
    .filter((rawAtom): rawAtom is Required<Atom> => !!rawAtom.fromChecksum);

  span?.setAttribute("numAtoms", atoms.length);
  if (!indexChecksum)
    throw new Error(`Expected index checksum for ${data.meta.toIndexChecksum}`);

  // non-list atoms
  // non-connections (e.g. components need to go before connections)
  const nonListAtoms = atoms.filter(
    (a) => !a.kind.includes("List") && !a.kind.includes("IncomingConnection"),
  );
  debug(
    "📦 Processing non-list atoms:",
    nonListAtoms.length,
    nonListAtoms.map(
      (a) => `${a.kind}.${a.id}: ${a.fromChecksum} -> ${a.toChecksum}`,
    ),
  );
  const atomsToBust = await Promise.all(
    nonListAtoms.map(async (atom) => {
      return applyPatch(atom, indexChecksum);
    }),
  );

  // connections (but NOT lists - avoid double processing IncomingConnectionsList)
  const connectionAtoms = atoms.filter(
    (a) => a.kind.includes("IncomingConnection") && !a.kind.includes("List"),
  );
  debug(
    "📦 Processing connection atoms:",
    connectionAtoms.length,
    connectionAtoms.map(
      (a) => `${a.kind}.${a.id}: ${a.fromChecksum} -> ${a.toChecksum}`,
    ),
  );
  const connAtomsToBust = await Promise.all(
    connectionAtoms.map(async (atom) => {
      return await applyPatch(atom, indexChecksum);
    }),
  );

  // list items (all lists, including IncomingConnectionsList)
  const listAtoms = atoms.filter((a) => a.kind.includes("List"));
  debug(
    "📦 Processing list atoms:",
    listAtoms.length,
    listAtoms.map(
      (a) => `${a.kind}.${a.id}: ${a.fromChecksum} -> ${a.toChecksum}`,
    ),
  );
  const listAtomsToBust = await Promise.all(
    listAtoms.map(async (atom) => {
      return applyPatch(atom, indexChecksum);
    }),
  );

  await updateChangeSetWithNewIndex(data.meta);
  await removeOldIndex();

  debug(
    "🧹 Busting cache for atoms:",
    atomsToBust.length + connAtomsToBust.length + listAtomsToBust.length,
  );

  atomsToBust.forEach((atom) => {
    if (atom) {
      debug("🧹 Busting cache for atom:", atom.kind, atom.id);
      bustCacheAndReferences(
        atom.workspaceId,
        atom.changeSetId,
        atom.kind,
        atom.id,
      );
    }
  });
  connAtomsToBust.forEach((atom) => {
    if (atom) {
      debug("🧹 Busting cache for connection:", atom.kind, atom.id);
      bustCacheAndReferences(
        atom.workspaceId,
        atom.changeSetId,
        atom.kind,
        atom.id,
      );
    }
  });
  listAtomsToBust.forEach((atom) => {
    if (atom) {
      debug("🧹 Busting cache for list:", atom.kind, atom.id);
      bustCacheAndReferences(
        atom.workspaceId,
        atom.changeSetId,
        atom.kind,
        atom.id,
      );
    }
  });

  debug("📦 BATCH COMPLETE:", batchId);
};

const applyPatch = async (atom: Required<Atom>, indexChecksum: Checksum) => {
  return await tracer.startActiveSpan("applyPatch", async (span) => {
    span.setAttribute("atom", JSON.stringify(atom));
    debug(
      "🔧 Applying patch:",
      atom.kind,
      atom.id,
      `${atom.fromChecksum} -> ${atom.toChecksum}`,
    );

    // Check if we actually have the atom data, not just the MTM relationship
    const upToDateAtomIndexes = await atomExistsOnIndexes(
      atom.kind,
      atom.id,
      atom.toChecksum,
    );
    if (upToDateAtomIndexes.length > 0) {
      debug(
        "🔧 No Op!",
        atom.kind,
        atom.id,
        atom.toChecksum,
        upToDateAtomIndexes,
      );
      span.addEvent("noop", {
        upToDateAtomIndexes: JSON.stringify(upToDateAtomIndexes),
      });
      span.end();
      return;
    }

    // do we have an index with the fromChecksum (without we cannot patch)
    const previousIndexes = await atomExistsOnIndexes(
      atom.kind,
      atom.id,
      atom.fromChecksum,
    );
    span.setAttribute("previousIndexes", JSON.stringify(previousIndexes));
    const exists = previousIndexes.length > 0;
    span.setAttribute("exists", exists);
    debug(
      "🔧 Previous indexes exist:",
      exists,
      "fromChecksum:",
      atom.fromChecksum,
    );

    let needToInsertMTM = false;
    let bustCache = false;
    let doc;
    if (atom.fromChecksum === "0") {
      if (!exists) {
        // if i already have it, this is a NOOP
        debug("🔧 Creating new atom from patch:", atom.kind, atom.id);
        span.setAttribute("createAtomFromPatch", true);
        doc = await createAtomFromPatch(atom, span);
        needToInsertMTM = true;
        bustCache = true;
      } else {
        debug("🔧 New atom already exists (noop):", atom.kind, atom.id);
      }
    } else if (atom.toChecksum === "0") {
      // if i've already removed it, this is a NOOP
      if (exists) {
        debug("🔧 Removing atom:", atom.kind, atom.id);
        span.setAttribute("removeAtom", true);
        await removeAtom(indexChecksum, atom);
        bustCache = true;
      } else {
        debug("🔧 Atom already removed (noop):", atom.kind, atom.id);
      }
    } else {
      // patch it if I can
      if (exists) {
        debug("🔧 Patching existing atom:", atom.kind, atom.id);
        span.setAttribute("patchAtom", true);
        doc = await patchAtom(atom);
        needToInsertMTM = true;
        bustCache = true;
      }
      // otherwise, fire the small hammer to get the full object
      else {
        debug(
          "🔨 MJOLNIR RACE: Missing fromChecksum data, firing hammer:",
          atom.kind,
          atom.id,
          "fromChecksum:",
          atom.fromChecksum,
        );
        span.addEvent("mjolnir", {
          atom: JSON.stringify(atom),
          previousIndexes: JSON.stringify(previousIndexes),
          toChecksumIndexes: JSON.stringify([]), // indexes variable was removed
          source: "applyPatch",
        });
        debug("applyPatch mjolnir", atom.kind, atom.id);
        mjolnir(
          atom.workspaceId,
          atom.changeSetId,
          atom.kind,
          atom.id,
          atom.toChecksum,
        );
      }
    }

    // this insert potentially replaces the MTM row that exists for the current index
    // based on the table constraint
    span.setAttribute("needToInsertMTM", needToInsertMTM);
    if (needToInsertMTM) {
      debug(
        "🔧 Inserting MTM for:",
        atom.kind,
        atom.id,
        "indexChecksum:",
        indexChecksum,
      );
      const inserted = await insertAtomMTM(atom, indexChecksum);
      span.setAttribute("insertedMTM", inserted);
      debug("🔧 MTM inserted:", inserted, "for:", atom.kind, atom.id);
    }
    span.end();

    if (doc && COMPUTED_KINDS.includes(atom.kind)) {
      debug("🔧 Updating computed for:", atom.kind, atom.id);
      await updateComputed(
        atom.workspaceId,
        atom.changeSetId,
        atom.kind,
        doc,
        indexChecksum,
      );
    }

    if (bustCache) {
      debug("🔧 Patch successful, will bust cache for:", atom.kind, atom.id);
      return atom;
    }
    debug("🔧 Patch completed (no cache bust needed):", atom.kind, atom.id);
    return undefined;
  });
};

const patchAtom = async (atom: Required<Atom>) => {
  const atomRows = db.exec({
    sql: `SELECT kind, args, checksum, data
      FROM atoms
      WHERE
        kind = ? and
        args = ? and
        checksum = ?
      ;`,
    bind: [atom.kind, atom.id, atom.fromChecksum],
    returnValue: "resultRows",
  });
  if (atomRows.length === 0) throw new Error("Cannot find atom");
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const atomRow = atomRows[0]!;

  const _doc = atomRow[3] as ArrayBuffer;
  const doc = decodeDocumentFromDB(_doc);
  let afterDoc = doc;
  if (atom.operations) {
    const applied = applyOperations(doc, atom.operations);
    afterDoc = applied.newDocument;
  }

  db.exec({
    sql: `
    insert into atoms
      (kind, args, checksum, data)
    values
      (?, ?, ?, ?)
    ON CONFLICT DO NOTHING
    ;`,
    bind: [
      atom.kind,
      atom.id,
      atom.toChecksum,
      await encodeDocumentForDB(afterDoc),
    ],
  });
  return afterDoc;
};

type BulkResponse = { successful: IndexObjectMeta[]; failed: MjolnirBulk[] };
const mjolnirBulk = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  objs: MjolnirBulk,
  indexChecksum: string,
) => {
  debug("🔨 BULK MJOLNIR:", objs.length, objs);

  const pattern = [
    "v2",
    "workspaces",
    { workspaceId },
    "change-sets",
    { changeSetId },
    "index",
    "multi_mjolnir",
  ] as URLPattern;
  const [url, desc] = describePattern(pattern);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let req: undefined | AxiosResponse<BulkResponse, any>;

  objs.forEach((o) => {
    inFlight(changeSetId, `${o.kind}.${o.id}`);
  });

  await tracer.startActiveSpan(`GET ${desc}`, async (span) => {
    span.setAttributes({
      workspaceId,
      changeSetId,
      indexChecksum,
      numHammers: objs.length,
    });
    try {
      req = await sdf<BulkResponse>({
        method: "post",
        url,
        data: { requests: objs },
      });
      debug("🔨 MJOLNIR BULK HTTP SUCCESS:", indexChecksum);
      span.setAttributes({
        successful: req.data.successful.length,
        failed: req.data.failed.length,
      });
    } catch (err) {
      span.setAttribute("http.status", 404);
      debug("🔨 MJOLNIR HTTP 404:", indexChecksum, err);
      error("MJOLNIR 404", url, objs, err);
    } finally {
      if (req?.status) span.setAttribute("http.status", req.status);
      span.end();
    }
  });

  if (!req) {
    debug("🔨 MJOLNIR BULK FAILED:", indexChecksum, "no response");
    bulkDone(true);
    return;
  }

  const first = req.data.successful.shift();
  if (!first) {
    debug("🔨 MJOLNIR BULK NO FIRST?:", req.data.successful.length);
    return;
  }
  const msg: AtomMessage = {
    kind: MessageKind.MJOLNIR,
    atom: {
      id: first.frontEndObject.id,
      kind: first.frontEndObject.kind,
      toChecksum: first.frontEndObject.checksum,
      workspaceId,
      changeSetId,
      toIndexChecksum: first.indexChecksum,
      fromIndexChecksum: first.indexChecksum,
    },
    data: first.frontEndObject.data,
  };
  // doing this first, by itself, await'd, because its going to make the new index, etc
  // and we dont want that to race across multiple patches
  returned(
    changeSetId,
    `${first.frontEndObject.kind}.${first.frontEndObject.id}`,
  );
  await handleHammer(msg);

  await Promise.all(
    req.data.successful.map((obj) => {
      const msg: AtomMessage = {
        kind: MessageKind.MJOLNIR,
        atom: {
          id: obj.frontEndObject.id,
          kind: obj.frontEndObject.kind,
          toChecksum: obj.frontEndObject.checksum,
          workspaceId,
          changeSetId,
          toIndexChecksum: obj.indexChecksum,
          fromIndexChecksum: obj.indexChecksum,
        },
        data: obj.frontEndObject.data,
      };
      returned(
        changeSetId,
        `${obj.frontEndObject.kind}.${obj.frontEndObject.id}`,
      );
      return handleHammer(msg);
    }),
  );
  debug("🔨 MJOLNIR BULK DONE!");
  bulkDone();
};

const mjolnir = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: EntityKind,
  id: Id,
  checksum?: Checksum,
) => {
  const atomKey = `${kind}.${id}`;
  debug("🔨 MJOLNIR REQUESTED:", atomKey, "checksum:", checksum);

  maybeMjolnir({ workspaceId, changeSetId, kind, id }, async () => {
    debug("🔨 MJOLNIR FIRING:", atomKey);
    inFlight(changeSetId, `${kind}.${id}`);
    // NOTE: since we're moving to all weak refs
    // storing the index becomes useful here, we can lookup the
    // checksum we would expect to be returned, and see if we have it already
    if (!checksum)
      return mjolnirJob(workspaceId, changeSetId, kind, id, checksum);

    // these are sent after patches are completed
    // double check that i am still necessary!
    const exists = await atomExistsOnIndexes(kind, id, checksum);
    if (exists.length === 0)
      return mjolnirJob(workspaceId, changeSetId, kind, id, checksum);
    // if i have it, bust!
    else bustCacheAndReferences(workspaceId, changeSetId, kind, id);
  });
};

const mjolnirJob = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: string,
  id: Id,
  checksum?: Checksum,
) => {
  debug("🔨 MJOLNIR JOB START:", kind, id, "requested checksum:", checksum);
  // TODO this is probably a WsEvent, so SDF knows who to reply to
  const pattern = [
    "v2",
    "workspaces",
    { workspaceId },
    "change-sets",
    { changeSetId },
    "index",
    "mjolnir",
  ] as URLPattern;
  const [url, desc] = describePattern(pattern);
  const params = { changeSetId, kind, id, checksum };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let req: undefined | AxiosResponse<IndexObjectMeta, any>;

  await tracer.startActiveSpan(`GET ${desc}`, async (span) => {
    span.setAttributes({ workspaceId, changeSetId, kind, id, checksum });
    try {
      req = await sdf<IndexObjectMeta>({
        method: "get",
        url,
        params,
      });
      debug("🔨 MJOLNIR HTTP SUCCESS:", kind, id, "status:", req.status);
    } catch (err) {
      span.setAttribute("http.status", 404);
      debug("🔨 MJOLNIR HTTP 404:", kind, id, err);
      error("MJOLNIR 404", url, params, err);
    } finally {
      if (req?.status) span.setAttribute("http.status", req.status);
      span.end();
    }
  });

  returned(changeSetId, `${kind}.${id}`);
  hasReturned({
    workspaceId,
    changeSetId,
    kind,
    id,
  });

  if (!req) {
    debug("🔨 MJOLNIR JOB FAILED:", kind, id, "no response");
    return; // 404
  }

  // Include index checksum in the atom meta for better validation
  const indexChecksum = req.data.indexChecksum;
  const responseChecksum = req.data.frontEndObject.checksum;
  debug(
    "🔨 MJOLNIR RESPONSE:",
    kind,
    id,
    "response checksum:",
    responseChecksum,
    "index checksum:",
    indexChecksum,
  );

  // Check if this conflicts with what we requested
  if (checksum && checksum !== responseChecksum) {
    debug(
      "🔨 MJOLNIR CHECKSUM MISMATCH:",
      kind,
      id,
      "requested:",
      checksum,
      "received:",
      responseChecksum,
    );
  }

  const msg: AtomMessage = {
    kind: MessageKind.MJOLNIR,
    atom: {
      id: req.data.frontEndObject.id,
      kind: req.data.frontEndObject.kind,
      toChecksum: req.data.frontEndObject.checksum,
      workspaceId,
      changeSetId,
      toIndexChecksum: indexChecksum,
      fromIndexChecksum: indexChecksum,
    },
    data: req.data.frontEndObject.data,
  };

  debug("🔨 MJOLNIR JOB COMPLETE:", kind, id, "sending to handleHammer");
  processMjolnirQueue.add(() => handleHammer(msg));
};

const updateChangeSetWithNewIndex = async (meta: AtomMeta) => {
  db.exec({
    sql: "update changesets set index_checksum = ? where change_set_id = ?;",
    bind: [meta.toIndexChecksum, meta.changeSetId],
  });
};

const removeOldIndex = async () => {
  await tracer.startActiveSpan("removeOldIndex", async (span) => {
    // Keep the last 5 indexes per changeset for debugging purposes
    // This helps track previous session checksums
    const deleteIndexes = db.exec({
      sql: `
        DELETE FROM indexes
        WHERE checksum NOT IN (
          SELECT index_checksum FROM changesets
        )
        RETURNING *;
      `,
      returnValue: "resultRows",
    });

    // Only delete atoms that aren't referenced by any index (including retained ones)
    const deleteAtoms = db.exec({
      sql: `
        DELETE FROM atoms
        WHERE (kind, args, checksum) NOT IN (
          SELECT kind, args, checksum FROM index_mtm_atoms
        ) returning atoms.kind, atoms.args, atoms.checksum;
      `,
      returnValue: "resultRows",
    });

    span.setAttributes({
      indexes: JSON.stringify(deleteIndexes),
      atoms: JSON.stringify(deleteAtoms),
    });

    if (deleteIndexes.length > 0) {
      debug(
        "🗑️ Cleaned up",
        deleteIndexes.length,
        "old indexes (keeping recent 5 per workspace)",
        deleteIndexes,
      );
    }
    if (deleteAtoms.length > 0) {
      debug("🗑️ Cleaned up", deleteAtoms.length, "orphaned atoms", deleteAtoms);
    }

    span.end();
  });
};

const pruneAtomsForClosedChangeSet = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
) => {
  await tracer.startActiveSpan("pruneClosedChangeSet", async (span) => {
    span.setAttributes({ workspaceId, changeSetId });
    db.exec({
      sql: `
        DELETE FROM changesets WHERE change_set_id = ?;
      `,
      bind: [changeSetId],
    });
    await removeOldIndex();
    span.end();
  });
};

const atomChecksumsFor = async (
  changeSetId: ChangeSetId,
): Promise<Record<QueryKey, Checksum>> => {
  const mapping: Record<QueryKey, Checksum> = {};
  const rows = db.exec({
    sql: `
    select atoms.kind, atoms.args, atoms.checksum
    from atoms
    inner join index_mtm_atoms mtm
      ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
    inner join indexes ON mtm.index_checksum = indexes.checksum
    inner join changesets ON changesets.index_checksum = indexes.checksum
    where changesets.change_set_id = ?
    ;
    `,
    bind: [changeSetId],
    returnValue: "resultRows",
  });
  rows.forEach((row) => {
    const key = `${row[0]}|${row[1]}` as QueryKey;
    const checksum = row[2] as Checksum;
    mapping[key] = checksum;
  });
  return mapping;
};

/**
 *
 * LIFECYCLE EVENTS
 *
 */

const niflheim = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
): Promise<boolean> => {
  return await tracer.startActiveSpan("niflheim", async (span: Span) => {
    // build connections list based on data we have in the DB
    // connections list will rebuild as data comes in
    bulkInflight();
    const computedPromise = coldStartComputed(workspaceId, changeSetId);

    // clear out references, no queries have been performed yet
    clearAllWeakReferences(changeSetId);

    const pattern = [
      "v2",
      "workspaces",
      { workspaceId },
      "change-sets",
      { changeSetId },
      "index",
    ] as URLPattern;

    const [url, desc] = describePattern(pattern);
    const frigg = tracer.startSpan(`GET ${desc}`);
    frigg.setAttributes({ workspaceId, changeSetId });
    const reqPromise = sdf<IndexObjectMeta>({
      method: "get",
      url,
    });
    const [req, _p] = await Promise.all([reqPromise, computedPromise]);

    // Check for 202 status - user needs to go to lobby
    if (req.status === 202) {
      frigg.setAttribute("status", 202);
      frigg.setAttribute("shouldNavigateToLobby", true);
      frigg.end();
      span.end();
      return false;
    }

    // Use index checksum for validation - this is more reliable than snapshot addresses
    const indexChecksum = req.data.indexChecksum;
    const atoms = req.data.frontEndObject.data.mvList;
    frigg.setAttribute("numEntries", atoms.length);
    frigg.setAttribute("indexChecksum", indexChecksum);
    frigg.end();

    debug("🔍 Index checksum validation", indexChecksum);

    const local = tracer.startSpan("localChecksums");
    const localChecksums = await atomChecksumsFor(changeSetId);
    local.setAttribute("numEntries", Object.keys(localChecksums).length);
    local.end();

    let numHammers = 0;
    // Compare each atom checksum from the index with local checksums
    const objs: MjolnirBulk = [];
    atoms.forEach(({ kind, id, checksum }) => {
      const key = partialKeyFromKindAndArgs(kind, id);
      const local = localChecksums[key];
      if (!local || local !== checksum) {
        const { kind, id } = kindAndArgsFromKey(key);
        objs.push({ kind, id, checksum });

        numHammers++;
      }
    });
    span.setAttribute("numHammers", numHammers);
    span.setAttribute("indexChecksum", indexChecksum);

    if (objs.length > 0)
      await mjolnirBulk(workspaceId, changeSetId, objs, indexChecksum);
    else {
      bulkDone(true);
      span.setAttribute("noop", true);
    }

    span.end();
    return true;
  });
};

const ragnarok = async (
  workspaceId: string,
  changeSetId: string,
  noColdStart = false,
) => {
  // get rid of the indexes we have for this changeset
  db.exec({
    sql: `delete from indexes
          where checksum IN (
            select index_checksum
            from changesets
            where workspace_id = ? and change_set_id = ?
          );`,
    bind: [workspaceId, changeSetId],
  });
  // remove the atoms we have for this change set
  await pruneAtomsForClosedChangeSet(workspaceId, changeSetId);
  if (!noColdStart) {
    // call for a cold start to re-populate
    await niflheim(workspaceId, changeSetId);
  }
};

/**
 *
 * WEAK REFERENCE TRACKING
 *
 */

const clearAllWeakReferences = async (changeSetId: string) => {
  const sql = `
    delete from weak_references
    where change_set_id = ?
  ;`;
  const bind = [changeSetId];
  db.exec({
    sql,
    bind,
  });
};

const clearWeakReferences = async (
  changeSetId: string,
  referrer: { kind: string; args: string },
) => {
  const sql = `
    delete from weak_references
    where change_set_id = ? and referrer_kind = ? and referrer_args = ?
  ;`;
  const bind = [changeSetId, referrer.kind, referrer.args];
  db.exec({
    sql,
    bind,
  });
};

const weakReference = async (
  changeSetId: string,
  target: { kind: string; args: string },
  referrer: { kind: string; args: string },
) => {
  const bind = [
    changeSetId,
    target.kind,
    target.args,
    referrer.kind,
    referrer.args,
  ];
  try {
    const sql = `
      insert into weak_references
        (change_set_id, target_kind, target_args, referrer_kind, referrer_args)
      values
        (?, ?, ?, ?, ?)
      on conflict do nothing
    ;`;
    db.exec({
      sql,
      bind,
    });
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(bind, err);
  }
};

/**
 *
 * COMPUTED IMPLEMENTATIONS
 *
 */
const COMPUTED_KINDS: EntityKind[] = [
  EntityKind.Component,
  EntityKind.IncomingConnections,
];

const allPossibleConns = new DefaultMap<
  string,
  Record<string, PossibleConnection>
>(() => ({}));

// the `string` is `${toAttributeValueId}-${fromAttributeValueId}`
const allOutgoingConns = new DefaultMap<
  ChangeSetId,
  DefaultMap<ComponentId, Record<string, BifrostConnection>>
>(() => new DefaultMap(() => ({})));

const coldStartComputed = async (workspaceId: string, changeSetId: string) => {
  const data = (await get(
    workspaceId,
    changeSetId,
    EntityKind.ComponentList,
    workspaceId,
  )) as BifrostComponentList | -1;

  if (data === -1) return;

  await Promise.all(
    data.components.map((c) =>
      updateComputed(
        workspaceId,
        changeSetId,
        EntityKind.Component,
        c,
        undefined,
        false,
      ),
    ),
  );
  // bust everything all at once on cold start
  await bustCacheAndReferences(
    workspaceId,
    changeSetId,
    EntityKind.PossibleConnections,
    workspaceId,
    true,
  );

  const list = (await get(
    workspaceId,
    changeSetId,
    EntityKind.IncomingConnectionsList,
    workspaceId,
    undefined,
    undefined,
    false, // don't compute
  )) as BifrostIncomingConnectionsList | -1;

  if (list === -1) return;

  await Promise.all(
    list.componentConnections.map((c) =>
      updateComputed(
        workspaceId,
        changeSetId,
        EntityKind.IncomingConnections,
        c,
        undefined,
        false,
        false,
      ),
    ),
  );
  // bust everything all at once on cold start
  await bustCacheAndReferences(
    workspaceId,
    changeSetId,
    EntityKind.OutgoingConnections,
    workspaceId,
    true,
  );
};

const updateComputed = async (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  doc: AtomDocument,
  indexChecksum?: string,
  bust = true,
  followReferences = true,
) => {
  if (!COMPUTED_KINDS.includes(kind)) return;

  if (followReferences) {
    const result = await getReferences(
      doc,
      workspaceId,
      changeSetId,
      kind,
      doc.id,
      indexChecksum,
      false,
    );
    doc = result[0];
  }

  if (kind === EntityKind.IncomingConnections) {
    const data = doc as BifrostComponentConnections;
    data.incoming.forEach((incoming) => {
      const id =
        incoming.kind === "management"
          ? `mgmt-${incoming.toComponent.id}-${incoming.fromComponent.id}`
          : `${incoming.toAttributeValueId}-${incoming.fromAttributeValueId}`;
      const outgoing = flip(incoming);
      const conns = allOutgoingConns
        .get(changeSetId)
        .get(incoming.fromComponent.id);
      conns[id] = outgoing;
    });
  } else if (kind === EntityKind.Component) {
    const conns: Record<string, PossibleConnection> = {};

    const component = doc as BifrostComponent;
    Object.values(component.attributeTree.attributeValues).forEach((av) => {
      const prop = component.attributeTree.props[av.propId ?? ""];
      if (av.path && prop && prop.eligibleForConnection && !prop.hidden) {
        if (av.secret) {
          const conn: PossibleConnection = {
            attributeValueId: av.id,
            value: av.secret.name,
            path: av.path,
            name: prop.name,
            componentId: component.id,
            componentName: component.name,
            schemaName: component.schemaName,
            annotation: prop.kind,
          };
          conns[av.id] = conn;
        } else {
          const conn: PossibleConnection = {
            attributeValueId: av.id,
            value: av.value || "<computed>",
            path: av.path,
            name: prop.name,
            componentId: component.id,
            componentName: component.name,
            schemaName: component.schemaName,
            annotation: prop.kind,
          };
          conns[av.id] = conn;
        }
      }
    });

    const existing = allPossibleConns.get(changeSetId);
    allPossibleConns.set(changeSetId, { ...existing, ...conns });

    // dont bust individually on cold start
    if (bust)
      bustCacheFn(
        workspaceId,
        changeSetId,
        EntityKind.PossibleConnections,
        changeSetId,
      );
  }
};

const getConnectionByAnnotation = (
  _workspaceId: string,
  changeSetId: string,
  annotation: string,
) =>
  sortByAnnotation(
    Object.values(allPossibleConns.get(changeSetId)),
    annotation,
  );

const sortByAnnotation = (
  possible: Array<PossibleConnection>,
  annotation: string,
) => {
  const exactMatches: Array<PossibleConnection> = [];
  const typeMatches: Array<PossibleConnection> = [];
  const nonMatches: Array<PossibleConnection> = [];

  possible.forEach((conn) => {
    const kind = conn.annotation;
    // if we've got something like "VPC id" e.g. not one of the basic types
    if (
      !["string", "boolean", "object", "map", "integer"].includes(annotation)
    ) {
      // look for exact matches
      if (kind === annotation) exactMatches.push(conn);
      // otherwise, all string types match "exact" types
      else if (annotation === "string") typeMatches.push(conn);
      else nonMatches.push(conn);
    } else {
      if (kind === annotation) typeMatches.push(conn);
      else nonMatches.push(conn);
    }
  });

  const cmp = (a: PossibleConnection, b: PossibleConnection) =>
    `${a.name} ${a.path}`.localeCompare(`${b.name} ${b.path}`);
  exactMatches.sort(cmp);
  typeMatches.sort(cmp);
  nonMatches.sort(cmp);

  return { exactMatches, typeMatches, nonMatches };
};

const getOutgoingConnectionsByComponentId = (
  _workspaceId: string,
  changeSetId: string,
) => {
  return allOutgoingConns.get(changeSetId);
};

const flip = (i: BifrostConnection): BifrostConnection => {
  const o: BifrostConnection = {
    ...i,
    fromComponent: i.toComponent,
    toComponent: i.fromComponent,
  };
  if ("toPropId" in i && o.kind === "prop") {
    o.fromPropId = i.toPropId;
    o.fromPropPath = i.toPropPath;
    o.toPropId = i.fromPropId;
    o.toPropPath = i.fromPropId;
    o.fromAttributeValueId = i.toAttributeValueId;
    o.fromAttributeValuePath = i.toAttributeValuePath;
    o.toAttributeValueId = i.fromAttributeValueId;
    o.toAttributeValuePath = i.fromAttributeValuePath;
  }
  return o;
};

/**
 *
 * FETCHING LOGIC
 *
 * EXAMPLE OF HOW WE MOVE FROM
 * - `get` (aka `bifrost`)
 * - `getReferences`
 * - `getComputed`

 * Looking at `get` where `kind="ComponentList"
 *
 * 1. get the atom, edda generates references for us
 * 2. That type is the `EddaComponentList`
 * 3. Call `getReferences`
 * 3. Look up the strong references and fill them in with the `Component` type
 * 4. This translates type to `BifrostComponentList`, which is what we are returning
 * 6. Call `getComputed`
 * 7. Create a map of outgoing connections based on the incoming connections
 * 8. Fill in the `Component.outputCount` connections with them
 * 9. return (we don't need to translate this type)
 */

const getComputed = async (
  atomDoc: AtomDocument,
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
) => {
  // PSA: in general, any `get` you do in here, you're going to want to pass `followComputed=false`
  // otherwise you're liable to run into an infinite recursion lookup
  if (
    ![
      EntityKind.Component,
      EntityKind.ViewComponentList,
      EntityKind.ComponentList,
    ].includes(kind)
  )
    return atomDoc;

  const connectionsById = getOutgoingConnectionsByComponentId(
    workspaceId,
    changeSetId,
  );
  if (!connectionsById) {
    debug("~ missing connections ~");
    // making this, so when connections populate, we re-query
    weakReference(
      changeSetId,
      { kind: EntityKind.OutgoingConnections, args: workspaceId },
      { kind, args: id },
    );
    return atomDoc;
  }

  //  debug("🔗 computed operation", kind, id);

  if (
    kind === EntityKind.ViewComponentList ||
    kind === EntityKind.ComponentList
  ) {
    const data = atomDoc as BifrostComponentList;
    data.components.forEach((c) => {
      c.outputCount = Object.values(connectionsById.get(c.id)).length;
    });
    clearWeakReferences(changeSetId, { kind, args: id });
    weakReference(
      changeSetId,
      { kind: "OutgoingConnections", args: workspaceId },
      { kind, args: id },
    );
    return data;
  } else if (kind === EntityKind.Component) {
    const data = atomDoc as BifrostComponent;
    data.outputCount = Object.values(connectionsById.get(id)).length;
    clearWeakReferences(changeSetId, { kind, args: id });
    weakReference(
      changeSetId,
      { kind: "OutgoingConnections", args: workspaceId },
      { kind, args: id },
    );
    return data;
  } else return atomDoc;
};

/**
 * RULES FOR REFERENCES
 * When you look up a reference with a `get` call
 * you must check for missing data (-1)
 *
 * If you are looking up a `WeakReference`
 * THOU SHALT make a `weakReference` entry for it in all cases
 *
 * If you are looking up a `Reference`
 * THOU SHALT make a `weakReference` on a miss (-1)
 */
const getReferences = async (
  atomDoc: AtomDocument,
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: EntityKind,
  id: Id,
  indexChecksum?: string,
  followComputed?: boolean,
) => {
  if (
    ![
      EntityKind.Component,
      EntityKind.ViewList,
      EntityKind.ComponentList,
      EntityKind.ViewComponentList,
      EntityKind.IncomingConnections,
      EntityKind.IncomingConnectionsList,
      EntityKind.SchemaVariantCategories,
      EntityKind.SecretDefinitionList,
      EntityKind.SecretList,
      EntityKind.Secret,
    ].includes(kind)
  )
    return [atomDoc, false];

  const span = tracer.startSpan("getReferences");
  span.setAttributes({
    workspaceId,
    changeSetId,
    kind,
    id,
  });

  //  debug("🔗 reference query", kind, id);

  let hasReferenceError = false;

  if (kind === EntityKind.SchemaVariantCategories) {
    const data = atomDoc as EddaSchemaVariantCategories;
    const bifrost: BifrostSchemaVariantCategories = {
      id: data.id,
      categories: [],
    };
    await Promise.all(
      data.categories.map(async (category) => {
        const variants = await Promise.all(
          category.schemaVariants.map(async (schemaVariant) => {
            if (schemaVariant.type === "uninstalled") {
              // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
              const variant = data.uninstalled[schemaVariant.id]!;
              variant.uninstalled = "uninstalled";
              return variant;
            } else {
              const sd = (await get(
                workspaceId,
                changeSetId,
                EntityKind.SchemaVariant,
                schemaVariant.id,
                undefined,
                indexChecksum,
                followComputed,
              )) as SchemaVariant | -1;

              if (sd === -1) {
                hasReferenceError = true;
                mjolnir(
                  workspaceId,
                  changeSetId,
                  EntityKind.SchemaVariant,
                  schemaVariant.id,
                );
              }
              weakReference(
                changeSetId,
                { kind: EntityKind.SchemaVariant, args: schemaVariant.id },
                { kind, args: data.id },
              );
              return sd;
            }
          }),
        );
        const schemaVariants = variants.filter(
          (v): v is CategoryVariant => v !== -1 && v && "schemaId" in v,
        );
        bifrost.categories.push({
          displayName: category.displayName,
          schemaVariants,
        });
      }),
    );
    span.end();
    return [bifrost, hasReferenceError];
  } else if (kind === EntityKind.Component) {
    const data = atomDoc as EddaComponent;
    const sv = (await get(
      workspaceId,
      changeSetId,
      data.schemaVariantId.kind,
      data.schemaVariantId.id,
      undefined,
      indexChecksum,
      followComputed,
    )) as SchemaVariant | -1;

    if (sv === -1) {
      hasReferenceError = true;
      span.addEvent("mjolnir", {
        workspaceId,
        changeSetId,
        kind: data.schemaVariantId.kind,
        id: data.schemaVariantId.id,
        source: "getReferences",
        sourceKind: kind,
      });
      mjolnir(
        workspaceId,
        changeSetId,
        data.schemaVariantId.kind,
        data.schemaVariantId.id,
      );
      // add a weak reference in the case of a miss
      // because if we throw a hammer for what we missed
      // this referencing data doesn't change and needs to bust
      weakReference(
        changeSetId,
        { kind: data.schemaVariantId.kind, args: data.schemaVariantId.id },
        { kind, args: data.id },
      );
    }

    const sm = (await get(
      workspaceId,
      changeSetId,
      data.schemaMembers.kind,
      data.schemaMembers.id,
      undefined,
      indexChecksum,
      followComputed,
    )) as SchemaMembers | -1;

    if (sm === -1) {
      hasReferenceError = true;
      span.addEvent("mjolnir", {
        workspaceId,
        changeSetId,
        kind: data.schemaMembers.kind,
        id: data.schemaMembers.id,
        source: "getReferences",
        sourceKind: kind,
      });
      mjolnir(
        workspaceId,
        changeSetId,
        data.schemaMembers.kind,
        data.schemaMembers.id,
      );
      // add a weak reference in the case of a miss
      // because if we throw a hammer for what we missed
      // this referencing data doesn't change and needs to bust
      weakReference(
        changeSetId,
        { kind: data.schemaMembers.kind, args: data.schemaMembers.id },
        { kind, args: data.id },
      );
    }

    const schemaMembers = sm !== -1 ? sm : ({} as SchemaMembers);
    let canBeUpgraded = false;
    if (schemaMembers) {
      if (
        schemaMembers.editingVariantId &&
        data.schemaVariantId.id !== schemaMembers.editingVariantId
      ) {
        canBeUpgraded = true;
      } else if (data.schemaVariantId.id !== schemaMembers.defaultVariantId) {
        canBeUpgraded = true;
      }
    }

    const component: BifrostComponent = {
      ...data,
      canBeUpgraded,
      schemaVariant: sv !== -1 ? sv : ({} as SchemaVariant),
    };
    span.end();
    return [component, hasReferenceError];
  } else if (kind === EntityKind.ViewList) {
    const rawList = atomDoc as RawViewList;
    const maybeViews = await Promise.all(
      rawList.views.map(async (v) => {
        const maybeDoc = (await get(
          workspaceId,
          changeSetId,
          v.kind,
          v.id,
          v.checksum,
          indexChecksum,
          followComputed,
        )) as View | -1;
        if (maybeDoc === -1) {
          hasReferenceError = true;
          span.addEvent("mjolnir", {
            workspaceId,
            changeSetId,
            kind: v.kind,
            id: v.id,
            source: "getReferences",
            sourceKind: kind,
          });
          mjolnir(workspaceId, changeSetId, v.kind, v.id);
          weakReference(
            changeSetId,
            { kind: v.kind, args: v.id },
            { kind, args: rawList.id },
          );
        }
        return maybeDoc;
      }),
    );
    const views = maybeViews.filter(
      (v): v is View => v !== -1 && v && "id" in v,
    );
    const list: BifrostViewList = {
      id: rawList.id,
      views,
    };
    span.end();
    return [list, hasReferenceError];
  } else if (
    kind === EntityKind.ComponentList ||
    kind === EntityKind.ViewComponentList
  ) {
    const rawList = atomDoc as EddaComponentList;
    const maybeComponents = await Promise.all(
      rawList.components.map(async (c) => {
        const maybeDoc = (await get(
          workspaceId,
          changeSetId,
          c.kind,
          c.id,
          c.checksum,
          indexChecksum,
          followComputed,
        )) as BifrostComponent | -1;

        if (maybeDoc === -1) {
          hasReferenceError = true;
          span.addEvent("mjolnir", {
            workspaceId,
            changeSetId,
            kind: c.kind,
            id: c.id,
            source: "getReferences",
            sourceKind: kind,
          });
          mjolnir(workspaceId, changeSetId, c.kind, c.id);
          weakReference(
            changeSetId,
            { kind: c.kind, args: c.id },
            { kind, args: rawList.id },
          );
        }
        return maybeDoc;
      }),
    );
    const components = maybeComponents.filter(
      (c): c is BifrostComponent => c !== -1 && c && "id" in c,
    );
    // NOTE: this is either a bifrost component list or a view component list
    // FUTURE: improve this with some typing magic
    const list: BifrostComponentList = {
      id: rawList.id,
      components,
    };
    span.end();
    return [list, hasReferenceError];
  } else if (kind === EntityKind.IncomingConnections) {
    const raw = atomDoc as EddaIncomingConnections;
    const component = (await get(
      workspaceId,
      changeSetId,
      EntityKind.Component,
      raw.id,
      undefined,
      indexChecksum,
      false,
    )) as BifrostComponent | -1;

    clearWeakReferences(changeSetId, {
      kind: EntityKind.IncomingConnections,
      args: raw.id,
    });

    weakReference(
      changeSetId,
      { kind: EntityKind.Component, args: raw.id },
      { kind: EntityKind.IncomingConnections, args: raw.id },
    );

    if (component === -1) {
      span.addEvent("mjolnir", {
        workspaceId,
        changeSetId,
        kind: EntityKind.Component,
        id: raw.id,
        source: "getReferences",
        sourceKind: kind,
      });
      mjolnir(workspaceId, changeSetId, EntityKind.Component, raw.id);
      debug(`Connection ${raw.id} missing own component`);
      hasReferenceError = true;
    }
    // explicitly setting this as a warning that these fields are not to be used
    else (component as BifrostComponent).outputCount = -1;

    const connections = await Promise.all(
      raw.connections.map(async (c: EddaConnection) => {
        // NOTE: when looking up the weak referenced components in a list of component connections
        // we pass `followComputed=false` because we don't need the BifrostComponent objects to look up
        // their own connection stats, we're calling `IncomingConnections` after all!
        const fromComponent = await get(
          workspaceId,
          changeSetId,
          c.fromComponentId.kind,
          c.fromComponentId.id,
          undefined,
          indexChecksum,
          false,
        );

        weakReference(
          changeSetId,
          { kind: c.fromComponentId.kind, args: c.fromComponentId.id },
          { kind: EntityKind.IncomingConnections, args: raw.id },
        );

        if (fromComponent === -1) {
          span.addEvent("mjolnir", {
            workspaceId,
            changeSetId,
            kind: c.fromComponentId.kind,
            id: c.fromComponentId.id,
            source: "getReferences",
            sourceKind: kind,
          });
          mjolnir(
            workspaceId,
            changeSetId,
            c.fromComponentId.kind,
            c.fromComponentId.id,
          );
          hasReferenceError = true;
        }
        // explicitly setting this as a warning that these fields are not to be used
        else (fromComponent as BifrostComponent).outputCount = -1;

        const conn: BifrostConnection = {
          ...c,
          fromComponent: fromComponent as BifrostComponent,
          toComponent: component as BifrostComponent,
        };
        return conn;
      }),
    );

    span.end();
    return [
      {
        id: raw.id,
        component,
        incoming: connections,
        outgoing: [] as BifrostConnection[],
      } as BifrostComponentConnections,
      hasReferenceError,
    ];
  } else if (kind === EntityKind.IncomingConnectionsList) {
    const rawList = atomDoc as EddaIncomingConnectionsList;
    const maybeIncomingConnections = await Promise.all(
      rawList.componentConnections.map(async (c) => {
        const maybeDoc = (await get(
          workspaceId,
          changeSetId,
          c.kind,
          c.id,
          undefined,
          indexChecksum,
        )) as BifrostComponentConnections | -1;
        if (maybeDoc === -1) {
          hasReferenceError = true;
          span.addEvent("mjolnir", {
            workspaceId,
            changeSetId,
            kind: c.kind,
            id: c.id,
            source: "getReferences",
            sourceKind: kind,
          });
          mjolnir(workspaceId, changeSetId, c.kind, c.id);
          weakReference(
            changeSetId,
            { kind: c.kind, args: c.id },
            { kind, args: rawList.id },
          );
        }
        return maybeDoc;
      }),
    );
    const componentConnections = maybeIncomingConnections.filter(
      (c): c is BifrostComponentConnections => c !== -1 && c && "id" in c,
    );
    const list: BifrostIncomingConnectionsList = {
      id: rawList.id,
      componentConnections,
    };
    span.end();
    return [list, hasReferenceError];
  } else {
    span.end();
    return [atomDoc, hasReferenceError];
  }
};

const get = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: EntityKind,
  id: Id,
  checksum?: string, // intentionally not used in sql, putting it on the wire for consistency & observability purposes
  indexChecksum?: string,
  followComputed = true,
  followReferences = true,
): Promise<-1 | object> => {
  const sql = `
    select
      data
    from
      atoms
      inner join index_mtm_atoms mtm
        ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
      inner join indexes ON mtm.index_checksum = indexes.checksum
    ${
      indexChecksum
        ? ""
        : "inner join changesets ON changesets.index_checksum = indexes.checksum"
    }
    where
      ${indexChecksum ? "indexes.checksum = ?" : "changesets.change_set_id = ?"}
      AND
      atoms.kind = ? AND
      atoms.args = ?
    ;`;
  const bind = [indexChecksum ?? changeSetId, kind, id];
  const start = Date.now();
  const atomData = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  const end = Date.now();
  const data = oneInOne(atomData);
  debug(
    "❓ sql get",
    `[${end - start}ms]`,
    bind,
    " returns ?",
    !(data === NOROW),
  );
  if (data === NOROW) {
    mjolnir(workspaceId, changeSetId, kind, id, checksum);
    return -1;
  }
  const atomDoc = decodeDocumentFromDB(data as ArrayBuffer);
  // debug("📄 atom doc", atomDoc);

  // THIS GETS REPLACED WITH AUTO-GEN CODE
  if (!followReferences) return atomDoc;

  try {
    const [docAndRefs, hasReferenceError] = await getReferences(
      atomDoc,
      workspaceId,
      changeSetId,
      kind,
      id,
      indexChecksum,
      followComputed,
    );
    // this is a choice, we could send through objects that don't match the types
    // and potentially have something drawn on the screen—but that seems worse
    // for the possible side-effects
    if (hasReferenceError) return -1;

    // NOTE: Whenever we ask for the full list of connections
    // This implementation will not compute the outgoing connections (infinite recursion)
    // You will only get incoming—which is all we need when we ask for the whole list
    if (followComputed && !["IncomingConnectionsList"].includes(kind)) {
      return await getComputed(docAndRefs, workspaceId, changeSetId, kind, id);
    }
    return docAndRefs;
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(err);
    return -1;
  }
};

/**
 *
 * INTERFACE DEFINITION
 *
 */

let socket: ReconnectingWebSocket;
let bustCacheFn: BustCacheFn;
let bearerToken: string;

let inFlight: RainbowFn;
let returned: RainbowFn;
let lobbyExitFn: LobbyExitFn;

const dbInterface: DBInterface = {
  setBearer(token) {
    bearerToken = token;
    let apiUrl: string;
    if (import.meta.env.VITE_API_PROXY_PATH) {
      // eslint-disable-next-line no-restricted-globals
      apiUrl = `${location.protocol}//${location.host}${
        import.meta.env.VITE_API_PROXY_PATH
      }`;
    } else throw new Error("Invalid API env var config");
    const API_HTTP_URL = apiUrl;

    sdf = Axios.create({
      headers: {
        "Content-Type": "application/json",
      },
      baseURL: API_HTTP_URL,
    });
    function injectBearerTokenAuth(config: InternalAxiosRequestConfig) {
      // inject auth token from the store as a custom header
      config.headers = config.headers || {};

      if (token) {
        config.headers.authorization = `Bearer ${token}`;
      }
      return config;
    }

    sdf.interceptors.request.use(injectBearerTokenAuth);
  },
  async initDB(testing: boolean) {
    return initializeSQLite(testing);
  },

  async migrate(testing: boolean) {
    const result = ensureTables(testing);
    debug("Migration completed");
    return result;
  },

  async initSocket() {
    try {
      socket = new ReconnectingWebSocket(
        () => `/api/ws/bifrost?token=Bearer+${bearerToken}`,
        [],
        {
          // see options https://www.npmjs.com/package/reconnecting-websocket#available-options
          startClosed: true, // don't start connected - we'll watch auth to trigger
          // TODO: tweak settings around reconnection behaviour
        },
      );
    } catch (err) {
      error(err);
    }

    socket.addEventListener("message", (messageEvent) => {
      tracer.startActiveSpan("handleEvent", async (span) => {
        // we'll either be getting AtomMessages as patches to the data
        // OR we'll be getting mjolnir responses with the Atom as a whole
        // TODO we also need "changeset closed" messages
        // TODO: handle Index Updates!
        try {
          const data = JSON.parse(messageEvent.data) as
            | PatchBatch
            | AtomMessage
            | IndexUpdate;

          if (import.meta.env.VITE_LOG_WS) {
            log("🌈 bifrost incoming", data);
          }

          // Track message processing to detect duplicates
          let messageId: string;
          if (data.kind === MessageKind.PATCH) {
            messageId = `${data.kind}-${data.meta.toIndexChecksum}-${data.patches.length}`;
          } else if (data.kind === MessageKind.MJOLNIR) {
            messageId = `${data.kind}-${data.atom.kind}-${data.atom.id}-${data.atom.toChecksum}`;
          } else {
            messageId = `${data.kind}`;
          }
          debug("📨 Processing message:", messageId);

          if (!("kind" in data)) span.setAttribute("kindMissing", "no kind");
          else {
            span.setAttribute("messageKind", data.kind);
            if (data.kind === MessageKind.PATCH) {
              if (!data.meta.toIndexChecksum)
                // eslint-disable-next-line no-console
                console.error(
                  "ATTEMPTING TO PATCH BUT INDEX CHECKSUM IS MISSING",
                  data.meta,
                );
              debug(
                "📨 PATCH MESSAGE START:",
                data.meta.toIndexChecksum,
                "patches:",
                data.patches.length,
              );
              processPatchQueue.add(() => handlePatchMessage(data));
              debug("📨 PATCH MESSAGE COMPLETE:", data.meta.toIndexChecksum);
            } else if (data.kind === MessageKind.MJOLNIR) {
              debug(
                "📨 MJOLNIR MESSAGE START:",
                data.atom.kind,
                data.atom.id,
                "toChecksum:",
                data.atom.toChecksum,
              );
              returned(
                data.atom.changeSetId,
                `${data.atom.kind}.${data.atom.id}`,
              );
              hasReturned({
                workspaceId: data.atom.workspaceId,
                changeSetId: data.atom.changeSetId,
                kind: data.atom.kind,
                id: data.atom.id,
              });
              processMjolnirQueue.add(() => handleHammer(data));
              debug(
                "📨 MJOLNIR MESSAGE COMPLETE:",
                data.atom.kind,
                data.atom.id,
              );
            } else if (data.kind === MessageKind.INDEXUPDATE) {
              // Index has been updated - signal lobby exit
              if (lobbyExitFn) {
                lobbyExitFn(data.meta.workspaceId, data.meta.changeSetId);
              }
            }
          }
        } catch (err: unknown) {
          error(err);
          if (err instanceof Error) {
            if (import.meta.env.VITE_LOG_WS) {
              log("🌈 bifrost incoming", messageEvent.data);
            }
            span.addEvent("error", {
              "error.message": err.message,
              "error.stacktrace": err.stack,
            });
          }
        }
        span.end();
      });
    });

    socket.addEventListener("error", (errorEvent) => {
      error("ws error", errorEvent.error, errorEvent.message);
    });
  },

  async initBifrost() {
    debug("🌈 Initializing Bifrost");
    await Promise.all([this.initDB(false), this.initSocket()]);
    await this.migrate(false);
    debug("🌈 Bifrost initialization complete");
  },

  async bifrostClose() {
    try {
      if (socket) socket.close();
    } catch (err) {
      error(err);
    }
  },

  async bifrostReconnect() {
    try {
      if (socket) socket.reconnect();
    } catch (err) {
      error(err);
    }
  },

  async addListenerBustCache(cb: BustCacheFn) {
    bustCacheFn = cb;
  },

  async addListenerInFlight(cb: RainbowFn) {
    inFlight = cb;
  },
  async addListenerReturned(cb: RainbowFn) {
    returned = cb;
  },

  async addListenerLobbyExit(cb: LobbyExitFn) {
    lobbyExitFn = cb;
  },

  get,
  getOutgoingConnectionsByComponentId,
  getConnectionByAnnotation,
  partialKeyFromKindAndId: partialKeyFromKindAndArgs,
  kindAndIdFromKey: kindAndArgsFromKey,
  mjolnirBulk,
  mjolnir,
  atomChecksumsFor,
  pruneAtomsForClosedChangeSet,
  niflheim,
  encodeDocumentForDB,
  decodeDocumentFromDB,
  handlePatchMessage,
  exec,
  oneInOne,
  handleHammer,
  bobby: dropTables,
  ragnarok,
  changeSetExists: async (workspaceId: string, changeSetId: ChangeSetId) => {
    const row = db.exec({
      sql: "select change_set_id from changesets where workspace_id = ? and change_set_id = ?",
      returnValue: "resultRows",
      bind: [workspaceId, changeSetId],
    });
    const cId = oneInOne(row);
    return cId === changeSetId;
  },

  async odin(changeSetId: ChangeSetId): Promise<object> {
    const c = db.exec({
      sql: "select * from changesets where change_set_id=?;",
      bind: [changeSetId],
      returnValue: "resultRows",
    });
    const i = db.exec({
      sql: `select indexes.* from indexes
            inner join changesets
              on indexes.checksum = changesets.index_checksum
            where changesets.change_set_id = ?;
      `,
      bind: [changeSetId],
      returnValue: "resultRows",
    });
    const m = db.exec({
      sql: `select index_mtm_atoms.* from index_mtm_atoms
            inner join changesets
              on index_mtm_atoms.index_checksum = changesets.index_checksum
            where changesets.change_set_id = ?;
      `,
      bind: [changeSetId],
      returnValue: "resultRows",
    });
    const a = db.exec({
      sql: `select atoms.* from atoms
            inner join index_mtm_atoms
              on index_mtm_atoms.kind = atoms.kind
              and index_mtm_atoms.args = atoms.args
              and index_mtm_atoms.checksum = atoms.checksum
            inner join changesets
              on index_mtm_atoms.index_checksum = changesets.index_checksum
            where changesets.change_set_id = ?;
      `,
      bind: [changeSetId],
      returnValue: "resultRows",
    });
    const [changesets, indexes, atoms, mtm] = await Promise.all([c, i, a, m]);
    return { changesets, indexes, atoms, mtm };
  },
  async linkNewChangeset(
    workspaceId,
    headChangeSet,
    changeSetId,
  ): Promise<void> {
    try {
      const headRows = db.exec({
        sql: "select index_checksum from changesets where workspace_id = ? and change_set_id = ?;",
        bind: [workspaceId, headChangeSet],
        returnValue: "resultRows",
      });
      const headRow = oneInOne(headRows);
      if (headRow === NOROW)
        throw new Error(`HEAD is missing: ${workspaceId}: ${headChangeSet}`);
      const currentIndexChecksum = headRow;

      db.exec({
        sql: "insert into changesets (change_set_id, workspace_id, index_checksum) VALUES (?, ?, ?);",
        bind: [changeSetId, workspaceId, currentIndexChecksum],
      });
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error("linkNewChangeset", err);
    }
  },
};

Comlink.expose(dbInterface);
