import * as Comlink from "comlink";
import { applyPatch as applyOperations, Operation } from "fast-json-patch";
import QuickLRU from "quick-lru";
import sqlite3InitModule, {
  Database,
  ExecBaseOptions,
  ExecReturnResultRowsOptions,
  ExecRowModeArrayOptions,
  FlexibleString,
  SAHPoolUtil,
  Sqlite3Static,
  SqlValue,
} from "@sqlite.org/sqlite-wasm";
import ReconnectingWebSocket from "reconnecting-websocket";
import { Span, trace } from "@opentelemetry/api";
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
import { describePattern, URLPattern } from "@si/vue-lib";
import Axios, {
  AxiosInstance,
  AxiosResponse,
  InternalAxiosRequestConfig,
} from "axios";
import * as _ from "lodash-es";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { nonNullable } from "@/utils/typescriptLinter";
import { DefaultMap } from "@/utils/defaultmap";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { WorkspacePk } from "@/api/sdf/dal/workspace";
import { ViewId } from "@/api/sdf/dal/views";
import {
  WorkspaceAtom,
  AtomDocument,
  WorkspaceAtomMessage,
  WorkspaceAtomMeta,
  BroadcastMessage,
  BustCacheFn,
  Checksum,
  Common,
  ComponentInfo,
  FORCE_LEADER_ELECTION,
  Gettable,
  Id,
  IndexObjectMeta,
  WorkspaceIndexUpdate,
  Listable,
  LobbyExitFn,
  MessageKind,
  MjolnirBulk,
  NOROW,
  WorkspacePatchBatch,
  QueryAttributesTerm,
  QueryKey,
  RainbowFn,
  TabDBInterface,
  DB_NOT_INIT_ERR,
  UpdateFn,
  DeploymentIndexUpdate,
  DeploymentPatchBatch,
  ConnStatusFn,
  StoredMvIndex,
  AtomWithData,
  AtomOperation,
} from "./types/dbinterface";
import {
  BifrostComponent,
  Connection,
  EddaComponent,
  IncomingConnections,
  EntityKind,
  PossibleConnection,
  SchemaMembers,
  SchemaVariant,
  AttributeTree,
  ManagementConnections,
  DefaultSubscriptions,
  DefaultSubscription,
  AttributeValue,
  GLOBAL_ENTITIES,
  GlobalEntity,
  GLOBAL_IDENTIFIER,
} from "./types/entity_kind_types";
import {
  bulkDone,
  bulkInflight,
  bustQueueAdd,
  hasReturned,
  maybeMjolnir,
  processMjolnirQueue,
  processPatchQueue,
} from "./mjolnir_queue";

const WORKER_LOCK_KEY = "BIFROST_LOCK";

let hasTheLock = false;

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
const error = console.error;
const _DEBUG = import.meta.env.VITE_SI_ENV === "local";
const _START_FRESH = false;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function debug(...args: any | any[]) {
  // eslint-disable-next-line no-console
  if (_DEBUG) console.debug(...args);
}
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function log(...args: any | any[]) {
  // eslint-disable-next-line no-console
  if (_DEBUG) console.log(...args);
}

/**
 *  INITIALIZATION FNS
 */
let sqlite: Database | undefined;
let poolUtil: SAHPoolUtil | undefined;
const sdfClients: { [key: string]: AxiosInstance } = {};

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

  if ("opfs" in sqlite3) {
    if (!poolUtil) {
      poolUtil = await sqlite3.installOpfsSAHPoolVfs({});
    } else if (poolUtil.isPaused()) {
      await poolUtil.unpauseVfs();
    }
    sqlite = new poolUtil.OpfsSAHPoolDb(`/${dbname}`);
    debug(
      `OPFS is available, created persisted database in SAH Pool VFS at ${sqlite.filename}`,
    );
  } else {
    sqlite = new sqlite3.oo1.DB(`/${dbname}`, "c");
    debug(
      `OPFS is not available, created transient database ${sqlite.filename}`,
    );
  }

  sqlite.exec({ sql: "PRAGMA foreign_keys = ON;" });
};

const initializeSQLite = async (testing: boolean) => {
  try {
    const sqlite3 = await sqlite3InitModule({ print: log, printErr: error });
    await start(sqlite3, testing);
  } catch (err) {
    if (err instanceof Error) {
      error("Initialization error:", err.name, err.message);
    } else error("Initialization error:", err);
  }
};

const dropTables = () => {
  const sql = `
  DROP TABLE IF EXISTS index_mtm_atoms;
  DROP TABLE IF EXISTS atoms;
  DROP TABLE IF EXISTS global_atoms;
  DROP TABLE IF EXISTS indexes;
  DROP TABLE IF EXISTS changesets;
  DROP TABLE IF EXISTS weak_references;
  `;
  sqlite?.exec({ sql });
};

// INTEGER is 8 bytes, not large enough to store ULIDs
// we'll go with string, though reading that putting the bytes as BLOBs would save space
const ensureTables = (testing: boolean) => {
  if (_START_FRESH || testing) dropTables();
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

  CREATE TABLE IF NOT EXISTS global_atoms (
    kind TEXT NOT NULL,
    args TEXT NOT NULL,
    checksum TEXT NOT NULL,
    data BLOB,
    PRIMARY KEY (kind, args)
    CONSTRAINT uniqueness UNIQUE (kind, args) ON CONFLICT REPLACE
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

  if (!sqlite) {
    throw new Error(DB_NOT_INIT_ERR);
  }

  return sqlite.exec({ sql });
};

// NOTE: this is just for external test usage, do not use this within this file
const exec = (
  opts: ExecBaseOptions &
    ExecRowModeArrayOptions &
    ExecReturnResultRowsOptions & {
      sql: FlexibleString;
    },
): SqlValue[][] => {
  if (!sqlite) {
    throw new Error(DB_NOT_INIT_ERR);
  }
  return sqlite.exec(opts);
};

/**
 * A few small utilities
 */
const textEncoder = new TextEncoder();
const encodeDocumentForDB = (doc: object) => {
  const docJson = JSON.stringify(doc);
  return textEncoder.encode(docJson);
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
 * INDEX LOGIC
 */

const workspaceAtomExistsOnIndexes = (
  db: Database,
  kind: EntityKind,
  id: string,
  checksum: Checksum,
): Checksum[] => {
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

/**
 * Create a new index, as a copy of an existing index (fromIndexChecksum) if we have it.
 *
 * This assumes no index exists for the given checksum, and that the index for fromIndexChecksum
 * is complete (i.e. associated with a changeset record).
 *
 * @param meta the new and previous indexes for the changeset.
 * @param fromIndexChecksum the checksum the changeset currently has in the frontend
 */
const newChangesetIndex = (
  db: Database,
  meta: WorkspaceAtomMeta,
  fromIndexChecksum: string | undefined,
) => {
  //
  // Create a new empty index
  //
  db.exec({
    sql: `INSERT INTO indexes (checksum) VALUES (?);`,
    bind: [meta.toIndexChecksum],
  });

  //
  // Copy atoms from the previous index
  //
  const rows = db.exec({
    sql: `SELECT index_checksum FROM changesets WHERE change_set_id = ?`,
    bind: [meta.changeSetId],
    returnValue: "resultRows",
  });
  const lastKnownFromChecksum = oneInOne(rows) as
    | string
    | undefined
    | typeof NOROW;

  let sourceChecksum;
  if (fromIndexChecksum && fromIndexChecksum !== meta.toIndexChecksum) {
    // Copy the index from the previous changeset if one exists
    sourceChecksum = fromIndexChecksum;
  } else if (lastKnownFromChecksum && lastKnownFromChecksum !== NOROW) {
    // Copy the index from the previous changeset if one exists
    // TODO may be redundant; the only caller (indexLogic()) already gets fromIndexChecksum
    // from the same place.
    debug(`HIT ELSE BRANCH NEW FROM CHECKSUM SHIT`);
    sourceChecksum = lastKnownFromChecksum;
  } else {
    // we have a new change set and a patch at the same time
    // which means that the change set record did not exist, no "from" in the DB
    // but we have the from in the payload
    //
    // NOTE: this could be incomplete! Cannot be sure an index/atoms are complete unless
    // they are associated with a change_sets record, and we're not checking that here.
    debug(
      `New changeset and patch at the same time! Copying index atoms from edda's changeset ${meta.fromIndexChecksum}`,
    );
    sourceChecksum = meta.fromIndexChecksum;
  }

  // Copy all entries found for sourceChecksum, while rewriting the index_checksum to the incoming one.
  db.exec({
    sql: `INSERT INTO index_mtm_atoms
        SELECT
          ?, kind, args, checksum
        FROM index_mtm_atoms
        WHERE
          index_checksum = ?
        `,
    bind: [meta.toIndexChecksum, sourceChecksum],
  });
};

const bulkRemoveAtoms = async (
  db: Database,
  atoms: Common[],
  indexChecksum: Checksum,
  chunkSize = 2000,
) => {
  for (let i = 0; i < atoms.length; i += chunkSize) {
    const placeholders = [];
    const bind = [];
    const chunk = atoms.slice(i, i + chunkSize);
    for (const atom of chunk) {
      placeholders.push("(?, ?, ?, ?)");
      bind.push(indexChecksum, atom.kind, atom.id, atom.checksum);
    }

    const sql = `delete from index_mtm_atoms
      where (index_checksum, kind, args, checksum) in (${placeholders.join(
        ",",
      )})`;
    db.exec({ sql, bind });
  }
};

const removeAtom = (
  db: Database,
  indexChecksum: Checksum,
  kind: EntityKind,
  id: string,
  checksum: string,
  span?: Span,
) => {
  const start = performance.now();
  db.exec({
    sql: `
    DELETE FROM index_mtm_atoms
    WHERE index_checksum = ? AND kind = ? AND args = ? AND checksum = ?
    `,
    bind: [indexChecksum, kind, id, checksum],
  });
  const end = performance.now();
  span?.setAttribute("performance.removeAtom", end - start);
};

const createAtom = async (
  db: Database,
  atom: Omit<WorkspaceAtom, "toIndexChecksum" | "fromIndexChecksum">,
  doc: object,
  span?: Span,
) => {
  debug("createAtom", atom, doc);

  const encodedDoc = encodeDocumentForDB(doc);
  try {
    const start = performance.now();
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
    const end = performance.now();
    span?.setAttribute("performance.createAtom", end - start);

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
  if (!pieces[0] || !pieces[1]) {
    throw new Error(`Missing key ${key} -> ${pieces}`);
  }
  const kind = pieces[0] as EntityKind;
  const id = pieces[1];
  return { kind, id };
};

const bustDeployment = (kind: GlobalEntity, id: string) => {
  bustCacheFn(GLOBAL_IDENTIFIER, GLOBAL_IDENTIFIER, kind, id);
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
  // if we're busting an entity kind, and the key is not the workspaceId, bust the kind with workspaceId too
  // this make the `getKind` query with `makeArgs` and no `id` passed (which uses the workspace) remain responsive
  if (id !== workspaceId)
    bustOrQueue(workspaceId, changeSetId, kind, workspaceId, skipQueue);
};

const bustCacheAndReferences = (
  db: Database,
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
  skipQueue = false,
  force = false,
) => {
  // don't bust lists in the whole, we're using atomUpdatedFn to update the contents of lists
  // unless its a hammer b/c i am missing a list
  // FIXME(nick,jobelenus): do not bust lists and find a way to support add/remove component(s)
  // from a view without it.
  if (
    kind !== EntityKind.ViewComponentList &&
    LISTABLE.includes(kind) &&
    !force
  )
    return;

  // bust me
  bustOrQueue(workspaceId, changeSetId, kind, id, skipQueue);

  // we're being "special" in that there is no MV for the list of SchemaMembers, b/c we don't even need it
  // but we're using a QueryKey of "all the schema members", under the workspaceId
  if (kind === EntityKind.SchemaMembers) {
    bustOrQueue(workspaceId, changeSetId, kind, workspaceId, skipQueue);
  }

  // FIXME(nick): do not bust lists and find a way to support add/remove component(s) from a view
  // without it.
  if (kind === EntityKind.ViewComponentList) {
    bustOrQueue(
      workspaceId,
      changeSetId,
      EntityKind.ComponentsInViews,
      workspaceId,
      skipQueue,
    );
    bustOrQueue(
      workspaceId,
      changeSetId,
      EntityKind.ComponentsInOnlyOneView,
      workspaceId,
      skipQueue,
    );
  }

  // if we know it doesnt have references, dont even run the sql
  if (!HAVE_REFERENCES.includes(kind)) return;

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
    if (ref_kind && ref_id) {
      bustOrQueue(
        workspaceId,
        changeSetId,
        ref_kind as EntityKind,
        ref_id as string,
        skipQueue,
      );
    }
  });
};

const handleHammer = async (db: Database, msg: WorkspaceAtomMessage) => {
  await tracer.startActiveSpan("Mjolnir", async (span) => {
    debug(
      "🔨 HAMMER RECEIVED:",
      msg.atom.kind,
      msg.atom.id,
      "toChecksum:",
      msg.atom.toChecksum,
    );

    const { changeSetId, workspaceId, toIndexChecksum } = { ...msg.atom };

    span.setAttributes({
      changeSetId,
      workspaceId,
      toIndexChecksum,
    });

    // Log index checksum for validation context
    if (msg.atom.toChecksum) {
      debug("🔨 handling hammer with index checksum", msg.atom.toIndexChecksum);
    }

    // Make sure the index exists before we try to insert atoms into it
    const indexChecksum = initIndexAndChangeSet(db, msg.atom, span);
    if (!indexChecksum) {
      throw new Error(
        `Expected index checksum for ${msg.atom.toIndexChecksum}`,
      );
    }
    // in between throwing a hammer and receiving it, i might already have written the atom
    const indexes = workspaceAtomExistsOnIndexes(
      db,
      msg.atom.kind,
      msg.atom.id,
      msg.atom.toChecksum,
    );
    let noop = false;
    if (indexes.length > 0) {
      if (indexes.includes(msg.atom.toIndexChecksum)) {
        span.setAttributes({
          noop: true,
          upToDateAtomIndexes: indexes,
          needToInsertMTM: false,
        });
        debug(
          "🔨 HAMMER NOOP: Atom already exists in index:",
          msg.atom.kind,
          msg.atom.id,
          msg.atom.toChecksum,
          indexes,
        );
        noop = true;
      } else {
        debug("HAMMER: Atom exists, MTM needed");
        span.setAttributes({
          noop: true,
          upToDateAtomIndexes: indexes,
          needToInsertMTM: true,
        });
        const inserted = insertAtomMTM(db, msg.atom, msg.atom.toIndexChecksum);
        span.setAttribute("insertedMTM", inserted);
        noop = true;
      }
    }

    // if the atom exists, i just need the MTM
    if (indexes.length === 0) {
      debug(
        "🔨 HAMMER: Creating new atom:",
        msg.atom.kind,
        msg.atom.id,
        "checksum:",
        msg.atom.toChecksum,
      );
      span.setAttribute("createAtom", true);
      await createAtom(db, msg.atom, msg.data, span);
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
      span.setAttributes({
        upToDateAtomIndexes: indexes,
        needToInsertMTM: true,
      });
    }

    if (!noop) {
      debug(
        "🔨 HAMMER: Inserting MTM for:",
        msg.atom.kind,
        msg.atom.id,
        "checksum:",
        msg.atom.toChecksum,
        "index:",
        indexChecksum,
      );
      span.setAttributes({
        needToInsertMTM: true,
      });
      const inserted = insertAtomMTM(db, msg.atom, indexChecksum);
      span.setAttribute("insertedMTM", inserted);
    }

    updateChangeSetWithNewIndex(db, msg.atom);
    span.setAttribute("updatedWithNewIndex", true);
    removeOldIndex(db, span);

    if (
      COMPUTED_KINDS.includes(msg.atom.kind) ||
      LISTABLE_ITEMS.includes(msg.atom.kind)
    ) {
      debug("🔨 HAMMER: Updating computed for:", msg.atom.kind, msg.atom.id);
      postProcess(
        db,
        msg.atom.workspaceId,
        msg.atom.changeSetId,
        msg.atom.kind,
        msg.data,
        msg.atom.id,
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
    span.setAttribute(
      "bustCache",
      JSON.stringify([msg.atom.kind, msg.atom.id]),
    );
    bustCacheAndReferences(
      db,
      msg.atom.workspaceId,
      msg.atom.changeSetId,
      msg.atom.kind,
      msg.atom.id,
      false,
      true,
    );
  });
};

// Insert atoms in chunks of 2000 per query
const bulkCreateAtoms = (
  db: Database,
  indexObjects: (BulkSuccess | AtomWithDocument)[],
  chunkSize = 2000,
) => {
  for (let i = 0; i < indexObjects.length; i += chunkSize) {
    const chunk = indexObjects.slice(i, i + chunkSize);
    const placeholders = [];
    const bind = [];

    for (const atom of chunk) {
      placeholders.push("(?, ?, ?, ?)");
      if ((<IndexObjectMeta>atom).frontEndObject !== undefined) {
        const obj = atom as IndexObjectMeta;
        bind.push(
          obj.frontEndObject.kind,
          obj.frontEndObject.checksum,
          obj.frontEndObject.id,
          encodeDocumentForDB(obj.frontEndObject.data),
        );
      } else {
        const obj = atom as AtomWithDocument;
        bind.push(obj.kind, obj.checksum, obj.id, encodeDocumentForDB(obj.doc));
      }
    }

    const sql = `insert into atoms
        (kind, checksum, args, data)
          VALUES
        ${placeholders.join(",")}
        ON CONFLICT (kind, checksum, args)
        DO UPDATE SET data=excluded.data;
      `;

    db.exec({ sql, bind });
  }
};

// Insert many-to-many relationships for atoms in chunks of 2000 per query
const bulkInsertAtomMTMs = (
  db: Database,
  indexObjects: (Common | BulkSuccess)[],
  indexChecksum: Checksum,
  chunkSize = 2000,
) => {
  for (let i = 0; i < indexObjects.length; i += chunkSize) {
    const chunk = indexObjects.slice(i, i + chunkSize);
    const placeholders = [];
    const bind = [];
    for (const atom of chunk) {
      placeholders.push("(?, ?, ?, ?)");
      if ((<IndexObjectMeta>atom).frontEndObject !== undefined) {
        const obj = atom as IndexObjectMeta;
        bind.push(
          indexChecksum,
          obj.frontEndObject.kind,
          obj.frontEndObject.id,
          obj.frontEndObject.checksum,
        );
      } else {
        const obj = atom as Common;
        bind.push(indexChecksum, obj.kind, obj.id, obj.checksum);
      }
    }

    const sql = `insert into index_mtm_atoms
      (index_checksum, kind, args, checksum)
      values
      ${placeholders.join(",")}
      on conflict (index_checksum, kind, args) do update set checksum=excluded.checksum
      ;`;

    db.exec({ sql, bind });
  }
};

const insertAtomMTM = (
  db: Database,
  atom: Omit<WorkspaceAtom, "toIndexChecksum" | "fromIndexChecksum">,
  indexChecksum: Checksum,
) => {
  try {
    const bind = [indexChecksum, atom.kind, atom.id, atom.toChecksum];
    db.exec({
      sql: `insert into index_mtm_atoms
        (index_checksum, kind, args, checksum)
          VALUES
        (?, ?, ?, ?)
        on conflict (index_checksum, kind, args) do update set checksum=excluded.checksum
      ;`,
      bind,
    });
  } catch (err) {
    error("createMTM failed", atom, err);
  }
  return true;
};

/**
 * Create an index and changeset if they don't exist, and copy the previous index if we have it.
 *
 * @param db the database client
 * @param meta new (and previous) index for the changeset
 * @param span tracing span to work with
 */
const initIndexAndChangeSet = (
  db: Database,
  meta: WorkspaceAtomMeta,
  span: Span,
) => {
  const { toIndexChecksum } = {
    ...meta,
  };

  //
  // Figure out what index the change set has right now
  //
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
    span.setAttributes({
      changeSetExists,
      currentIndexChecksum,
    });
  }

  const indexQuery = db.exec({
    sql: `select checksum from indexes where checksum = ?`,
    returnValue: "resultRows",
    bind: [toIndexChecksum],
  });
  const indexExists = oneInOne(indexQuery);
  if (indexExists) span.setAttribute("indexExists", indexExists?.toString());

  if (changeSetExists && !currentIndexChecksum) {
    throw new Error("Null value from SQL, impossible");
  }

  //
  // Create the index if it doesn't exist--and copy the previous index if we have them
  //
  if (indexExists === NOROW) {
    span.setAttribute("newIndexCreated", true);
    newChangesetIndex(db, meta, currentIndexChecksum);
  }

  //
  // Create the changeset record if it doesn't exist
  //
  // TODO this is the wrong place to do this, or at least it shouldn't use the toIndexChecksum;
  // in general, we don't associate a changeset with a specific index until that index is complete!
  if (!changeSetExists) {
    span.setAttribute("changeSetCreated", true);
    db.exec({
      sql: "insert into changesets (change_set_id, workspace_id, index_checksum) VALUES (?, ?, ?);",
      bind: [meta.changeSetId, meta.workspaceId, toIndexChecksum],
    });
  }

  // Index checksum provides validation - every time MVs are generated, there's a new index checksum
  debug("✓ Index checksum validation passed", toIndexChecksum);

  return toIndexChecksum;
};

const handleIndexMvPatch = async (db: Database, msg: WorkspaceIndexUpdate) => {
  await tracer.startActiveSpan("IndexMvPatch", async (span) => {
    if (!msg.patch) {
      span.setAttribute("no_patch", true);
      return;
    }

    const data = {
      kind: msg.patch.kind,
      fromChecksum: msg.patch.fromChecksum,
      toChecksum: msg.patch.toChecksum,
      workspaceId: msg.meta.workspaceId,
      changeSetId: msg.meta.changeSetId,
      fromIndexChecksum: msg.meta.fromIndexChecksum,
      toIndexChecksum: msg.meta.toIndexChecksum,
    };
    span.setAttributes({
      ...data,
      insertedMTM: false,
      indexExists: false,
      previousIndexes: "",
    });

    if (msg.patch.kind !== EntityKind.MvIndex) {
      span.end();
      throw new Error("This is not an index patch");
    }

    // should always be present
    if (!msg.patch.fromChecksum) {
      span.end();
      error("Missing fromChecksum on MvIndex Patch", msg.patch);
      return;
    }

    // if we don't *already* have the toIndexChecksum stored
    // that means we never got the MV data we care about for this index
    // so the patch is useless to us, abort
    const indexQuery = db.exec({
      sql: `select checksum from indexes where checksum = ?`,
      returnValue: "resultRows",
      bind: [msg.meta.toIndexChecksum],
    });
    const indexNotThere = oneInOne(indexQuery) === NOROW;
    if (indexNotThere) {
      span.end();
      debug(`${msg.meta.toIndexChecksum} doesn't exist, ignoring index patch`);
      return;
    } else {
      span.setAttribute("indexExists", msg.meta.toIndexChecksum);
    }

    // if we don't have the fromChecksum, we can't patch
    // Ragnarok the world has ended, fetch the full index
    const previousIndexes = workspaceAtomExistsOnIndexes(
      db,
      msg.patch.kind,
      msg.meta.workspaceId,
      msg.patch.fromChecksum,
    );
    span.setAttribute("previousIndexes", JSON.stringify(previousIndexes));
    if (previousIndexes.length === 0) {
      span.setAttribute("ragnarok", true);
      debug(
        `Cannot patch ${msg.meta.toIndexChecksum}, atom not found at ${msg.patch.fromChecksum}: ${previousIndexes}`,
      );
      await niflheim(db, msg.meta.workspaceId, msg.meta.changeSetId);
      span.end();
      return;
    }

    // PASSED CHECKS, patch it
    const atom: Required<WorkspaceAtom> = {
      ...data,
      fromChecksum: msg.patch.fromChecksum, // duplicated for tsc
      operations: msg.patch.patch,
      id: msg.meta.workspaceId,
    };
    const patchedIndex = await patchAtom(db, atom);
    const inserted = insertAtomMTM(db, atom, msg.meta.toIndexChecksum);
    span.setAttribute("insertedMTM", inserted);
    span.end();

    // don't move any indexes on the changeset record, thats taken care of elsewhere

    // now delete any atom mtms that dont exist in the index
    const placeholders: string[] = [];
    const bind: string[] = [];
    const mvList = (patchedIndex as StoredMvIndex).mvList;
    // add the MvIndex atom to the list of atoms so we don't delete it
    const common: Common = {
      ...atom,
      checksum: msg.patch.toChecksum,
    };
    mvList.push(common);
    mvList.forEach((atom) => {
      placeholders.push("(?, ?, ?, ?)");
      bind.push(msg.meta.toIndexChecksum, atom.kind, atom.id, atom.checksum);
    });
    bind.push(msg.meta.toIndexChecksum);
    if (placeholders.length > 0) {
      const sql = `
        delete from index_mtm_atoms
        where
          (
            index_checksum,
            kind,
            args,
            checksum
          )
        NOT IN
          (
            (${placeholders.join("), (")})
          )
        AND index_checksum = ?;
      `;
      db.exec({
        sql,
        bind,
      });
    }
    span.setAttribute("mvList.length", placeholders.length);
    span.end();
  });
};

type AtomPieces = Pick<Required<WorkspaceAtom>, "kind" | "id" | "toChecksum">;
const existingWorkspaceAtoms = (db: Database, atoms: AtomPieces[]) => {
  return _existingAtoms(db, "atoms", atoms);
};

const existingDeploymentAtoms = (db: Database, atoms: AtomPieces[]) => {
  return _existingAtoms(db, "global_atoms", atoms);
};

const _existingAtoms = (db: Database, table: string, atoms: AtomPieces[]) => {
  const placeholders = [];
  const bind: string[] = [];

  for (const atom of atoms) {
    placeholders.push("(?, ?)");
    bind.push(atom.kind, atom.id);
  }

  const sql = `
    select ${table}.kind, ${table}.args, ${table}.checksum, data
    from ${table}
    where (${table}.kind, ${table}.args) in (${placeholders.join(",")})
  `;

  const rows = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });

  const result: { [key: string]: AtomWithRawData } = {};
  for (const row of rows) {
    const [kind, id, checksum, data] = row;
    const atom = {
      kind: kind as EntityKind,
      id: id as string,
      checksum: checksum as string,
      data: data as ArrayBuffer,
    };
    result[atomCacheKey(atom)] = atom;
  }

  return result;
};

type GlobalAtom = Common & {
  kind: GlobalEntity;
};
const handleDeploymentPatchMessage = async (
  db: Database,
  data: DeploymentPatchBatch,
) => {
  await tracer.startActiveSpan("DeploymentPatchBatch", async (span) => {
    span.setAttributes({
      patchKind: "workspace",
      numRawPatches: data.patches.length,
      rawPatches: JSON.stringify(data.patches),
    });

    const patches = data.patches.filter(
      (patch): patch is Required<AtomOperation> =>
        patch.fromChecksum !== undefined,
    );

    const existing = existingDeploymentAtoms(db, patches);

    const inserts: AtomWithData[] = [];
    const removals: Common[] = [];
    const modifications: AtomWithData[] = [];
    const hammers: Common[] = [];

    while (patches.length > 0) {
      const p = patches.shift();
      if (!p) break;

      // filter out no-ops based on the toChecksum
      const key = atomCacheKey({ ...p, checksum: p.toChecksum });
      if (existing[key]) continue;

      if (p.fromChecksum === "0") {
        const doc = applyOperations({}, p.patch).newDocument;
        inserts.push({ ...p, checksum: p.toChecksum, data: doc });
        continue;
      }

      if (p.toChecksum === "0") {
        removals.push({ ...p, checksum: p.toChecksum });
        continue;
      }

      const fromKey = atomCacheKey({ ...p, checksum: p.fromChecksum });
      const existingAtom = existing[fromKey];
      if (!existingAtom) {
        hammers.push({ ...p, checksum: p.toChecksum });
        continue;
      }
      const oldDoc = decodeDocumentFromDB(existingAtom.data);
      const doc = applyOperations(oldDoc, p.patch).newDocument;
      modifications.push({ ...p, data: doc, checksum: p.toChecksum });
    }

    if (inserts.length > 0) writeDeploymentAtoms(db, inserts);

    if (removals.length > 0) {
      const bind: string[] = [];
      const placeholders: string[] = [];
      removals.forEach((r) => {
        placeholders.push("(?, ?)");
        bind.push(r.kind, r.id);
      });
      db.exec({
        sql: `DELETE FROM global_atoms WHERE (kind, args) IN (${placeholders.join(
          ",",
        )})`,
        bind,
      });
    }

    if (modifications.length > 0) writeDeploymentAtoms(db, modifications);

    const kinds: GlobalEntity[] = [];
    [...modifications, ...inserts, ...removals]
      .filter((atom): atom is GlobalAtom =>
        GLOBAL_ENTITIES.includes(atom.kind as GlobalEntity),
      )
      .forEach((atom) => {
        kinds.push(atom.kind);
        bustDeployment(atom.kind, atom.id);
      });
    kinds.forEach((k) => {
      bustDeployment(k, GLOBAL_IDENTIFIER);
    });

    if (hammers.length > 0) {
      span.setAttributes({
        upToDate: false,
        numHammers: hammers.length,
      });
      const workspaceId = Object.keys(sdfClients).pop();
      if (!workspaceId) {
        error("Cannot process deployment patch, no workspace clients found");
        return;
      }
      await deploymentBulk(db, workspaceId, hammers, span);
    } else span.setAttribute("upToDate", true);
  });
};

const handleWorkspacePatchMessage = async (
  db: Database,
  data: WorkspacePatchBatch,
) => {
  await tracer.startActiveSpan("PatchBatch", async (span) => {
    const batchId = `${data.meta.toIndexChecksum}-${data.patches.length}`;
    const perfStart = performance.now();
    try {
      debug("📦 BATCH START:", batchId);

      const { changeSetId, toIndexChecksum, workspaceId, fromIndexChecksum } =
        data.meta;

      span.setAttributes({
        patchKind: "workspace",
        batchId,
        numRawPatches: data.patches.length,
        rawPatches: JSON.stringify(data.patches),
        changeSetId,
        workspaceId,
        toIndexChecksum,
        fromIndexChecksum,
      });
      debug("RAW PATCHES", batchId, data.patches.length);
      // log(changeSetId);
      if (data.patches.length === 0) {
        try {
          initIndexAndChangeSet(db, data.meta, span);
          updateChangeSetWithNewIndex(db, data.meta);
        } catch (err) {
          error("Failed to handle empty patch", data);
        }
        span.end();
        return;
      }
      // Assumption: every patch is working on the same workspace and changeset
      // (e.g. we're not bundling messages across workspaces somehow)

      if (!data.meta.toIndexChecksum) throw new Error("Expected indexChecksum");

      // Log index checksum for tracing - this provides validation at the index level
      debug(
        "📦 Processing patches with index checksum",
        data.meta.toIndexChecksum,
      );
      debug(
        "📦 Patch details:",
        data.patches.map(
          (p, i) =>
            `[${i}] ${p.kind}.${p.id}: ${p.fromChecksum} -> ${p.toChecksum}`,
        ),
      );

      let indexChecksum: string;
      try {
        indexChecksum = initIndexAndChangeSet(db, data.meta, span);
        debug("📦 Index logic completed, resolved checksum:", indexChecksum);
      } catch (err: unknown) {
        span.addEvent("error", {
          source: "initIndexAndChangeSet",
          error: err instanceof Error ? err.toString() : "unknown",
        });
        throw err;
      }

      /**
       * Patches are not coming over the wire in any meaningful
       * order, which means they can be inter-dependent e.g. an item in
       * a list can be _after_ the list that wants it.
       * This causes an unnecessary hammer by the list when its cache busts
       * it doesn't have the item on the read.
       *
       * BUT NOW, we're not busting on a list (other than a hammer)
       * So we can do the lists first, which fixes the add/remove behavior
       * for postProcessing
       */

      const atoms = data.patches
        .filter((rawAtom) => !IGNORE_LIST.has(rawAtom.kind))
        .map((rawAtom) => {
          const atom: WorkspaceAtom = {
            ...rawAtom,
            ...data.meta,
            operations: rawAtom.patch,
          };
          return atom;
        })
        .filter(
          (rawAtom): rawAtom is Required<WorkspaceAtom> =>
            !!rawAtom.fromChecksum && !!rawAtom.operations,
        );

      span.setAttribute("numAtoms", atoms.length);
      if (!indexChecksum) {
        throw new Error(
          `Expected index checksum for ${data.meta.toIndexChecksum}`,
        );
      }

      const existingAtoms = existingWorkspaceAtoms(db, atoms);
      const handlePatchBatch = async (
        workspaceAtoms: Required<WorkspaceAtom>[],
      ) => {
        const ops = workspaceAtoms.map((atom) =>
          preprocessPatch(atom, existingAtoms),
        );
        return await handlePatchOperations(
          db,
          workspaceId,
          changeSetId,
          indexChecksum,
          ops,
          span,
        );
      };

      const atomsToBust = await handlePatchBatch(atoms);

      const listAtomsToBust = atomsToBust.filter((a) =>
        LISTABLE.includes(a.kind),
      );
      const nonListAtomsToBust = atomsToBust.filter(
        (a) =>
          !LISTABLE.includes(a.kind) &&
          a.kind !== EntityKind.IncomingConnections,
      );
      const connAtomsToBust = atomsToBust.filter(
        (a) => a.kind === EntityKind.IncomingConnections,
      );

      updateChangeSetWithNewIndex(db, data.meta);
      span.setAttribute("updatedWithNewIndex", true);
      removeOldIndex(db, span);

      nonListAtomsToBust.forEach((atom) => {
        bustCacheAndReferences(
          db,
          workspaceId,
          changeSetId,
          atom.kind,
          atom.id,
        );
      });

      listAtomsToBust.forEach((atom) => {
        if (atom && atom.kind === EntityKind.ViewComponentList) {
          bustCacheAndReferences(
            db,
            workspaceId,
            changeSetId,
            atom.kind,
            atom.id,
          );
        }
      });

      connAtomsToBust.forEach((atom) => {
        bustCacheAndReferences(
          db,
          workspaceId,
          changeSetId,
          atom.kind,
          atom.id,
        );
      });
    } finally {
      // this always runs regardless of return, throw, etc
      debug("BATCH END", batchId, "took", performance.now() - perfStart, "ms");

      span.end();
    }
  });
};

interface PatchOperation {
  kind: "NoOp" | "Create" | "Remove" | "Patch";
  atom: WorkspaceAtom;
}

const handlePatchOperations = async (
  db: Database,
  workspaceId: WorkspacePk,
  changeSetId: ChangeSetId,
  indexChecksum: Checksum,
  patchOperations: PatchOperation[],
  span: Span,
): Promise<Common[]> => {
  const realHammers: MjolnirBulk = [];

  // No-op, just post-process
  const noops: Common[] = patchOperations
    .filter((op) => op.kind === "NoOp")
    .map((op) => ({
      kind: op.atom.kind,
      id: op.atom.id,
      checksum: op.atom.toChecksum,
    }));
  const bulkMtmStart = performance.now();
  bulkInsertAtomMTMs(db, noops, indexChecksum);
  span.setAttribute(
    "performance.NoopBulkMtm",
    performance.now() - bulkMtmStart,
  );

  const atomsToInsert: AtomWithDocument[] = [];
  const creates = patchOperations
    .filter((op) => op.kind === "Create")
    .map((op) => op.atom);
  for (const atom of creates) {
    try {
      // A create means the operation set will be equal to the entire document for the atom
      const doc = atom.operations
        ? applyOperations({}, atom.operations).newDocument
        : {};

      atomsToInsert.push({
        kind: atom.kind,
        id: atom.id,
        checksum: atom.toChecksum,
        doc,
      });
    } catch (err) {
      error("Failed to apply create operations for patch", atom, err);
    }
  }

  interface CommonWithOps extends Common {
    operations?: Operation[];
  }

  const patches = patchOperations
    .filter((op) => op.kind === "Patch")
    .map((op) => op.atom);
  const atomsToUpdate: CommonWithOps[] = [];
  const atomsByKindAndId: { [key: string]: WorkspaceAtom } = {};
  for (const atom of patches) {
    if (atom.fromChecksum) {
      atomsToUpdate.push({
        kind: atom.kind,
        id: atom.id,
        checksum: atom.fromChecksum,
        operations: atom.operations,
      });

      atomsByKindAndId[`${atom.id}-${atom.kind}-${atom.fromChecksum}`] = atom;
    }
  }

  const startDocs = performance.now();
  const { existingDocuments, hammers } = atomDocumentsForChecksums(
    db,
    atomsToUpdate,
  );
  span.setAttribute(
    "performance.atomDocumentsForChecksums",
    performance.now() - startDocs,
  );

  // Apply patches for every atom we could find
  for (const atomToPatch of existingDocuments) {
    const atomKey = `${atomToPatch.id}-${atomToPatch.kind}-${atomToPatch.checksum}`;
    const maybeOperations = atomsByKindAndId[atomKey]?.operations;
    const toChecksum = atomsByKindAndId[atomKey]?.toChecksum;
    if (!toChecksum) {
      error("Patch missing toChecksum, skipping", atomToPatch);
      continue;
    }

    const beforeDoc = atomToPatch.doc ?? {};

    try {
      let afterDoc;
      if (maybeOperations && beforeDoc) {
        afterDoc = applyOperations(beforeDoc, maybeOperations).newDocument;
      }

      atomsToInsert.push({
        kind: atomToPatch.kind,
        id: atomToPatch.id,
        checksum: toChecksum,
        doc: afterDoc,
      });
    } catch (err) {
      error("Failed to apply patch operations", err);
    }
  }

  // Ok we have all the patches we could apply, insert them into the database
  if (atomsToInsert.length > 0) {
    const startCreate = performance.now();
    bulkCreateAtoms(db, atomsToInsert);
    span.setAttribute(
      "performance.bulkCreateAtoms",
      performance.now() - startCreate,
    );

    const startMtm = performance.now();
    bulkInsertAtomMTMs(db, atomsToInsert, indexChecksum);
    span.setAttribute(
      "performance.bulkCreateMtms",
      performance.now() - startMtm,
    );
  }

  // Now process removals
  const removals = patchOperations
    .filter((op) => op.kind === "Remove" && op.atom.fromChecksum)
    .map((op) => ({
      kind: op.atom.kind,
      id: op.atom.id,
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      checksum: op.atom.fromChecksum!,
    }));
  for (const atom of removals) {
    try {
      const doc = atomDocumentForChecksum(
        db,
        atom.kind,
        atom.id,
        atom.checksum,
      );

      removeAtom(db, indexChecksum, atom.kind, atom.id, atom.checksum);
      postProcess(
        db,
        workspaceId,
        changeSetId,
        atom.kind,
        doc,
        atom.id,
        indexChecksum,
        true,
        true,
      );
    } catch (err) {
      error("Failed to remove atom", err);
    }
  }

  // moving postProcess to after all the patching has been completed
  const startPost = performance.now();
  for (const atom of noops) {
    try {
      const doc = atomDocumentForChecksum(
        db,
        atom.kind,
        atom.id,
        atom.checksum,
      );

      postProcess(
        db,
        workspaceId,
        changeSetId,
        atom.kind,
        doc,
        atom.id,
        indexChecksum,
        false,
        true,
      );
    } catch (err) {
      error("Failed to apply NoOp patch", err, atom);
    }
  }

  for (const atom of atomsToInsert) {
    try {
      postProcess(
        db,
        workspaceId,
        changeSetId,
        atom.kind,
        atom.doc,
        atom.id,
        indexChecksum,
        false,
        true,
      );
    } catch (err) {
      error("Failed to post process atom", atom, error);
    }
  }
  span.setAttribute("performance.postProcess", performance.now() - startPost);

  // Throw hammers for the ones we couldn't find
  try {
    for (const hammer of hammers) {
      const atomKey = `${hammer.id}-${hammer.kind}-${hammer.checksum}`;
      const toChecksum = atomsByKindAndId[atomKey]?.toChecksum;
      if (toChecksum) {
        realHammers.push({
          ...hammer,
          checksum: toChecksum,
        });
      }
    }

    if (realHammers.length > 0) {
      await mjolnirBulk(
        db,
        workspaceId,
        changeSetId,
        realHammers,
        indexChecksum,
      );
    }
  } catch (err) {
    error("Failed to throw hammers during patch", err);
  }

  return [...atomsToInsert, ...removals, ...noops, ...realHammers];
};

const preprocessPatch = (
  atom: Required<WorkspaceAtom>,
  existingAtoms: { [key: string]: Common },
): PatchOperation => {
  const finishedAtomAsCommon: Common = {
    kind: atom.kind,
    id: atom.id,
    checksum: atom.toChecksum,
  };
  const finishedKey = atomCacheKey(finishedAtomAsCommon);

  // Does the atom already exist? Then it's a no-op, but process and insert the mtm
  if (
    atom.toChecksum &&
    atom.toChecksum !== "0" &&
    typeof existingAtoms[finishedKey] !== "undefined"
  ) {
    return {
      kind: "NoOp",
      atom,
    };
  }

  // atom needs to be created, the operations will construct the entire document
  if (atom.fromChecksum === "0") {
    return {
      kind: "Create",
      atom,
    };
  }
  // Atom is being removed
  else if (atom.toChecksum === "0") {
    return {
      kind: "Remove",
      atom,
    };
  }
  // Not being removed? it's being patched! We will detect if we need to throw a
  // hammer for it later
  else {
    return {
      kind: "Patch",
      atom,
    };
  }
};

const patchAtom = async (
  db: Database,
  atom: Required<WorkspaceAtom>,
  span?: Span,
) => {
  const start = performance.now();
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
  const end = performance.now();
  span?.setAttribute("perf.patchAtom", end - start);
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
    ON CONFLICT (kind, checksum, args) DO UPDATE SET data=excluded.data
    ;`,
    bind: [atom.kind, atom.id, atom.toChecksum, encodeDocumentForDB(afterDoc)],
  });
  return afterDoc;
};

interface BulkSuccess {
  workspaceSnapshotAddress: string;
  frontEndObject: AtomWithData;
  indexChecksum: string;
}
type BulkResponse = { successful: BulkSuccess[]; failed: MjolnirBulk[] };
const mjolnirBulk = async (
  db: Database,
  workspaceId: string,
  changeSetId: ChangeSetId,
  objs: MjolnirBulk,
  indexChecksum: string,
) => {
  debug("🔨 BULK MJOLNIR:", objs.length, objs);

  // We might already have these cached in memory
  const cachedAtoms: AtomWithDocument[] = [];
  const hammerObjs: MjolnirBulk = [];
  for (const obj of objs) {
    if (obj.checksum) {
      const doc = getCachedDocument(obj);
      if (doc) {
        cachedAtoms.push({
          id: obj.id,
          kind: obj.kind as EntityKind,
          checksum: obj.checksum,
          doc,
        });
      } else {
        hammerObjs.push(obj);
      }
    } else {
      hammerObjs.push(obj);
    }
  }

  bulkCreateAtoms(db, cachedAtoms);
  bulkInsertAtomMTMs(db, cachedAtoms, indexChecksum);

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

  hammerObjs.forEach((o) => {
    inFlightFn(changeSetId, `${o.kind}.${o.id}`);
  });

  await tracer.startActiveSpan(`GET ${desc}`, async (span) => {
    const sdf = getSdfClientForWorkspace(workspaceId, span);
    if (!sdf) {
      span.end();
      return;
    }

    span.setAttributes({
      workspaceId,
      changeSetId,
      indexChecksum,
      numHammers: hammerObjs.length,
    });
    const startBulkMjolnirReq = performance.now();
    req = await sdf<BulkResponse>({
      method: "post",
      url,
      data: { requests: hammerObjs },
    });
    if (req.status !== 200) {
      span.setAttribute("http.status", req.status);
      debug("🔨 MJOLNIR HTTP:", req.status, indexChecksum);
      error("MJOLNIR", req.status, url, hammerObjs);
    } else {
      debug(
        "🔨 MJOLNIR BULK HTTP SUCCESS:",
        indexChecksum,
        `${performance.now() - startBulkMjolnirReq}ms`,
      );
      span.setAttributes({
        successful: req.data.successful.length,
        failed: req.data.failed.length,
      });
    }
    if (req?.status) span.setAttribute("http.status", req.status);
    span.end();
  });

  if (!req || req.status !== 200) {
    debug("🔨 MJOLNIR BULK FAILED:", indexChecksum, "no response");
    bulkDone({ workspaceId, changeSetId }, true);
    return;
  }

  const startWriteToSql = performance.now();

  const first = req.data.successful.shift();
  if (!first) {
    debug("🔨 MJOLNIR BULK NO FIRST?:", req.data.successful.length);
    return;
  }
  const msg: WorkspaceAtomMessage = {
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
  returnedFn(
    changeSetId,
    `${first.frontEndObject.kind}.${first.frontEndObject.id}`,
  );
  await handleHammer(db, msg);

  bulkCreateAtoms(db, req.data.successful);
  bulkInsertAtomMTMs(db, req.data.successful, indexChecksum);

  for (const obj of req.data.successful) {
    returnedFn(
      changeSetId,
      `${obj.frontEndObject.kind}.${obj.frontEndObject.id}`,
    );

    postProcess(
      db,
      workspaceId,
      changeSetId,
      obj.frontEndObject.kind,
      obj.frontEndObject.data,
      obj.frontEndObject.id,
      indexChecksum,
    );

    bustCacheAndReferences(
      db,
      workspaceId,
      changeSetId,
      obj.frontEndObject.kind,
      obj.frontEndObject.id,
      false,
      true,
    );
  }

  const writeToSqlMs = performance.now() - startWriteToSql;
  debug(`🔨 MJOLNIR BULK DONE! ${writeToSqlMs}ms`);
  bulkDone({ workspaceId, changeSetId });
};

const deploymentMjolnir = async (
  db: Database,
  workspaceId: string,
  kind: GlobalEntity,
  id: Id,
) => {
  const pattern = [
    "v2",
    "workspaces",
    { workspaceId },
    "mjolnir",
  ] as URLPattern;
  const [url, desc] = describePattern(pattern);
  const params = { kind, id };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let req: undefined | AxiosResponse<AtomWithData, any>;

  await tracer.startActiveSpan(`GET ${desc}`, async (span) => {
    const sdf = getSdfClientForWorkspace(workspaceId, span);
    if (!sdf) {
      span.end();
      return;
    }
    span.setAttributes({ workspaceId, kind, id });
    try {
      req = await sdf<AtomWithData>({
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

  if (!req || req.status !== 200) return;

  const { checksum, data } = req.data;
  writeDeploymentAtoms(db, [{ checksum, data, kind, id }]);
};

const mjolnir = (
  db: Database,
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: EntityKind,
  id: Id,
  checksum?: Checksum,
) => {
  if (IGNORE_LIST.has(kind)) return;

  const atomKey = `${kind}.${id}`;
  debug("🔨 MJOLNIR REQUESTED:", atomKey, "checksum:", checksum);

  maybeMjolnir({ workspaceId, changeSetId, kind, id }, async () => {
    debug("🔨 MJOLNIR FIRING:", atomKey);
    inFlightFn(changeSetId, `${kind}.${id}`);
    // NOTE: since we're moving to all weak refs
    // storing the index becomes useful here, we can lookup the
    // checksum we would expect to be returned, and see if we have it already
    if (!checksum) {
      return mjolnirJob(workspaceId, changeSetId, kind, id, checksum);
    }

    // these are sent after patches are completed
    // double check that i am still necessary!
    const exists = workspaceAtomExistsOnIndexes(db, kind, id, checksum);
    if (exists.length === 0) {
      return mjolnirJob(workspaceId, changeSetId, kind, id, checksum);
    } // if i have it, bust!
    else
      bustCacheAndReferences(
        db,
        workspaceId,
        changeSetId,
        kind,
        id,
        false,
        true,
      );
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
    const sdf = getSdfClientForWorkspace(workspaceId, span);
    if (!sdf) {
      span.end();
      return;
    }
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

  returnedFn(changeSetId, `${kind}.${id}`);
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

  const msg: WorkspaceAtomMessage = {
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
  processMjolnirQueue.add(
    async () =>
      sqlite &&
      (await sqlite.transaction(async (db) => await handleHammer(db, msg))),
  );
};

const updateChangeSetWithNewIndex = (
  db: Database,
  meta: Omit<WorkspaceAtomMeta, "fromIndexChecksum" | "workspaceId">,
) => {
  db.exec({
    sql: "update changesets set index_checksum = ? where change_set_id = ?;",
    bind: [meta.toIndexChecksum, meta.changeSetId],
  });
};

const removeOldIndex = async (_db: Database, _span: Span) => {
  return;
  // Keep the last 5 indexes per changeset for debugging purposes
  // This helps track previous session checksums
  // const deleteIndexes = db.exec({
  //   sql: `
  //     DELETE FROM indexes
  //     WHERE checksum NOT IN (
  //       SELECT index_checksum FROM changesets
  //     )
  //     RETURNING *;
  //   `,
  //   returnValue: "resultRows",
  // });

  // Only delete atoms that aren't referenced by any index (including retained ones)
  // const deleteAtoms = db.exec({
  //   sql: `
  //     DELETE FROM atoms
  //     WHERE (kind, args, checksum) NOT IN (
  //       SELECT kind, args, checksum FROM index_mtm_atoms
  //     ) returning atoms.kind, atoms.args, atoms.checksum;
  //   `,
  //   returnValue: "resultRows",
  // });

  // span.setAttributes({
  //   deletedIndexes: JSON.stringify(deleteIndexes),
  //   deletedAtoms: JSON.stringify(deleteAtoms),
  // });

  // if (deleteIndexes.length > 0) {
  //   debug(
  //     "🗑️ Cleaned up",
  //     deleteIndexes.length,
  //     "old indexes (keeping recent 5 per workspace)",
  //     deleteIndexes,
  //   );
  // }
  // if (deleteAtoms.length > 0) {
  //   debug("🗑️ Cleaned up", deleteAtoms.length, "orphaned atoms", deleteAtoms);
  // }
};

const pruneAtomsForClosedChangeSet = async (
  db: Database,
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
    removeOldIndex(db, span);
    span.end();
  });
};

// 128k atom documents
const MAX_CACHE_SIZE = 65536 * 2;
const decodedAtomCache = new QuickLRU<string, AtomDocument>({
  maxSize: MAX_CACHE_SIZE,
});

const atomCacheKey = (atom: Common) =>
  `${atom.id}-${atom.kind}-${atom.checksum}`;

const getCachedDocument = (atom: Common) => {
  const cacheKey = atomCacheKey(atom);
  return _.cloneDeep(decodedAtomCache.get(cacheKey));
};

const setCachedDocument = (atom: Common, data: AtomDocument) => {
  const cacheKey = atomCacheKey(atom);
  decodedAtomCache.set(cacheKey, _.cloneDeep(data));
};

const atomDocumentForChecksum = (
  db: Database,
  kind: EntityKind,
  id: string,
  checksum: string,
): AtomDocument | undefined => {
  const atom = { kind, id, checksum };
  const maybeCachedAtom = getCachedDocument(atom);
  if (maybeCachedAtom) {
    return maybeCachedAtom;
  }

  const rows = db.exec({
    sql: `select atoms.data from atoms where atoms.kind = ? AND atoms.args = ? and atoms.checksum = ? limit 1;`,
    bind: [kind, id, checksum],
    returnValue: "resultRows",
  });

  const atomData = rows[0]?.[0];
  if (atomData) {
    const decoded = decodeDocumentFromDB(atomData as ArrayBuffer);
    setCachedDocument(atom, decoded);
    return decoded;
  }

  return undefined;
};

interface AtomWithDocument extends Common {
  doc: AtomDocument;
}

interface AtomWithRawData extends Common {
  data: ArrayBuffer;
}

const atomDocumentsForChecksums = (
  db: Database,
  atoms: Common[],
): {
  existingDocuments: AtomWithDocument[];
  hammers: Common[];
} => {
  const existingAtoms = new Map<string, AtomWithDocument>();
  const uncachedAtoms = new Map<string, Common>();

  const placeholders = [];
  const bind: string[] = [];

  for (const atom of atoms) {
    placeholders.push("(?, ?, ?)");
    bind.push(atom.kind, atom.id, atom.checksum);
  }

  const sql = `
    select atoms.kind, atoms.args, atoms.checksum, atoms.data
    from atoms
    where (atoms.kind, atoms.args, atoms.checksum) in (${placeholders.join(
      ",",
    )})
  `;

  const rows = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });

  for (const row of rows) {
    const [kind, id, checksum, data] = row;
    if (data) {
      const atom = {
        kind: kind as EntityKind,
        id: id as string,
        checksum: checksum as string,
      };
      const key = atomCacheKey(atom);
      const maybeCached = getCachedDocument(atom);
      const doc = maybeCached ?? decodeDocumentFromDB(data as ArrayBuffer);
      setCachedDocument(atom, doc);
      existingAtoms.set(key, { ...atom, doc });
    }
  }

  for (const atom of atoms) {
    const key = atomCacheKey(atom);
    if (!existingAtoms.has(key)) {
      uncachedAtoms.set(key, atom);
    }
  }

  return {
    existingDocuments: Array.from(existingAtoms.values()),
    hammers: Array.from(uncachedAtoms.values()),
  };
};

interface AtomWithArrayBuffer extends Common {
  data: ArrayBuffer;
}
const atomsForChangeSet = (
  db: Database,
  indexChecksum: string,
): AtomWithArrayBuffer[] => {
  const rows = db.exec({
    sql: `
    select atoms.kind, atoms.args, atoms.checksum, atoms.data
    from atoms
    inner join index_mtm_atoms mtm
      ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
    inner join indexes ON mtm.index_checksum = indexes.checksum
    WHERE indexes.checksum = ?
    ;
    `,
    bind: [indexChecksum],
    returnValue: "resultRows",
  });
  return rows.map((row) => ({
    kind: row[0] as EntityKind,
    id: row[1] as string,
    checksum: row[2] as string,
    data: row[3] as ArrayBuffer,
  }));
};

/**
 * LIFECYCLE EVENTS
 */

export const CHANGE_SET_INDEX_URL = (
  workspaceId: string,
  changeSetId: string,
) =>
  [
    "v2",
    "workspaces",
    { workspaceId },
    "change-sets",
    { changeSetId },
    "index",
  ] as URLPattern;

export const DEPLOYMENT_INDEX_URL = (workspaceId: string) =>
  ["v2", "workspaces", { workspaceId }, "deployment_index"] as URLPattern;

export const STATUS_INDEX_IN_PROGRESS = 202;

const getSdfClientForWorkspace = (workspaceId: string, span?: Span) => {
  const sdf = sdfClients[workspaceId];

  if (!sdf) {
    const errorMessage = `SDF client not found for workspace: ${workspaceId}`;
    error(errorMessage);
    span?.addEvent("error", {
      "error.message": errorMessage,
    });
  }

  return sdf;
};

const sleep = async (ms: number) => {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
};

type AxiosFn<T> = () => Promise<AxiosResponse<T>>;
const ONE_MIN = 1000 * 60;
const MAX_RETRY = 4;
const retry = async <T>(
  fn: AxiosFn<T>,
  retryNum?: number,
): Promise<AxiosResponse<T> | undefined> => {
  let r;
  try {
    r = await fn();
  } catch (err) {
    // only handling axios errors
    if (!(Axios.isAxiosError(err) && err.response)) return r;

    if ((retryNum ?? 0) >= MAX_RETRY) {
      return err.response;
    }
    // dont retry on 404s, those are being handled with `IndexUpdate` messages
    if (err.response.status >= 500) {
      retryNum = retryNum ? retryNum + 1 : 1;
      const ms = retryNum ** 2 * 2000;
      await sleep(Math.min(ms, ONE_MIN));
      return retry(fn, retryNum);
    }
    return err.response;
  }
  return r;
};

type DeploymentBulkResponse = {
  successful: AtomWithData[];
  failed: MjolnirBulk[];
};
const deploymentBulk = async (
  db: Database,
  workspaceId: string,
  hammerObjs: Common[],
  span: Span,
) => {
  const bulkPattern = [
    "v2",
    "workspaces",
    { workspaceId },
    "multi_mjolnir",
  ] as URLPattern;
  const [bulkUrl, bulkDesc] = describePattern(bulkPattern);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let bulkReq: undefined | AxiosResponse<DeploymentBulkResponse, any>;
  await tracer.startActiveSpan(`GET ${bulkDesc}`, async (span) => {
    const sdf = getSdfClientForWorkspace(workspaceId, span);
    if (!sdf) {
      span.end();
      return;
    }

    span.setAttributes({
      workspaceId,
      numHammers: hammerObjs.length,
    });
    const startBulkMjolnirReq = performance.now();
    bulkReq = await sdf<DeploymentBulkResponse>({
      method: "post",
      url: bulkUrl,
      data: { requests: hammerObjs },
    });
    if (bulkReq.status !== 200) {
      span.setAttribute("http.status", bulkReq.status);
      debug("🔨 DEPLOYMENT MJOLNIR HTTP:", bulkReq.status);
      error("MJOLNIR", bulkReq.status, bulkUrl, hammerObjs);
    } else {
      debug(
        "🔨 DEPLOYMENT MJOLNIR BULK HTTP SUCCESS:",
        `${performance.now() - startBulkMjolnirReq}ms`,
      );
      span.setAttributes({
        successful: bulkReq.data.successful.length,
        failed: bulkReq.data.failed.length,
      });
    }
    if (bulkReq?.status) span.setAttribute("http.status", bulkReq.status);
    span.end();
  });

  if (!bulkReq || bulkReq.status !== 200) {
    debug("🔨 DEPLOYMENT MJOLNIR BULK FAILED:", "no response");
    return false;
  }

  const startWriteToSql = performance.now();
  writeDeploymentAtoms(db, bulkReq.data.successful);
  const writeToSqlMs = performance.now() - startWriteToSql;
  span.setAttributes({
    "performance.deploymentBulkSqlWrite": writeToSqlMs,
  });
  debug(`🔨 DEPLOYMENT MJOLNIR BULK DONE! ${writeToSqlMs}ms`);
};

/**
 * This is a coldstart for the "global" / deployment MVs
 * that are not workspace / changeset specific.
 */
const vanaheim = async (
  db: Database,
  workspaceId: string,
): Promise<boolean> => {
  return await tracer.startActiveSpan("vanaheim", async (span: Span) => {
    span.setAttributes({ workspaceId });
    const sdf = getSdfClientForWorkspace(workspaceId, span);
    if (!sdf) {
      span.end();
      return false;
    }

    const pattern = DEPLOYMENT_INDEX_URL(workspaceId);
    const [url, desc] = describePattern(pattern);
    const frigg = tracer.startSpan(`GET ${desc}`);
    frigg.setAttributes({ workspaceId });
    const req = await retry<IndexObjectMeta>(async () => {
      const req = await sdf<IndexObjectMeta>({
        method: "get",
        url,
      });
      return req;
    });

    if (!req || req?.status >= 500) {
      frigg.setAttribute("indexFailure", true);
      frigg.setAttribute("http.status_code", req?.status ?? 500);
      return false;
    }
    frigg.setAttribute("status", req.status);
    if (req.status === STATUS_INDEX_IN_PROGRESS) {
      debug("‼️ DEPLOYMENT INDEX NOT READY");
      frigg.end();
      span.end();
      return false;
    }

    const atoms = req.data.frontEndObject.data.mvList;
    const existingAtoms = new Map<string, Common>();
    const uncachedAtoms = new Map<string, Common>();

    const placeholders = [];
    const bind: string[] = [];

    for (const atom of atoms) {
      placeholders.push("(?, ?, ?)");
      bind.push(atom.kind, atom.id, atom.checksum);
    }

    const sql = `
      select global_atoms.kind, global_atoms.args, global_atoms.checksum
      from global_atoms
      where (global_atoms.kind, global_atoms.args, global_atoms.checksum) in (${placeholders.join(
        ",",
      )})
    `;

    const rows = db.exec({
      sql,
      bind,
      returnValue: "resultRows",
    });

    for (const row of rows) {
      const [kind, id, checksum] = row;
      const atom = {
        kind: kind as EntityKind,
        id: id as string,
        checksum: checksum as string,
      };
      const key = atomCacheKey(atom);
      existingAtoms.set(key, { ...atom });
    }

    for (const atom of atoms) {
      const key = atomCacheKey(atom);
      if (!existingAtoms.has(key)) {
        uncachedAtoms.set(key, atom);
      }
    }

    const hammerObjs = [...uncachedAtoms.values()];
    if (hammerObjs.length === 0) {
      span.setAttribute("upToDate", true);
      return true;
    }
    span.setAttribute("upToDate", false);

    await deploymentBulk(db, workspaceId, hammerObjs, span);

    return true;
  });
};

const writeDeploymentAtoms = (db: Database, atoms: AtomWithData[]) => {
  const bind: Array<string | Uint8Array> = [];
  const placeholders: string[] = [];
  atoms.forEach((atom) => {
    bind.push(
      atom.kind,
      atom.checksum,
      atom.id,
      encodeDocumentForDB(atom.data),
    );
    placeholders.push("(?, ?, ?, ?)");
  });

  const sql = `insert into global_atoms
    (kind, checksum, args, data)
      VALUES
    ${placeholders.join(",")}
    ON CONFLICT (kind, args)
    DO UPDATE SET checksum=excluded.checksum, data=excluded.data;
  `;

  db.exec({ sql, bind });
};

const niflheim = async (
  db: Database,
  workspaceId: string,
  changeSetId: ChangeSetId,
): Promise<-1 | 0 | 1> => {
  // NOTE, we are using this integer return type because we can't handle exceptions over the thread boundary
  return await tracer.startActiveSpan("niflheim", async (span: Span) => {
    span.setAttributes({ workspaceId, changeSetId });
    const sdf = getSdfClientForWorkspace(workspaceId, span);
    if (!sdf) {
      span.end();
      // don't make this a -1, b/c if the user can't open this workspace for auth reasons
      // they shouldn't get past this step...
      return 0;
    }

    // build connections list based on data we have in the DB
    // connections list will rebuild as data comes in
    bulkInflight({ workspaceId, changeSetId });

    // clear out references, no queries have been performed yet
    clearAllWeakReferences(db, changeSetId);

    const pattern = CHANGE_SET_INDEX_URL(workspaceId, changeSetId);

    const [url, desc] = describePattern(pattern);
    const frigg = tracer.startSpan(`GET ${desc}`);
    frigg.setAttributes({ workspaceId, changeSetId });
    const req = await retry<IndexObjectMeta>(async () => {
      const req = await sdf<IndexObjectMeta>({
        method: "get",
        url,
      });
      return req;
    });
    if (!req || req?.status >= 500) {
      frigg.setAttribute("indexFailure", true);
      // could be any 5XX, but, we'll go with this for now
      frigg.setAttribute("http.status_code", req?.status ?? 500);
      frigg.end();
      span.end();
      return -1;
    }

    // Check for 202 status - user needs to go to lobby
    frigg.setAttribute("status", req.status);
    if (req.status === STATUS_INDEX_IN_PROGRESS || req.status === 404) {
      debug("‼️ INDEX NOT READY", changeSetId);
      frigg.end();
      span.end();
      return 0;
    }

    // Use index checksum for validation - this is more reliable than snapshot addresses
    const indexChecksum = req.data.indexChecksum;
    const atoms = req.data.frontEndObject.data.mvList.filter(
      (atom) => !IGNORE_LIST.has(atom.kind),
    );
    const meta = {
      changeSetId,
      workspaceId,
      toIndexChecksum: indexChecksum,
      fromIndexChecksum: indexChecksum,
    };
    initIndexAndChangeSet(db, meta, frigg);
    debug("niflheim atom count", atoms.length);
    frigg.setAttribute("numEntries", atoms.length);
    frigg.setAttribute("indexChecksum", indexChecksum);
    frigg.end();

    debug("🔍 Index checksum validation", indexChecksum);

    // Compare each atom checksum from the index with local checksums
    const hammerObjs: MjolnirBulk = [];

    // Gather up a set of all atoms for detecting atoms to remove
    const atomSet = new Set();
    for (const atom of atoms) {
      atomSet.add(atomCacheKey(atom));
    }

    // Insert all atoms into the database, or throw hammers for them
    const chunkSize = 2000;
    for (let i = 0; i < atoms.length; i += chunkSize) {
      const chunk = atoms.slice(i, i + chunkSize);
      const { existingDocuments, hammers } = atomDocumentsForChecksums(
        db,
        chunk,
      );
      bulkInsertAtomMTMs(db, existingDocuments, indexChecksum, chunkSize);
      hammerObjs.push(...hammers);
    }

    // Now that all atoms have been inserted, refetch all atoms currently in the change set
    const finalAtoms = atomsForChangeSet(db, indexChecksum);
    const atomsToUnlink: Array<Common> = [];

    const processAtom = async (atom: AtomWithArrayBuffer) => {
      let doc = getCachedDocument(atom);
      if (!doc) {
        doc = decodeDocumentFromDB(atom.data);
        setCachedDocument(atom, doc);
      }

      postProcess(
        db,
        workspaceId,
        changeSetId,
        atom.kind,
        doc,
        atom.id,
        indexChecksum,
        false,
        false,
        false,
      );

      bustCacheAndReferences(
        db,
        workspaceId,
        changeSetId,
        atom.kind,
        atom.id,
        false,
        true,
      );
    };

    for (const atom of finalAtoms) {
      // Atom is in the database, but not in the index? Delete it
      if (!atomSet.has(atomCacheKey(atom))) {
        atomsToUnlink.push(atom);
      } else {
        // Placing this in a promise to yield control back to the event loop
        await processAtom(atom);
      }
    }

    span.setAttribute("numUnlink", atomsToUnlink.length);
    span.setAttribute("numHammers", hammerObjs.length);
    span.setAttribute("indexChecksum", indexChecksum);

    if (atomsToUnlink.length > 0) {
      // We are not awaiting this promise so that we can continue forward since we don't
      // need to see the result
      bulkRemoveAtoms(db, atomsToUnlink, indexChecksum);
    }

    // store the MvIndex itself
    const mvAtom = {
      workspaceId,
      changeSetId,
      id: workspaceId,
      kind: EntityKind.MvIndex,
      toChecksum: indexChecksum,
    };
    await createAtom(db, mvAtom, req.data.frontEndObject.data);
    insertAtomMTM(db, mvAtom, indexChecksum);

    // link the checksum to the change set (just in case its not done in init)
    updateChangeSetWithNewIndex(db, meta);

    // Now to deal with all the atoms we don't have present. Throw the big hammer.
    if (hammerObjs.length > 0) {
      await mjolnirBulk(
        db,
        workspaceId,
        changeSetId,
        hammerObjs,
        indexChecksum,
      );
    } else {
      debug("NIFLHEIM NOOP DONE", changeSetId);
      bulkDone({ workspaceId, changeSetId }, true);
      span.setAttribute("noop", true);
    }

    span.end();
    return 1;
  });
};

const ragnarok = async (
  db: Database,
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
  await pruneAtomsForClosedChangeSet(db, workspaceId, changeSetId);
  if (!noColdStart) {
    // call for a cold start to re-populate
    await niflheim(db, workspaceId, changeSetId);
  }
};

/**
 * WEAK REFERENCE TRACKING
 */

const clearAllWeakReferences = (db: Database, changeSetId: string) => {
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

const _clearWeakReferences = (
  db: Database,
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

const weakReference = (
  db: Database,
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
 * COMPUTED IMPLEMENTATIONS
 */
const COMPUTED_KINDS: EntityKind[] = [
  EntityKind.AttributeTree,
  EntityKind.IncomingConnections,
  EntityKind.ManagementConnections,
  EntityKind.Component,
];

const defaultSubscriptions = new DefaultMap<ChangeSetId, DefaultSubscriptions>(
  () => ({
    defaultSubscriptions: new Map(),
    componentsForSubs: new DefaultMap(() => new Set()),
    subsForComponents: new DefaultMap(() => new Set()),
  }),
);

// A mapping of possible connections per component, per change set
const possibleConns = new DefaultMap<
  ChangeSetId,
  DefaultMap<ComponentId, Record<string, PossibleConnection>>
>(() => new DefaultMap(() => ({})));

// the `string` is `${toAttributeValueId}-${fromAttributeValueId}`
const allOutgoingConns = new DefaultMap<
  ChangeSetId,
  DefaultMap<ComponentId, Record<string, Connection>>
>(() => new DefaultMap(() => ({})));

// the `string` is `${toComponentId}-${fromComponentId}`
const allIncomingMgmt = new DefaultMap<
  ChangeSetId,
  DefaultMap<ComponentId, Record<string, Connection>>
>(() => new DefaultMap(() => ({})));

// Given a single Av, process all the data related to default subscriptions (is
// it a default sub, does it have external sources?)
const processAvForDefaultSubscriptions = (
  changeSetId: ChangeSetId,
  componentId: ComponentId,
  av: AttributeValue,
  defaultSubsForComponent: Set<string>,
) => {
  const defaultSubKey: DefaultSubscription = {
    componentId,
    path: av.path,
  };

  const defaultSubKeyString = JSON.stringify(defaultSubKey);

  const defaultSubscriptionsForChangeSet =
    defaultSubscriptions.get(changeSetId);
  if (av.isDefaultSource) {
    defaultSubscriptionsForChangeSet.defaultSubscriptions.set(
      defaultSubKeyString,
      defaultSubKey,
    );
  } else {
    defaultSubscriptionsForChangeSet.defaultSubscriptions.delete(
      defaultSubKeyString,
    );
  }

  for (const externalSource of av.externalSources ?? []) {
    const externalSourceKey: DefaultSubscription = {
      componentId: externalSource.componentId,
      path: externalSource.path as AttributePath,
    };
    defaultSubsForComponent.add(JSON.stringify(externalSourceKey));
  }
};

// A component was deleted, so we have to remove all references to it
// from the default subscriptions data
const removeDefaultSubscriptionsForComponent = (
  changeSetId: ChangeSetId,
  componentId: ComponentId,
) => {
  const defaultSubsForChangeSet = defaultSubscriptions.get(changeSetId);
  for (const [
    keyString,
    defaultSub,
  ] of defaultSubsForChangeSet.defaultSubscriptions.entries()) {
    if (defaultSub.componentId === componentId) {
      defaultSubscriptions
        .get(changeSetId)
        .defaultSubscriptions.delete(keyString);
    }
  }

  for (const componentSet of defaultSubsForChangeSet.componentsForSubs.values()) {
    if (componentSet.has(componentId)) {
      componentSet.delete(componentId);
    }
  }

  defaultSubsForChangeSet.subsForComponents.delete(componentId);
};

// All the avs for this component have been processed, so we can
// now finalize the default subscription data
const finalizeDefaultSubscriptionsForComponent = (
  changeSetId: ChangeSetId,
  componentId: ComponentId,
  defaultSubsForComponent: Set<string>,
) => {
  const defaultSubsForChangeSet = defaultSubscriptions.get(changeSetId);

  defaultSubsForChangeSet.subsForComponents.set(
    componentId,
    defaultSubsForComponent,
  );

  const componentsForSubs = defaultSubsForChangeSet.componentsForSubs;

  for (const key of defaultSubsForComponent) {
    componentsForSubs.get(key).add(componentId);
  }

  for (const [key, componentIds] of componentsForSubs.entries()) {
    if (componentIds.has(componentId) && !defaultSubsForComponent.has(key)) {
      componentIds.delete(componentId);
    }
  }
};

const postProcess = (
  db: Database,
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  doc: AtomDocument,
  id: Id,
  indexChecksum?: string,
  removed = false,
  bust = true,
  followReferences = true,
) => {
  // NOTE: patch ordering matters for us, we need to have list patched
  // prior to doing this work
  // So when we move to streaming patches, we have to do something else
  // to support adding & removing items from lists
  if (LISTABLE_ITEMS.includes(kind)) {
    const listIds: string[] = [];
    if (kind === EntityKind.ComponentInList) {
      const sql = `
      select distinct
        viewId
      FROM
        (select
          atoms.args as viewId,
          json_each.value as ref
        from
          atoms,
          json_each(jsonb_extract(CAST(atoms.data as text), '$.components'))
          inner join index_mtm_atoms mtm
            ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
          inner join indexes ON mtm.index_checksum = indexes.checksum
        ${
          indexChecksum
            ? ""
            : "inner join changesets ON changesets.index_checksum = indexes.checksum"
        }
        where
          ${
            indexChecksum
              ? "indexes.checksum = ?"
              : "changesets.change_set_id = ?"
          }
          AND
          atoms.kind = ?
        )
      WHERE
        ref ->> '$.id' = ?
      `;
      const bind = [
        indexChecksum ?? changeSetId,
        EntityKind.ViewComponentList,
        id,
      ];
      const rows = db.exec({
        sql,
        bind,
        returnValue: "resultRows",
      });
      rows.forEach((r) => {
        listIds.push(r[0] as string);
      });
    }

    atomUpdatedFn(workspaceId, changeSetId, kind, id, doc, listIds, removed);
  }

  if (!COMPUTED_KINDS.includes(kind)) return;

  if (followReferences && !removed) {
    const result = getReferences(
      db,
      doc,
      workspaceId,
      changeSetId,
      kind,
      id,
      indexChecksum,
      false,
    );
    doc = result[0];
  }

  if (kind === EntityKind.Component) {
    if (bust) {
      bustCacheFn(
        workspaceId,
        changeSetId,
        EntityKind.ComponentDetails,
        workspaceId,
      );
    }
  } else if (kind === EntityKind.ManagementConnections) {
    // these are OUTGOING connections
    const data = doc as ManagementConnections;
    if (removed) {
      // delete the outgoing conns for the deleted component
      const conns = allIncomingMgmt.get(changeSetId);
      conns.delete(id);
      for (const componentId of conns.keys()) {
        const outgoing = conns.get(componentId);
        Object.entries(outgoing).forEach(([outgoingId, conn]) => {
          if (conn.toComponentId === id) {
            delete outgoing[outgoingId];
          }
        });
      }
    } else {
      data.connections.forEach((outgoing) => {
        if (outgoing.kind !== "prop") {
          const id = `${outgoing.toComponentId}-${outgoing.fromComponentId}`;
          const incoming = flip(outgoing);
          const conns = allIncomingMgmt
            .get(changeSetId)
            .get(outgoing.toComponentId);
          conns[id] = incoming;
        }
      });
    }

    if (bust) {
      bustCacheFn(
        workspaceId,
        changeSetId,
        EntityKind.IncomingManagementConnections,
        workspaceId,
      );
    }
  } else if (kind === EntityKind.IncomingConnections) {
    const data = doc as IncomingConnections;

    if (removed) {
      // delete the outgoing conns for the deleted component
      const conns = allOutgoingConns.get(changeSetId);
      conns.delete(id);
      // remove the outgoing conns TO (which means FROMS) the deleted component
      for (const componentId of conns.keys()) {
        const incoming = conns.get(componentId);
        Object.entries(incoming).forEach(([incomingId, conn]) => {
          if (conn.fromComponentId === id) {
            delete incoming[incomingId];
          }
        });
      }
    } else {
      data.connections.forEach((incoming) => {
        if (incoming.kind !== "management") {
          const id = `${incoming.toAttributeValueId}-${incoming.fromAttributeValueId}`;
          const outgoing = flip(incoming);
          const conns = allOutgoingConns
            .get(changeSetId)
            .get(incoming.fromComponentId);
          conns[id] = outgoing;
        }
      });
    }
    if (bust) {
      bustCacheFn(
        workspaceId,
        changeSetId,
        EntityKind.OutgoingCounts,
        workspaceId,
      );

      bustCacheFn(
        workspaceId,
        changeSetId,
        EntityKind.OutgoingConnections,
        workspaceId,
      );
    }
  } else if (kind === EntityKind.AttributeTree) {
    if (!removed && !doc) {
      error("Atom is not removed, but no data for post processing", id);
      return;
    }

    const attributeTree = doc as AttributeTree;
    if (doc) {
      const defaultSubsForComponent: Set<string> = new Set();
      const possibleConnsForComponent: Record<string, PossibleConnection> = {};
      Object.values(attributeTree.attributeValues).forEach((av) => {
        processAvForDefaultSubscriptions(
          changeSetId,
          attributeTree.id,
          av,
          defaultSubsForComponent,
        );

        const prop = attributeTree.props[av.propId ?? ""];
        if (av.path && prop && prop.eligibleForConnection && !prop.hidden) {
          possibleConnsForComponent[av.id] = {
            attributeValueId: av.id,
            value: av.secret ? av.secret.name : av.value,
            path: av.path,
            name: prop.name,
            componentId: attributeTree.id,
            componentName: attributeTree.componentName,
            schemaName: attributeTree.schemaName,
            kind: prop.kind,
            isOriginSecret: prop.isOriginSecret,
            suggestAsSourceFor: prop.suggestAsSourceFor,
          };
        }
      });

      possibleConns
        .get(changeSetId)
        .set(attributeTree.id, possibleConnsForComponent);

      finalizeDefaultSubscriptionsForComponent(
        changeSetId,
        attributeTree.id,
        defaultSubsForComponent,
      );
    }

    if (removed) {
      possibleConns.get(changeSetId).delete(id);
      removeDefaultSubscriptionsForComponent(changeSetId, id);
    }

    // dont bust individually on cold start
    if (bust) {
      bustCacheFn(
        workspaceId,
        changeSetId,
        EntityKind.AttributeTree,
        attributeTree?.id ?? id,
      );
      bustCacheFn(
        workspaceId,
        changeSetId,
        EntityKind.PossibleConnections,
        workspaceId,
      );
      bustCacheFn(
        workspaceId,
        changeSetId,
        EntityKind.DefaultSubscriptions,
        workspaceId,
      );
      bustCacheFn(
        workspaceId,
        changeSetId,
        EntityKind.QueryAttributes,
        workspaceId,
      );
    }
  }
};

const getPossibleConnections = (_workspaceId: string, changeSetId: string) => {
  const result = [];
  for (const componentMap of possibleConns.get(changeSetId).values()) {
    for (const possibleConn of Object.values(componentMap)) {
      result.push(possibleConn);
    }
  }
  return result;
};

const getOutgoingConnectionsByComponentId = (
  _workspaceId: string,
  changeSetId: string,
) => {
  return allOutgoingConns.get(changeSetId);
};

const getOutgoingConnectionsCounts = (
  _workspaceId: string,
  changeSetId: string,
) => {
  const data = allOutgoingConns.get(changeSetId);
  const counts: Record<ComponentId, number> = {};
  [...data.entries()].forEach(([componentId, conns]) => {
    counts[componentId] = Object.values(conns).length;
  });
  return counts;
};

const getIncomingManagementByComponentId = (
  _workspaceId: string,
  changeSetId: string,
) => {
  return allIncomingMgmt.get(changeSetId);
};

const getDefaultSubscriptions = (
  _workspaceId: string,
  changeSetId: string,
): DefaultSubscriptions => {
  return defaultSubscriptions.get(changeSetId);
};

const getComponentDetails = (
  db: Database,
  _workspaceId: string,
  changeSetId: string,
  indexChecksum?: string,
) => {
  const sql = `
    select
      atoms.args,
      replace(atoms.data -> '$.name', '"', ''),
      replace(atoms.data -> '$.schemaVariantName', '"', '')
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
      atoms.kind = 'Component'
    ;`;
  const bind = [indexChecksum ?? changeSetId];
  const start = performance.now();
  const data = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  const end = performance.now();
  debug("sql get names", end - start, "ms");
  const details: Record<string, ComponentInfo> = {};
  data.forEach((row) => {
    details[row[0] as string] = {
      name: row[1] as string,
      schemaVariantName: row[2] as string,
    };
  });
  return details;
};

const getComponentsInViews = (
  db: Database,
  _workspaceId: string,
  changeSetId: string,
  indexChecksum?: string,
) => {
  const sql = `
    SELECT DISTINCT
      atoms.args AS viewId,
      components.value ->> '$.id' AS componentId
    FROM ${
      indexChecksum
        ? "indexes"
        : "changesets JOIN indexes on indexes.checksum = changesets.index_checksum"
    }
      JOIN index_mtm_atoms ON indexes.checksum = index_mtm_atoms.index_checksum
      JOIN atoms ON atoms.kind = index_mtm_atoms.kind AND atoms.args = index_mtm_atoms.args AND atoms.checksum = index_mtm_atoms.checksum
      JOIN json_each(jsonb_extract(CAST(atoms.data as text), '$.components')) AS components
    WHERE ${
      indexChecksum
        ? "indexes.index_checksum = ?"
        : "changesets.change_set_id = ?"
    }
      AND atoms.kind = 'ViewComponentList'
  `;

  const bind = [indexChecksum ?? changeSetId];
  const data = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  }) as [ViewId, ComponentId][];

  const result: Record<ViewId, Set<ComponentId>> = {};
  for (const [viewId, componentId] of data) {
    result[viewId] ??= new Set();
    result[viewId]?.add(componentId);
  }
  return result;
};

const getComponentsInOnlyOneView = (
  db: Database,
  _workspaceId: string,
  changeSetId: string,
  indexChecksum?: string,
) => {
  const sql = `
    WITH views_and_components AS (
      SELECT
        atoms.args AS viewId,
        components.value ->> '$.id' AS componentId,
        ${
          indexChecksum ? "indexes.index_checksum" : "changesets.change_set_id"
        } AS filter_value
      FROM ${
        indexChecksum
          ? "indexes"
          : "changesets JOIN indexes ON indexes.checksum = changesets.index_checksum"
      }
        JOIN index_mtm_atoms ON indexes.checksum = index_mtm_atoms.index_checksum
        JOIN atoms ON atoms.kind = index_mtm_atoms.kind AND atoms.args = index_mtm_atoms.args AND atoms.checksum = index_mtm_atoms.checksum
        JOIN json_each(jsonb_extract(CAST(atoms.data AS text), '$.components')) AS components
      WHERE atoms.kind = 'ViewComponentList'
    )
    SELECT DISTINCT
      viewId,
      componentId
    FROM views_and_components
    WHERE filter_value = ?
      AND componentId IN (
        SELECT componentId
        FROM views_and_components
        GROUP BY componentId
        HAVING COUNT(*) = 1
      );
  `;

  const bind = [indexChecksum ?? changeSetId];
  const data = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  }) as [ViewId, ComponentId][];

  const result: Record<ComponentId, ViewId> = {};
  for (const [viewId, componentId] of data) {
    result[componentId] = viewId;
  }
  return result;
};

const flip = (i: Connection): Connection => {
  const o: Connection = {
    ...i,
    fromComponentId: i.toComponentId,
    toComponentId: i.fromComponentId,
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
const HAVE_REFERENCES = [EntityKind.Component, EntityKind.ViewList];
const getReferences = (
  db: Database,
  atomDoc: AtomDocument,
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: EntityKind,
  id: Id,
  indexChecksum?: string,
  followComputed?: boolean,
) => {
  if (!HAVE_REFERENCES.includes(kind)) {
    return [atomDoc, false];
  }

  const span = tracer.startSpan("getReferences");
  span.setAttributes({
    workspaceId,
    changeSetId,
    kind,
    id,
  });

  debug("🔗 reference query", kind, id);

  let hasReferenceError = false;

  if (kind === EntityKind.Component) {
    const data = atomDoc as EddaComponent;
    const sv = get(
      db,
      workspaceId,
      changeSetId,
      data.schemaVariantId.kind,
      data.schemaVariantId.id,
      undefined,
      indexChecksum,
      followComputed,
    ) as SchemaVariant | -1;

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
        db,
        workspaceId,
        changeSetId,
        data.schemaVariantId.kind,
        data.schemaVariantId.id,
      );
      // add a weak reference in the case of a miss
      // because if we throw a hammer for what we missed
      // this referencing data doesn't change and needs to bust
      weakReference(
        db,
        changeSetId,
        { kind: data.schemaVariantId.kind, args: data.schemaVariantId.id },
        { kind, args: data.id },
      );
    }

    const sm = get(
      db,
      workspaceId,
      changeSetId,
      data.schemaMembers.kind,
      data.schemaMembers.id,
      undefined,
      indexChecksum,
      followComputed,
    ) as SchemaMembers | -1;

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
        db,
        workspaceId,
        changeSetId,
        data.schemaMembers.kind,
        data.schemaMembers.id,
      );
      // add a weak reference in the case of a miss
      // because if we throw a hammer for what we missed
      // this referencing data doesn't change and needs to bust
      weakReference(
        db,
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
      } else if (
        !schemaMembers.editingVariantId &&
        data.schemaVariantId.id !== schemaMembers.defaultVariantId
      ) {
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
  } else {
    span.end();
    return [atomDoc, hasReferenceError];
  }
};

const IGNORE_LIST = new Set<EntityKind>([
  EntityKind.LuminorkDefaultVariant,
  EntityKind.LuminorkSchemaVariant,
]);

const LISTABLE_ITEMS = [
  EntityKind.ComponentInList,
  EntityKind.IncomingConnections,
  EntityKind.View,
];
const LISTABLE = [
  EntityKind.ComponentList,
  EntityKind.ViewComponentList,
  EntityKind.IncomingConnectionsList,
  EntityKind.ViewList,
];
const getList = (
  db: Database,
  _workspaceId: string,
  changeSetId: ChangeSetId,
  kind: Listable,
  id: Id,
  indexChecksum?: string,
): string => {
  let varname;
  switch (kind) {
    case EntityKind.ComponentList:
    case EntityKind.ViewComponentList:
      varname = "$.components";
      break;
    case EntityKind.IncomingConnectionsList:
      varname = "$.componentConnections";
      break;
    case EntityKind.ViewList:
      varname = "$.views";
      break;
    default:
      throw new Error("Missing kind");
  }

  const sql = `
select
  json_group_array(resolved.atom_json)
from
  (
    select
      jsonb_extract(CAST(data as text), '$') as atom_json
    from
      atoms
    INNER JOIN
      (
      select
        ref ->> '$.id' as args,
        ref ->> '$.kind' as kind
      from
        (
          select
            json_each.value as ref
          from
            atoms,
            json_each(jsonb_extract(CAST(atoms.data as text), '${varname}'))
            inner join index_mtm_atoms mtm
              ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
            inner join indexes ON mtm.index_checksum = indexes.checksum
          ${
            indexChecksum
              ? ""
              : "inner join changesets ON changesets.index_checksum = indexes.checksum"
          }
          where
            ${
              indexChecksum
                ? "indexes.checksum = ?"
                : "changesets.change_set_id = ?"
            }
            AND atoms.kind = ?
            AND atoms.args = ?
        ) as items
      ) item_refs
    ON
    atoms.args = item_refs.args
    AND atoms.kind = item_refs.kind
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
  ) as resolved
;      `;
  const bind = [
    indexChecksum ?? changeSetId,
    kind,
    id,
    indexChecksum ?? changeSetId,
  ];
  const start = performance.now();
  const atomData = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  const end = performance.now();
  debug(
    "❓ sql getList",
    `[${end - start}ms]`,
    bind,
    " returns ?",
    !(atomData.length === 0),
    atomData,
  );
  if (atomData.length === 0) return "";

  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return atomData[0]![0] as string;
};

const getKind = (
  db: Database,
  _workspaceId: string,
  changeSetId: ChangeSetId,
  kind: EntityKind,
  // NOTE: use the `makeArgs` helper for this call, and do not pass an ID
) => {
  const isGlobal = GLOBAL_ENTITIES.includes(kind as GlobalEntity);
  const sql = isGlobal
    ? `
    select CAST(data as text)
    from global_atoms
    where kind = ?
  `
    : `
    select CAST(data as text)
    from atoms
      inner join index_mtm_atoms mtm ON
        atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
      inner join indexes ON mtm.index_checksum = indexes.checksum
      inner join changesets ON changesets.index_checksum = indexes.checksum
    where
      atoms.kind = ? AND
      changesets.change_set_id = ?
  `;
  const rows = db.exec({
    sql,
    bind: isGlobal ? [kind] : [kind, changeSetId],
    returnValue: "resultRows",
  });
  return rows.map((r) => r[0] as string | undefined).filter(nonNullable);
};

const queryAttributes = (
  db: Database,
  _workspaceId: WorkspacePk,
  changeSetId: ChangeSetId,
  terms: QueryAttributesTerm[],
) => {
  // Generate the SQL statements and the respective binds
  const sqlTerms = terms.flatMap((term) => {
    const key = term.key.startsWith("/") ? term.key : `%/${term.key}`;
    // Extract the correct SQL like statement for our values, using * as the wildcard and respecting exact vs startsWith
    let value = term.value;
    // If the value is all digits, we always run an exact match, so no need to add % to the end
    if (!term.value.match(/^\d+$/)) {
      value =
        term.value.replaceAll("*", "%") + (term.op === "startsWith" ? "%" : "");
    }

    // This is the default search statements, we include special cases further down
    const sqlTerms = [
      {
        statement:
          "(attr.value ->> 'path' LIKE ? AND attr.value ->> 'value' LIKE ?)",
        binds: [key, value] as (string | boolean | number)[],
      },
    ];

    // We translate the strings "true" and "false" to literal booleans we can match on sqlite
    const booleanValues: Record<string, string | boolean> = {
      true: true,
      false: false,
    };
    const valueAsBoolean = booleanValues[term.value.toLowerCase()];
    if (valueAsBoolean !== undefined) {
      sqlTerms.push({
        statement:
          "(attr.value ->> 'path' LIKE ? AND attr.value ->> 'value' = ?)",
        binds: [key, valueAsBoolean],
      });
    }

    // When searching for schema, we also try to match schema name alongside any props called schema (default case)
    if (term.key === "schema") {
      sqlTerms.push({
        statement: "(schema_name LIKE ?)",
        binds: [value],
      });
    }

    return sqlTerms;
  });

  const sql = `
    SELECT
        atoms.args AS component_id,
        jsonb_extract(CAST(atoms.data as text), '$.schemaName') AS schema_name
       FROM changesets
       JOIN indexes ON changesets.index_checksum = indexes.checksum
       JOIN index_mtm_atoms ON indexes.checksum = index_mtm_atoms.index_checksum
       JOIN atoms ON atoms.kind = index_mtm_atoms.kind AND atoms.args = index_mtm_atoms.args AND atoms.checksum = index_mtm_atoms.checksum
       JOIN json_each(jsonb_extract(CAST(atoms.data as text), '$.attributeValues')) AS attr
      WHERE changesets.change_set_id = ?
        AND atoms.kind = 'AttributeTree'
        AND (${sqlTerms.map((t) => t.statement).join(" OR ")})
  `;

  const bind = [changeSetId, ...sqlTerms.flatMap((term) => term.binds)];

  const start = Date.now();
  const components = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });

  const end = Date.now();
  debug(
    "❓ sql queryAttributes",
    `[${end - start}ms]`,
    bind,
    " returns ?",
    !(components.length === 0),
    components,
  );
  return components.map((c) => c[0] as ComponentId);
};

const getGlobal = (
  db: Database,
  workspaceId: string,
  kind: GlobalEntity,
  id: Id,
): -1 | object => {
  const sql = `
    select
      data
    from
      global_atoms
    where
      global_atoms.kind = ? AND
      global_atoms.args = ?
    ;`;
  const bind = [kind, id];
  const start = performance.now();
  const atomData = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  const end = performance.now();
  const data = oneInOne(atomData);
  debug(
    "❓ sql get",
    `[${end - start}ms]`,
    bind,
    " returns ?",
    !(data === NOROW),
  );
  if (data === NOROW) {
    deploymentMjolnir(db, workspaceId, kind, id);
    return -1;
  }
  const atomDoc = decodeDocumentFromDB(data as ArrayBuffer);
  return atomDoc;
};

const get = (
  db: Database,
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: Gettable,
  id: Id,
  checksum?: string, // intentionally not used in sql, putting it on the wire for consistency & observability purposes
  indexChecksum?: string,
  followComputed = true,
  followReferences = true,
): -1 | object => {
  if (kind in GLOBAL_ENTITIES) throw new Error(`Use "get_global" for ${kind}`);

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
  const start = performance.now();
  const atomData = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  const end = performance.now();
  const data = oneInOne(atomData);
  debug(
    "❓ sql get",
    `[${end - start}ms]`,
    bind,
    " returns ?",
    !(data === NOROW),
  );
  if (data === NOROW) {
    mjolnir(db, workspaceId, changeSetId, kind, id, checksum);
    return -1;
  }
  const atomDoc = decodeDocumentFromDB(data as ArrayBuffer);
  // debug("📄 atom doc", atomDoc);

  // THIS GETS REPLACED WITH AUTO-GEN CODE
  if (!followReferences) return atomDoc;

  try {
    const [docAndRefs, hasReferenceError] = getReferences(
      db,
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

    return docAndRefs;
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(err);
    return -1;
  }
};

const getExists = (
  db: Database,
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: Gettable,
  id: Id,
) => {
  const sql = `
    select
      args
    from
      atoms
    INNER JOIN
      (
      select
        ref ->> '$.id' as args,
        ref ->> '$.kind' as kind
      from
        (
          select
            json_each.value as ref
          from
            atoms,
            json_each(jsonb_extract(CAST(atoms.data as text), 'mvList'))
            inner join index_mtm_atoms mtm
              ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
            inner join indexes ON mtm.index_checksum = indexes.checksum
              "inner join changesets ON changesets.index_checksum = indexes.checksum"
          where
            changesets.change_set_id = ?
            AND atoms.kind = ?
            AND atoms.args = ?
        ) as items
      ) item_refs
    ON
    atoms.args = item_refs.args
    AND atoms.kind = item_refs.kind
    inner join index_mtm_atoms mtm
      ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
    inner join indexes ON mtm.index_checksum = indexes.checksum
    inner join changesets ON changesets.index_checksum = indexes.checksum
    where
      changesets.change_set_id = ?
      and kind = ?
      and args = ?
  ) as resolved
;      `;
  const bind = [
    changeSetId,
    EntityKind.MvIndex,
    workspaceId,
    changeSetId,
    kind,
    id,
  ];
  const exists = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });

  return exists.length > 0;
};

const getSchemaMembers = (
  db: Database,
  _workspaceId: string,
  changeSetId: ChangeSetId,
  indexChecksum?: string,
): string => {
  const sql = `
    select
      json_group_array(jsonb_extract(CAST(data as text), '$'))
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
      atoms.kind = ?
    ;`;

  const bind = [indexChecksum ?? changeSetId, EntityKind.SchemaMembers];
  const start = performance.now();
  const atomData = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  const end = performance.now();

  debug("❓ sql getSchemaMembers", `[${end - start}ms]`);
  if (atomData.length === 0) return "";
  else return oneInOne(atomData) as string;
};

/**
 * NOTE: getMany returns Edda types, not Bifrost types! Because it does not follow references
 */
const _getMany = (
  db: Database,
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: EntityKind,
  ids: Id[],
  indexChecksum?: string,
): Record<Id, AtomDocument | -1> => {
  if (ids.length === 0) return {};

  const results: Record<Id, AtomDocument | -1> = {};

  // Build SQL query to fetch multiple atoms at once
  const placeholders = ids.map(() => "?").join(",");
  const sql = `
    select
      atoms.args as id,
      atoms.data
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
      atoms.args IN (${placeholders})
    ;`;

  const bind = [indexChecksum ?? changeSetId, kind, ...ids];
  const start = performance.now();
  const atomData = db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  const end = performance.now();

  debug(
    "❓ sql getMany",
    `[${end - start}ms]`,
    `kind: ${kind}, ids: ${ids.length}`,
    " returns",
    atomData.length,
    "results",
  );

  // Track which IDs we found vs missing
  const foundIds = new Set<Id>();

  // Process found results
  for (const row of atomData) {
    const id = row[0] as Id;
    const data = row[1] as ArrayBuffer;
    foundIds.add(id);

    const atomDoc = decodeDocumentFromDB(data);

    results[id] = atomDoc;
  }

  for (const id of ids) {
    if (!foundIds.has(id)) {
      results[id] = -1;
    }
  }

  return results;
};

const clientInterest: Record<string, number> = {};

const receiveInterest = (interest: Record<string, number>) => {
  Object.assign(clientInterest, interest);
};

const assignPriority = (workspaceId: string, changeSetId: string) => {
  const key = `${workspaceId}-${changeSetId}`;
  const priority = clientInterest[key] ?? 0;
  return { priority };
};

/**
 * INTERFACE DEFINITION
 */

const sockets: { [key: string]: ReconnectingWebSocket } = {};
const bearerTokens: { [key: string]: string } = {};

let bustCacheFn: BustCacheFn;
let inFlightFn: RainbowFn;
let returnedFn: RainbowFn;
let lobbyExitFn: LobbyExitFn;
let atomUpdatedFn: UpdateFn;
let updateConnectionStatus: ConnStatusFn;

let abortController: AbortController | undefined;

const forceLeaderElectionBroadcastChannel = new BroadcastChannel(
  FORCE_LEADER_ELECTION,
);

/**
 * This enforces that `receiveBroadcast` handles
 * each discriminant of `BroadcastMessage`
 */
const assertNever = (_foo: never) => {};

const dbInterface: TabDBInterface = {
  receiveInterest(interest: Record<string, number>) {
    receiveInterest(interest);
  },
  async receiveBroadcast(message: BroadcastMessage) {
    switch (message.messageKind) {
      case "updateConnectionStatus":
        updateConnectionStatus(
          message.arguments.workspaceId,
          message.arguments.connected,
          true,
        );
        break;
      case "cacheBust":
        bustCacheFn(
          message.arguments.workspaceId,
          message.arguments.changeSetId,
          message.arguments.kind,
          message.arguments.id,
          true,
        );
        break;
      case "listenerInFlight":
        inFlightFn(
          message.arguments.changeSetId,
          message.arguments.label,
          true,
        );
        break;
      case "listenerReturned":
        returnedFn(
          message.arguments.changeSetId,
          message.arguments.label,
          true,
        );
        break;
      case "atomUpdated":
        atomUpdatedFn(
          message.arguments.workspaceId,
          message.arguments.changeSetId,
          message.arguments.kind,
          message.arguments.id,
          message.arguments.data,
          message.arguments.listIds,
          message.arguments.removed,
          true,
        );
        break;
      case "interest":
        receiveInterest(message.arguments);
        break;
      case "lobbyExit":
        lobbyExitFn(
          message.arguments.workspaceId,
          message.arguments.changeSetId,
          true,
        );
        break;
      default:
        assertNever(message);
    }
  },
  setBearer(workspaceId, token) {
    bearerTokens[workspaceId] = token;
    let apiUrl: string;
    if (import.meta.env.VITE_API_PROXY_PATH) {
      // eslint-disable-next-line no-restricted-globals
      apiUrl = `${location.protocol}//${location.host}${
        import.meta.env.VITE_API_PROXY_PATH
      }`;
    } else throw new Error("Invalid API env var config");
    const API_HTTP_URL = apiUrl;

    sdfClients[workspaceId] = Axios.create({
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

    sdfClients[workspaceId]?.interceptors.request.use(injectBearerTokenAuth);
  },
  async initDB(testing: boolean) {
    return initializeSQLite(testing);
  },

  migrate(testing: boolean) {
    const result = ensureTables(testing);
    debug("Migration completed");
    return result;
  },

  async initSocket(workspaceId: string) {
    if (typeof sockets[workspaceId] !== "undefined") {
      return;
    }

    debug("Initializing websocket for workspaceId", workspaceId);

    try {
      const token = bearerTokens[workspaceId];
      sockets[workspaceId] = new ReconnectingWebSocket(
        () => `/api/ws/bifrost?token=Bearer+${token}`,
        [],
        {
          // see options https://www.npmjs.com/package/reconnecting-websocket#available-options
          startClosed: true, // don't start connected - we'll watch auth to trigger
        },
      );
    } catch (err) {
      error(err);
    }

    let interval: NodeJS.Timeout;
    sockets[workspaceId]?.addEventListener("close", () => {
      if (interval) clearInterval(interval);
      updateConnectionStatus(workspaceId, false);
    });
    sockets[workspaceId]?.addEventListener("open", () => {
      updateConnectionStatus(workspaceId, true);
      // heartbeat ping
      interval = setInterval(() => {
        sockets[workspaceId]?.send(new Uint8Array([0x9]));
      }, 1000 * 20);
    });

    sockets[workspaceId]?.addEventListener("message", (messageEvent) => {
      tracer.startActiveSpan("handleEvent", async (span) => {
        // we'll either be getting AtomMessages as patches to the data
        // OR we'll be getting mjolnir responses with the Atom as a whole
        // TODO we also need "changeset closed" messages
        // TODO: handle Index Updates!
        try {
          const data = JSON.parse(messageEvent.data) as
            | WorkspacePatchBatch
            | DeploymentPatchBatch
            | WorkspaceAtomMessage
            | WorkspaceIndexUpdate
            | DeploymentIndexUpdate;

          if (import.meta.env.VITE_LOG_WS) {
            log("🌈 bifrost incoming", data);
          }

          if (!("kind" in data)) span.setAttribute("kindMissing", "no kind");
          else {
            span.setAttributes({
              messageKind: data.kind,
            });
            if ("meta" in data) {
              span.setAttributes({
                messageKind: data.kind,
                ...data.meta,
              });
              if ("workspaceId" in data.meta) {
                span.setAttributes({
                  workspaceId: data.meta.workspaceId,
                  changeSetId: data.meta.changeSetId,
                  toIndexChecksum: data.meta.toIndexChecksum,
                  fromIndexChecksum: data.meta.fromIndexChecksum,
                });
              }
            }

            if (data.kind === MessageKind.WORKSPACE_PATCH) {
              debug(
                "📨 WORKSPACE PATCH MESSAGE START:",
                data.meta.toIndexChecksum,
                "patches:",
                data.patches.length,
              );
              processPatchQueue.add(
                async () =>
                  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                  await sqlite!.transaction(
                    async (db) => await handleWorkspacePatchMessage(db, data),
                  ),
                assignPriority(data.meta.workspaceId, data.meta.changeSetId),
              );
              debug(
                "📨 WORKSPACE PATCH MESSAGE ADDED:",
                data.meta.toIndexChecksum,
              );
            } else if (data.kind === MessageKind.DEPLOYMENT_PATCH) {
              debug(
                "📨 DEPLOYMENT PATCH MESSAGE START:",
                data.meta.toIndexChecksum,
                "patches:",
                data.patches.length,
              );

              processPatchQueue.add(
                async () =>
                  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                  await sqlite!.transaction(
                    async (db) => await handleDeploymentPatchMessage(db, data),
                  ),
              );
            } else if (data.kind === MessageKind.WORKSPACE_INDEXUPDATE) {
              // Index has been updated - signal lobby exit
              debug("📨 INDEX UPDATE", data.meta.changeSetId);

              // this part doesn't go into the queue, tell the app an index is ready right away
              // it will be re-requested
              if (lobbyExitFn) {
                lobbyExitFn(data.meta.workspaceId, data.meta.changeSetId);
              }
              processPatchQueue.add(async () => {
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                await sqlite!.transaction(async (db) =>
                  handleIndexMvPatch(db, data),
                );
              }, assignPriority(data.meta.workspaceId, data.meta.changeSetId));
            } else if (data.kind === MessageKind.DEPLOYMENT_INDEXUPDATE) {
              // NOOP for now, DEPLOYMENT_PATCH does the work
              debug(
                "📨 DEPLOYMENT INDEX UPDATE RECEIVED - IT IS NOT BEING HANDLED RIGHT NOW",
              );
            } else if (data.kind === MessageKind.MJOLNIR) {
              debug(
                "📨 MJOLNIR MESSAGE START:",
                data.atom.kind,
                data.atom.id,
                "toChecksum:",
                data.atom.toChecksum,
              );
              returnedFn(
                data.atom.changeSetId,
                `${data.atom.kind}.${data.atom.id}`,
              );
              hasReturned({
                workspaceId: data.atom.workspaceId,
                changeSetId: data.atom.changeSetId,
                kind: data.atom.kind,
                id: data.atom.id,
              });
              processMjolnirQueue.add(
                async () =>
                  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                  await sqlite!.transaction(async (db) => {
                    return await handleHammer(db, data);
                  }),
              );
              debug(
                "📨 MJOLNIR MESSAGE COMPLETE:",
                data.atom.kind,
                data.atom.id,
              );
            } else {
              /* eslint-disable-next-line no-console */
              console.error(`Unknown data kind on bifrost message: `, data);
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

    sockets[workspaceId]?.addEventListener("error", (errorEvent) => {
      error("ws error", errorEvent.error, errorEvent.message);
    });
  },

  async hasDbLock(): Promise<boolean> {
    return hasTheLock;
  },

  async initBifrost(gotLockPort: MessagePort) {
    debug("waiting for lock in webworker");
    if (abortController) {
      abortController.abort();
    }

    abortController = new AbortController();
    return await navigator.locks.request(
      WORKER_LOCK_KEY,
      { mode: "exclusive", signal: abortController.signal },
      async () => {
        hasTheLock = true;
        debug("lock acquired! 🌈 Initializing sqlite3 bifrost for real");
        await this.initDB(false);
        this.migrate(false);
        debug("🌈 Bifrost initialization complete");

        gotLockPort.postMessage("lock acquired");
        return new Promise((resolve) => {
          forceLeaderElectionBroadcastChannel.onmessage = () => {
            abortController?.abort(FORCE_LEADER_ELECTION);
          };
          abortController?.signal.addEventListener("abort", () => {
            hasTheLock = false;
            sqlite?.close();
            poolUtil?.pauseVfs();
            this.bifrostClose();
            resolve(abortController?.signal.reason);
          });
        });
      },
    );
  },

  bifrostClose() {
    try {
      for (const workspaceId in sockets) {
        sockets[workspaceId]?.close();
      }
    } catch (err) {
      error(err);
    }
  },

  bifrostReconnect() {
    try {
      for (const workspaceId in sockets) {
        const socket = sockets[workspaceId];
        // don't re-connect if you're already connected!
        if (socket && socket.readyState !== WebSocket.OPEN) {
          socket.reconnect();
        }
      }
    } catch (err) {
      error(err);
    }
  },

  addListenerBustCache(cb: BustCacheFn) {
    bustCacheFn = cb;
  },

  addConnStatusFn(cb: ConnStatusFn) {
    updateConnectionStatus = cb;
  },

  async addListenerInFlight(cb: RainbowFn) {
    inFlightFn = cb;
  },
  async addListenerReturned(cb: RainbowFn) {
    returnedFn = cb;
  },
  addAtomUpdated(cb: UpdateFn) {
    atomUpdatedFn = cb;
  },

  addListenerLobbyExit(cb: LobbyExitFn) {
    lobbyExitFn = cb;
  },
  getGlobal(workspaceId, kind, id) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) => getGlobal(db, workspaceId, kind, id));
  },
  get(workspaceId, changeSetId, kind, id) {
    if (IGNORE_LIST.has(kind)) return -1;

    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      get(db, workspaceId, changeSetId, kind, id),
    );
  },
  getExists(workspaceId, changeSetId, kind, id) {
    if (IGNORE_LIST.has(kind)) return false;

    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      getExists(db, workspaceId, changeSetId, kind, id),
    );
  },
  getList(workspaceId, changeSetId, kind, id) {
    if (IGNORE_LIST.has(kind)) return "";

    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      getList(db, workspaceId, changeSetId, kind, id),
    );
  },
  getKind(workspaceId, changeSetId, kind) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      getKind(db, workspaceId, changeSetId, kind),
    );
  },
  getOutgoingConnectionsByComponentId,
  getOutgoingConnectionsCounts,
  getIncomingManagementByComponentId,
  getComponentDetails(workspaceId, changeSetId) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      getComponentDetails(db, workspaceId, changeSetId),
    );
  },
  getComponentsInViews(workspaceId, changeSetId) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      getComponentsInViews(db, workspaceId, changeSetId),
    );
  },
  getComponentsInOnlyOneView(workspaceId, changeSetId) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      getComponentsInOnlyOneView(db, workspaceId, changeSetId),
    );
  },
  getSchemaMembers(workspaceId, changeSetId) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      getSchemaMembers(db, workspaceId, changeSetId),
    );
  },
  getDefaultSubscriptions,
  getPossibleConnections,
  queryAttributes(workspaceId, changeSetId, terms) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      queryAttributes(db, workspaceId, changeSetId, terms),
    );
  },
  partialKeyFromKindAndId: partialKeyFromKindAndArgs,
  kindAndIdFromKey: kindAndArgsFromKey,
  mjolnirBulk(workspaceId, changeSetId, objs, indexChecksum) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      mjolnirBulk(db, workspaceId, changeSetId, objs, indexChecksum),
    );
  },
  mjolnir(workspaceId, changeSetId, kind, id, checksum) {
    if (IGNORE_LIST.has(kind)) return;

    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      mjolnir(db, workspaceId, changeSetId, kind, id, checksum),
    );
  },
  pruneAtomsForClosedChangeSet(workspaceId, changeSetId) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      pruneAtomsForClosedChangeSet(db, workspaceId, changeSetId),
    );
  },
  async vanaheim(workspaceId) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return await sqlite.transaction(
      async (db) => await vanaheim(db, workspaceId),
    );
  },
  async niflheim(workspaceId, changeSetId) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return await sqlite.transaction(
      async (db) => await niflheim(db, workspaceId, changeSetId),
    );
  },
  encodeDocumentForDB,
  decodeDocumentFromDB,
  handleDeploymentPatchMessage(data) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) => handleDeploymentPatchMessage(db, data));
  },
  handleWorkspacePatchMessage(data) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) => handleWorkspacePatchMessage(db, data));
  },
  handleIndexMvPatch(data) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) => handleIndexMvPatch(db, data));
  },
  exec,
  oneInOne,
  // This is only called externally by tests
  handleHammer(msg) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return handleHammer(sqlite, msg);
  },
  bobby: dropTables,
  ragnarok(workspaceId, changeSetId, noColdStart) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) =>
      ragnarok(db, workspaceId, changeSetId, noColdStart),
    );
  },
  changeSetExists: (workspaceId: string, changeSetId: ChangeSetId) => {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    const row = sqlite.exec({
      sql: "select change_set_id from changesets where workspace_id = ? and change_set_id = ?",
      returnValue: "resultRows",
      bind: [workspaceId, changeSetId],
    });
    const cId = oneInOne(row);
    return cId === changeSetId;
  },

  odin(changeSetId: ChangeSetId): object {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    return sqlite.transaction((db) => {
      const changesets = db.exec({
        sql: "select * from changesets where change_set_id=?;",
        bind: [changeSetId],
        returnValue: "resultRows",
      });
      const indexes = db.exec({
        sql: `select indexes.* from indexes
            inner join changesets
              on indexes.checksum = changesets.index_checksum
            where changesets.change_set_id = ?;
      `,
        bind: [changeSetId],
        returnValue: "resultRows",
      });
      const mtm = db.exec({
        sql: `select index_mtm_atoms.* from index_mtm_atoms
            inner join changesets
              on index_mtm_atoms.index_checksum = changesets.index_checksum
            where changesets.change_set_id = ?;
      `,
        bind: [changeSetId],
        returnValue: "resultRows",
      });
      const atoms = db.exec({
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
      const global = db.exec({
        sql: `select
                kind, args, checksum,
                CAST(data as text)
              from
                global_atoms;`,
        returnValue: "resultRows",
      });
      return { changesets, indexes, atoms, mtm, global };
    });
  },
  /**
   * This fn needs to be idempotent. Every tab will listen for "ChangeSetCreated"
   * Every tab will call the one active web worker to run this fn
   * The change set table has `change_set_id` as its primary key
   * So we add "on conflict do nothing" to the insert.
   */
  linkNewChangeset(workspaceId, headChangeSet, changeSetId) {
    if (!sqlite) {
      throw new Error(DB_NOT_INIT_ERR);
    }
    try {
      sqlite.transaction((db) => {
        const headRows = db.exec({
          sql: "select index_checksum from changesets where workspace_id = ? and change_set_id = ?;",
          bind: [workspaceId, headChangeSet],
          returnValue: "resultRows",
        });
        const headRow = oneInOne(headRows);
        if (headRow === NOROW) {
          throw new Error(`HEAD is missing: ${workspaceId}: ${headChangeSet}`);
        }
        const currentIndexChecksum = headRow;
        db.exec({
          sql: "insert into changesets (change_set_id, workspace_id, index_checksum) VALUES (?, ?, ?) on conflict do nothing;",
          bind: [changeSetId, workspaceId, currentIndexChecksum],
        });
      });
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error("linkNewChangeset", err);
    }
  },
};

Comlink.expose(dbInterface);
