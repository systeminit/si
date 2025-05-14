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
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-base";
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
  BifrostViewList,
  RawViewList,
  BifrostView,
  Ragnarok,
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
} from "./types/dbinterface";

let otelEndpoint = import.meta.env.VITE_OTEL_EXPORTER_OTLP_ENDPOINT;
if (!otelEndpoint) otelEndpoint = "http://localhost:8080";
const exporter = new OTLPTraceExporter({
  url: `${otelEndpoint}/v1/traces`,
});

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
  await db.exec({ sql: "PRAGMA foreign_keys = ON;" });
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
  DROP TABLE IF EXISTS snapshots_mtm_atoms;
  DROP TABLE IF EXISTS atoms;
  DROP TABLE IF EXISTS snapshots;
  DROP TABLE IF EXISTS changesets;
  DROP TABLE IF EXISTS weak_references;
  `;
  await db.exec({ sql });
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
   * SOLUTION Use snapshot checksums and FK snapshot_mtm relationships to delete
   */
  const sql = `
  CREATE TABLE IF NOT EXISTS changesets (
    change_set_id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    snapshot_address TEXT NOT NULL,
    FOREIGN KEY (snapshot_address) REFERENCES snapshots(address) ON DELETE CASCADE
  ) WITHOUT ROWID;
  CREATE INDEX IF NOT EXISTS changeset_workspace_id ON changesets(workspace_id);

  CREATE TABLE IF NOT EXISTS snapshots (
    address TEXT PRIMARY KEY
  ) WITHOUT ROWID;

  CREATE TABLE IF NOT EXISTS atoms (
    kind TEXT NOT NULL,
    args TEXT NOT NULL,
    checksum TEXT NOT NULL,
    data BLOB,
    PRIMARY KEY (kind, args, checksum)
  ) WITHOUT ROWID;

  CREATE TABLE IF NOT EXISTS snapshots_mtm_atoms (
    snapshot_address TEXT NOT NULL,
    kind TEXT NOT NULL,
    args TEXT NOT NULL,
    checksum TEXT NOT NULL,
    PRIMARY KEY (snapshot_address, kind, args, checksum),
    FOREIGN KEY (snapshot_address) REFERENCES snapshots(address) ON DELETE CASCADE,
    FOREIGN KEY (kind, args, checksum) REFERENCES atoms(kind, args, checksum) ON DELETE CASCADE,
    CONSTRAINT uniqueness UNIQUE (snapshot_address, kind, args) ON CONFLICT REPLACE
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
   *  - UPDATE changesets SET snapshot_address = rowid
   *  - For each patch data
   *    - fromChecksum = 0, this is net new, insert atom
   *    - toChecksum = 0, this is a deletion, remove atom
   *    - nonzero checksums:
   *      - select * from atoms where kind=<kind>, args=<args>, checksum=<old_checksum>
   *        - if data doesn't exist throw mjolnir
   *      - apply patch data
   *      - atom_id = insert into atoms data=<blob>, kind=<kind>, args=<args>, checksum=<new_checksum>
   *      - insert into snapshots_mtm_atoms atom_id = atom_id, snapshot_address = rowid
   *  - DELETE FROM snapshots WHERE change_set_id=<this_changeSetId> AND checksum=<old_checksum>
   */

  return await db.exec({ sql });
};

// NOTE: this is just for external test usage, do not use this within this file
const exec = (
  opts: ExecBaseOptions &
    ExecRowModeArrayOptions &
    ExecReturnResultRowsOptions & {
      sql: FlexibleString;
    },
): SqlValue[][] => db.exec(opts);

const encodeDocumentForDB = async (doc: object) => {
  return await new Blob([JSON.stringify(doc)]).arrayBuffer();
};

const decodeDocumentFromDB = (doc: ArrayBuffer): AtomDocument => {
  const s = new TextDecoder().decode(doc);
  const j = JSON.parse(s);
  return j;
};

const oneInOne = (rows: SqlValue[][]): SqlValue | typeof NOROW => {
  const first = rows[0];
  if (first) {
    const id = first[0];
    if (id || id === 0) return id;
  }
  return NOROW;
};

const atomExistsOnSnapshots = async (
  atom: Atom,
  kindChecksum: Checksum,
): Promise<Checksum[]> => {
  const rows = await db.exec({
    sql: `
    select
     snapshot_address
    from snapshots_mtm_atoms
    where
      kind=? and
      args=? and
      checksum = ?
    ;
    `,
    bind: [atom.kind, atom.id, kindChecksum],
    returnValue: "resultRows",
  });
  return rows.flat().filter(nonNullable) as Checksum[];
};

const newSnapshot = async (meta: AtomMeta, fromSnapshotAddress?: string) => {
  await db.exec({
    sql: `INSERT INTO snapshots (address) VALUES (?);`,
    bind: [meta.snapshotToAddress],
  });

  if (fromSnapshotAddress && fromSnapshotAddress !== meta.snapshotToAddress) {
    await db.exec({
      sql: `INSERT INTO snapshots_mtm_atoms
        SELECT
          ?, kind, args, checksum
        FROM snapshots_mtm_atoms
        WHERE
          snapshot_address = ?
        `,
      bind: [meta.snapshotToAddress, fromSnapshotAddress],
    });
  }
};

const removeAtom = async (snapshotAddress: Checksum, atom: Required<Atom>) => {
  await db.exec({
    sql: `
    DELETE FROM snapshots_mtm_atoms
    WHERE snapshot_address = ? AND kind = ? AND args = ? AND checksum = ?
    `,
    bind: [snapshotAddress, atom.kind, atom.id, atom.fromChecksum],
  });
};

const createAtomFromPatch = async (atom: Atom, span?: Span) => {
  const doc = {};
  let afterDoc = {};
  if (atom.operations) {
    const applied = applyOperations(doc, atom.operations);
    afterDoc = applied.newDocument;
  }
  return await createAtom(atom, afterDoc, span);
};

const createAtom = async (atom: Atom, doc: object, span?: Span) => {
  debug("createAtom", atom, doc);
  try {
    await db.exec({
      sql: `insert into atoms
        (kind, checksum, args, data)
          VALUES
        (?, ?, ?, ?)
      ON CONFLICT DO NOTHING;
      `,
      bind: [
        atom.kind,
        atom.toChecksum,
        atom.id,
        await encodeDocumentForDB(doc),
      ],
    });
  } catch (err) {
    if (err instanceof Error) {
      if (
        err.name === "SQLite3Error" &&
        err.message.includes("UNIQUE constraint failed")
      )
        span?.setAttribute("unique_failed", true);
      else {
        error("createAtom failed", atom, doc);
        throw err;
      }
    }
  }
};

const partialKeyFromKindAndArgs = (kind: string, id: Id): QueryKey => {
  return `${kind}|${id}`;
};

const kindAndArgsFromKey = (key: QueryKey): { kind: string; id: Id } => {
  const pieces = key.split("|", 2);
  if (pieces.length !== 2) throw new Error(`Bad key ${key} -> ${pieces}`);
  if (!pieces[0] || !pieces[1])
    throw new Error(`Missing key ${key} -> ${pieces}`);
  const kind = pieces[0];
  const id = pieces[1];
  return { kind, id };
};

const dirtyConnections: Record<string, boolean> = {};
const areConnectionsDirty = (workspaceId: string, changeSetId: string) => {
  return dirtyConnections[`${workspaceId}-${changeSetId}`];
};
const markConnectionDirty = (workspaceId: string, changeSetId: string) => {
  dirtyConnections[`${workspaceId}-${changeSetId}`] = true;
};
const bustCacheAndReferences: BustCacheFn = async (
  workspaceId: string,
  changeSetId: string,
  kind: string,
  id: string,
) => {
  // bust me
  bustCacheFn(workspaceId, changeSetId, kind, id);

  // bust everyone who refers to me
  const sql = `
    select referrer_kind, referrer_args from weak_references where target_kind = ? and target_args = ? and change_set_id = ?;
  `;
  const bind = [kind, id, changeSetId];
  const refs = await db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  refs.forEach(([ref_kind, ref_id]) => {
    if (ref_kind && ref_id)
      bustCacheFn(
        workspaceId,
        changeSetId,
        ref_kind as string,
        ref_id as string,
      );
  });
};

const cachedConnections: Record<
  ChangeSetId,
  DefaultMap<ComponentId, BifrostConnection[]>
> = {};

const getOutgoingConnectionsByComponentId = (
  _workspaceId: string,
  changeSetId: string,
) => {
  return cachedConnections[changeSetId];
};

const cleanConnections = async (workspaceId: string, changeSetId: string) => {
  await tracer.startActiveSpan("cleanConnections", async (span: Span) => {
    debug("🧹 connections start...");
    const connectionsById = await _getOutgoingConnectionsByComponentId(
      workspaceId,
      changeSetId,
    );
    cachedConnections[changeSetId] = connectionsById;

    delete dirtyConnections[`${workspaceId}-${changeSetId}`];

    // whenever these change, the outgoing connections must bust as well
    bustCacheFn(workspaceId, changeSetId, "OutgoingConnections", changeSetId);
    const sql = `
      select referrer_kind, referrer_args from weak_references where target_kind = ? and target_args = ? and change_set_id = ?;
    `;
    const bind = ["OutgoingConnections", changeSetId, changeSetId];
    const refs = await db.exec({
      sql,
      bind,
      returnValue: "resultRows",
    });
    refs.forEach(([ref_kind, ref_id]) => {
      if (ref_kind && ref_id)
        bustCacheFn(
          workspaceId,
          changeSetId,
          ref_kind as string,
          ref_id as string,
        );
    });
    span.setAttribute("numBusts", refs.length);
    span.setAttribute("busts", JSON.stringify(refs));
    span.end();
    debug("...connections end 🧹");
  });
};

const handleHammer = async (msg: AtomMessage, span?: Span) => {
  // in between throwing a hammer and receiving it, i might already have written the atom
  const snapshots = await atomExistsOnSnapshots(msg.atom, msg.atom.toChecksum);
  if (snapshots.includes(msg.atom.snapshotToAddress)) return; // noop

  const toSnapshotAddress = await snapshotLogic(msg.atom, span);

  // if the atom exists, i just need the MTM
  if (snapshots.length === 0) {
    await createAtom(msg.atom, msg.data, span);
  }

  if (!toSnapshotAddress)
    throw new Error(
      `Expected snapshot ROWID for ${msg.atom.snapshotToAddress}`,
    );
  await insertAtomMTM(msg.atom, toSnapshotAddress);

  if (
    ["IncomingConnections", "IncomingConnectionsList"].includes(msg.atom.kind)
  )
    markConnectionDirty(msg.atom.workspaceId, msg.atom.changeSetId);

  updateChangeSetWithNewSnapshot(msg.atom);
  removeOldSnapshot();

  if (areConnectionsDirty(msg.atom.workspaceId, msg.atom.changeSetId))
    cleanConnections(msg.atom.workspaceId, msg.atom.changeSetId);

  bustCacheAndReferences(
    msg.atom.workspaceId,
    msg.atom.changeSetId,
    msg.atom.kind,
    msg.atom.id,
  );
};

const insertAtomMTM = async (atom: Atom, toSnapshotAddress: Checksum) => {
  try {
    await db.exec({
      sql: `insert into snapshots_mtm_atoms
        (snapshot_address, kind, args, checksum)
          VALUES
        (?, ?, ?, ?)
        ;`,
      bind: [toSnapshotAddress, atom.kind, atom.id, atom.toChecksum],
    });
  } catch (err) {
    error("createMTM failed", atom);
    throw err;
  }
};

const snapshotLogic = async (meta: AtomMeta, span?: Span) => {
  const { changeSetId, workspaceId, snapshotFromAddress, snapshotToAddress } = {
    ...meta,
  };
  span?.setAttributes({
    changeSetId,
    workspaceId,
    snapshotFromAddress,
    snapshotToAddress,
  });

  const changeSetQuery = await db.exec({
    sql: `select change_set_id, snapshot_address from changesets where change_set_id = ?`,
    returnValue: "resultRows",
    bind: [meta.changeSetId],
  });
  let changeSetExists;
  let fromSnapshotAddress;
  const changeSet = changeSetQuery[0] as string[];
  if (changeSet) {
    [changeSetExists, fromSnapshotAddress] = [...changeSet];
  }

  const snapshotQuery = await db.exec({
    sql: `select address from snapshots where address = ?`,
    returnValue: "resultRows",
    bind: [snapshotToAddress],
  });
  const snapshotExists = oneInOne(snapshotQuery);

  if (changeSetExists && !fromSnapshotAddress)
    throw new Error("Null value from SQL, impossible");

  if (
    changeSetExists &&
    meta.snapshotFromAddress &&
    fromSnapshotAddress !== snapshotFromAddress
  )
    throw new Ragnarok("From Snapshot Doesn't Exist", workspaceId, changeSetId);

  if (snapshotExists === NOROW) await newSnapshot(meta, snapshotFromAddress);

  if (!changeSetExists) {
    await db.exec({
      sql: "insert into changesets (change_set_id, workspace_id, snapshot_address) VALUES (?, ?, ?);",
      bind: [meta.changeSetId, meta.workspaceId, snapshotToAddress],
    });
  }

  return snapshotToAddress;
};

const handlePatchMessage = async (data: PatchBatch, span?: Span) => {
  span?.setAttribute("numRawPatches", data.patches.length);
  if (data.patches.length === 0) return;
  // Assumption: every patch is working on the same workspace and changeset
  // (e.g. we're not bundling messages across workspaces somehow)

  if (!data.meta.changeSetId) throw new Error("Expected changeSetId");

  let toSnapshotAddress: string;
  try {
    toSnapshotAddress = await snapshotLogic(data.meta, span);
  } catch (err) {
    if (err instanceof Ragnarok) {
      span?.addEvent("ragnarok");
      await ragnarok(err.workspaceId, err.changeSetId);
      return;
    } else {
      throw err;
    }
  }

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
  if (!toSnapshotAddress)
    throw new Error(`Expected snapshot for ${data.meta.snapshotToAddress}`);

  atoms.forEach((atom) => {
    if (["IncomingConnections", "IncomingConnectionsList"].includes(atom.kind))
      markConnectionDirty(atom.workspaceId, atom.changeSetId);
  });

  const atomsToBust = await Promise.all(
    atoms.map(async (atom) => {
      return await applyPatch(atom, toSnapshotAddress);
    }),
  );

  if (areConnectionsDirty(data.meta.workspaceId, data.meta.changeSetId))
    cleanConnections(data.meta.workspaceId, data.meta.changeSetId);

  atomsToBust.forEach((atom) => {
    if (atom)
      bustCacheAndReferences(
        atom.workspaceId,
        atom.changeSetId,
        atom.kind,
        atom.id,
      );
  });

  updateChangeSetWithNewSnapshot(data.meta);
  removeOldSnapshot();
};

const applyPatch = async (
  atom: Required<Atom>,
  toSnapshotAddress: Checksum,
) => {
  return await tracer.startActiveSpan("applyPatch", async (span) => {
    span.setAttribute("atom", JSON.stringify(atom));

    // if we have the change already don't do anything
    const snapshots = await atomExistsOnSnapshots(atom, atom.toChecksum);
    if (snapshots.includes(atom.snapshotToAddress)) {
      span.addEvent("noop");
      span.end();
      return;
    }

    // otherwise, find the old record
    const previousSnapshots = await atomExistsOnSnapshots(
      atom,
      atom.fromChecksum,
    );
    span.setAttribute("previousSnapshots", JSON.stringify(previousSnapshots));
    const exists = previousSnapshots.includes(atom.snapshotFromAddress);
    span.setAttribute("exists", exists);

    let needToInsertMTM = false;
    let bustCache = false;
    if (atom.fromChecksum === "0") {
      if (!exists) {
        // if i already have it, this is a NOOP
        span.setAttribute("createAtomFromPatch", true);
        await createAtomFromPatch(atom, span);
        needToInsertMTM = true;
        bustCache = true;
      }
    } else if (atom.toChecksum === "0") {
      // if i've already removed it, this is a NOOP
      if (exists) {
        span.setAttribute("removeAtom", true);
        await removeAtom(toSnapshotAddress, atom);
        bustCache = true;
      }
    } else {
      // patch it if I can
      if (exists) {
        span.setAttribute("patchAtom", true);
        await patchAtom(atom);
        needToInsertMTM = true;
        bustCache = true;
      }
      // otherwise, fire the small hammer to get the full object
      else {
        span.addEvent("mjolnir", { atom: JSON.stringify(atom) });
        mjolnir(
          atom.workspaceId,
          atom.changeSetId,
          atom.kind,
          atom.id,
          atom.toChecksum,
        );
      }
    }

    // this insert potentially replaces the MTM row that exists for the current snapshot
    // based on the table constraint
    if (needToInsertMTM) await insertAtomMTM(atom, toSnapshotAddress);
    span.end();
    if (bustCache) return atom;
    return undefined;
  });
};

const patchAtom = async (atom: Required<Atom>) => {
  const atomRows = await db.exec({
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

  await db.exec({
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
};

const mjolnir = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: string,
  id: Id,
  checksum?: Checksum,
) => {
  debug("🔨 mjolnir", kind, id, checksum);
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
    } catch (err) {
      span.setAttribute("http.status", 404);
      error("MJOLNIR 404", url, params, err);
    } finally {
      if (req?.status) span.setAttribute("http.status", req.status);
      span.end();
    }
  });
  if (!req) throw new Error("Impossible...");
  // TODO listen to the reply on the websocket

  const msg: AtomMessage = {
    kind: MessageKind.MJOLNIR,
    atom: {
      id: req.data.frontEndObject.id,
      kind: req.data.frontEndObject.kind,
      toChecksum: req.data.frontEndObject.checksum,
      workspaceId,
      changeSetId,
      snapshotToAddress: req.data.workspaceSnapshotAddress,
    },
    data: req.data.frontEndObject.data,
  };
  await handleHammer(msg);
};

const updateChangeSetWithNewSnapshot = async (meta: AtomMeta) => {
  await db.exec({
    sql: "update changesets set snapshot_address = ? where change_set_id = ?;",
    bind: [meta.snapshotToAddress, meta.changeSetId],
  });
};

const removeOldSnapshot = async () => {
  await tracer.startActiveSpan("pruneFromSnapshot", async (span) => {
    await db.exec({
      sql: `
        DELETE FROM snapshots WHERE address NOT IN (SELECT snapshot_address FROM changesets);
      `,
    });
    await db.exec({
      sql: `
        DELETE FROM atoms
        WHERE (kind, args, checksum) NOT IN (
          SELECT  kind, args, checksum  FROM snapshots_mtm_atoms
        ) returning atoms.kind, atoms.args, atoms.checksum;
      `,
    });
    span.end();
  });
};

const pruneAtomsForClosedChangeSet = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
) => {
  await tracer.startActiveSpan("pruneClosedChangeSet", async (span) => {
    span.setAttributes({ workspaceId, changeSetId });
    await db.exec({
      sql: `
        DELETE FROM changesets WHERE change_set_id = ?;
      `,
      bind: [changeSetId],
    });
    await removeOldSnapshot();
    span.end();
  });
};

const atomChecksumsFor = async (
  changeSetId: ChangeSetId,
): Promise<Record<QueryKey, Checksum>> => {
  const mapping: Record<QueryKey, Checksum> = {};
  const rows = await db.exec({
    sql: `
    select atoms.kind, atoms.args, atoms.checksum
    from atoms
    inner join snapshots_mtm_atoms mtm
      ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
    inner join snapshots ON mtm.snapshot_address = snapshots.address
    inner join changesets ON changesets.snapshot_address = snapshots.address
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

const niflheim = async (workspaceId: string, changeSetId: ChangeSetId) => {
  tracer.startActiveSpan("niflheim", async (span: Span) => {
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
    const req = await sdf<IndexObjectMeta>({
      method: "get",
      url,
    });
    const atoms = req.data.frontEndObject.data.mvList;
    frigg.setAttribute("numEntries", atoms.length);
    frigg.end();

    const local = tracer.startSpan("localChecksums");
    const localChecksums = await atomChecksumsFor(changeSetId);
    local.setAttribute("numEntries", Object.keys(localChecksums).length);
    local.end();

    const compare = tracer.startSpan("compare");
    let numHammers = 0;
    atoms.forEach(({ kind, id, checksum }) => {
      const key = partialKeyFromKindAndArgs(kind, id);
      const local = localChecksums[key];
      if (!local || local !== checksum) {
        const { kind, id } = kindAndArgsFromKey(key);
        mjolnir(workspaceId, changeSetId, kind, id, checksum);
        numHammers++;
      }
    });
    compare.setAttribute("numHammers", numHammers);
    compare.end();

    await cleanConnections(workspaceId, changeSetId);
    span.end();
  });
};

const ragnarok = async (workspaceId: string, changeSetId: string) => {
  // get rid of the snapshots we have for this changeset
  await db.exec({
    sql: `delete from snapshots where address IN (select snapshot_address from changesets where workspace_id = ? and change_set_id = ? );`,
    bind: [workspaceId, changeSetId],
  });
  // remove the atoms we have for this change set
  await pruneAtomsForClosedChangeSet(workspaceId, changeSetId);
  // call for a cold start to re-populate
  await niflheim(workspaceId, changeSetId);
};

const clear_weak_references = async (
  changeSetId: string,
  referrer: { kind: string; args: string },
) => {
  const sql = `
    delete from weak_references
    where change_set_id = ? and referrer_kind = ? and referrer_args = ?
  ;`;
  const bind = [changeSetId, referrer.kind, referrer.args];
  await db.exec({
    sql,
    bind,
  });
};

const weak_reference = async (
  changeSetId: string,
  target: { kind: string; args: string },
  referrer: { kind: string; args: string },
) => {
  const sql = `
    insert into weak_references
      (change_set_id, target_kind, target_args, referrer_kind, referrer_args)
    values
      (?, ?, ?, ?, ?)
    on conflict do nothing
  ;`;
  const bind = [
    changeSetId,
    target.kind,
    target.args,
    referrer.kind,
    referrer.args,
  ];
  await db.exec({
    sql,
    bind,
  });
};

const flip = (i: BifrostConnection): BifrostConnection => {
  const o: BifrostConnection = {
    ...i,
    fromComponent: i.toComponent,
    fromAttributeValueId: i.toAttributeValueId,
    fromAttributeValuePath: i.toAttributeValuePath,
    toComponent: i.fromComponent,
    toAttributeValueId: i.fromAttributeValueId,
    toAttributeValuePath: i.fromAttributeValuePath,
  };
  if ("toPropId" in i && o.kind === "prop") {
    o.fromPropId = i.toPropId;
    o.fromPropPath = i.toPropPath;
    o.toPropId = i.fromPropId;
    o.toPropPath = i.fromPropId;
  }
  if ("toSocketId" in i && o.kind === "socket") {
    o.fromSocketId = i.toSocketId;
    o.fromSocketName = i.toSocketName;
    o.toSocketId = i.fromSocketId;
    o.toSocketName = i.fromSocketName;
  }
  return o;
};

const _getOutgoingConnectionsByComponentId = async (
  workspaceId: string,
  changeSetId: string,
) => {
  const list = (await get(
    workspaceId,
    changeSetId,
    "IncomingConnectionsList",
    changeSetId,
    undefined,
    false, // don't compute
  )) as BifrostIncomingConnectionsList;

  const all = list.componentConnections.flatMap((conn) => conn.incoming);

  return all.reduce((obj, conn) => {
    const m = obj.get(conn.fromComponent.id);
    m.push(flip(conn));
    obj.set(conn.fromComponent.id, m);
    return obj;
  }, new DefaultMap<string, BifrostConnection[]>(() => [] as BifrostConnection[]));
};

/**
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
  kind: string,
  id: string,
) => {
  // PSA: in general, any `get` you do in here, you're going to want to pass `followComputed=false`
  // otherwise you're liable to run into an infinite recursion lookup
  if (!["Component", "ViewComponentList", "ComponentList"].includes(kind))
    return atomDoc;

  const connectionsById = getOutgoingConnectionsByComponentId(
    workspaceId,
    changeSetId,
  );
  if (!connectionsById) {
    debug("~ missing connections ~");
    // making this, so when connections populate, we re-query
    weak_reference(
      changeSetId,
      { kind: "OutgoingConnections", args: changeSetId },
      { kind, args: id },
    );
    return atomDoc;
  }

  debug("🔗 computed operation", kind, id);

  if (kind === "ViewComponentList" || kind === "ComponentList") {
    const data = atomDoc as BifrostComponentList;
    data.components.forEach((c) => {
      c.outputCount = connectionsById.get(c.id).length;
    });
    clear_weak_references(changeSetId, { kind, args: id });
    weak_reference(
      changeSetId,
      { kind: "OutgoingConnections", args: changeSetId },
      { kind, args: id },
    );
    return data;
  } else if (kind === "Component") {
    const data = atomDoc as BifrostComponent;
    data.outputCount = connectionsById.get(id).length;
    clear_weak_references(changeSetId, { kind, args: id });
    weak_reference(
      changeSetId,
      { kind: "OutgoingConnections", args: changeSetId },
      { kind, args: id },
    );
    return data;
  } else return atomDoc;
};

const getReferences = async (
  atomDoc: AtomDocument,
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: string,
  id: Id,
  followComputed?: boolean,
) => {
  if (
    ![
      "Component",
      "ViewList",
      "ComponentList",
      "ViewComponentList",
      "IncomingConnections",
      "IncomingConnectionsList",
    ].includes(kind)
  )
    return atomDoc;

  debug("🔗 reference query", kind, id);

  if (kind === "Component") {
    const data = atomDoc as EddaComponent;
    const sv = await get(
      workspaceId,
      changeSetId,
      data.schemaVariantId.kind,
      data.schemaVariantId.id,
      data.schemaVariantId.checksum,
      followComputed,
    );
    const schemaVariant = sv !== -1 ? (sv as SchemaVariant) : undefined;
    if (!schemaVariant) throw new Error("Schema Variant not found");
    const component: BifrostComponent = {
      ...data,
      schemaVariant,
    };
    return component;
  } else if (kind === "ViewList") {
    const rawList = atomDoc as RawViewList;
    const maybeViews = await Promise.all(
      rawList.views.map(async (v) => {
        return await get(
          workspaceId,
          changeSetId,
          v.kind,
          v.id,
          undefined,
          followComputed,
        );
      }),
    );
    const views = maybeViews.filter(
      (v): v is BifrostView => v !== -1 && Object.keys(v).length > 0,
    );
    const list: BifrostViewList = {
      id: rawList.id,
      views,
    };
    return list;
  } else if (kind === "ComponentList" || kind === "ViewComponentList") {
    const rawList = atomDoc as EddaComponentList;
    const maybeComponents = await Promise.all(
      rawList.components.map(async (c) => {
        return await get(
          workspaceId,
          changeSetId,
          c.kind,
          c.id,
          undefined,
          followComputed,
        );
      }),
    );
    const components = maybeComponents.filter(
      (c): c is BifrostComponent => c !== -1 && Object.keys(c).length > 0,
    );
    // NOTE: this is either a bifrost component list or a view component list
    // FUTURE: improve this with some typing magic
    const list: BifrostComponentList = {
      id: rawList.id,
      components,
    };
    return list;
  } else if (kind === "IncomingConnections") {
    const raw = atomDoc as EddaIncomingConnections;
    const component = (await get(
      workspaceId,
      changeSetId,
      "Component",
      raw.id,
      undefined,
      false,
    )) as BifrostComponent;
    clear_weak_references(changeSetId, {
      kind: "IncomingConnections",
      args: raw.id,
    });
    weak_reference(
      changeSetId,
      { kind: "Component", args: component.id },
      { kind: "IncomingConnections", args: raw.id },
    );

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
          false,
        );

        if (fromComponent === -1) throw new Error("Missing component");
        weak_reference(
          changeSetId,
          { kind: c.fromComponentId.kind, args: c.fromComponentId.id },
          { kind: "IncomingConnections", args: raw.id },
        );

        const conn: BifrostConnection = {
          ...c,
          fromComponent: fromComponent as BifrostComponent,
          toComponent: component as BifrostComponent,
        };
        // explicitly setting this as a warning that these fields are not to be used
        conn.fromComponent.outputCount = -1;
        conn.toComponent.outputCount = -1;
        return conn;
      }),
    );

    return {
      id: raw.id,
      component,
      incoming: connections,
      outgoing: [] as BifrostConnection[],
    } as BifrostComponentConnections;
  } else if (kind === "IncomingConnectionsList") {
    const rawList = atomDoc as EddaIncomingConnectionsList;
    const maybeIncomingConnections = await Promise.all(
      rawList.componentConnections.map(async (c) => {
        return (await get(workspaceId, changeSetId, c.kind, c.id)) as
          | BifrostComponentConnections
          | -1;
      }),
    );
    const componentConnections = maybeIncomingConnections.filter(
      (c) => c !== -1 && c && "id" in c,
    ) as BifrostComponentConnections[];
    const list: BifrostIncomingConnectionsList = {
      id: rawList.id,
      componentConnections,
    };
    return list;
  } else return atomDoc;
};

const get = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: string,
  id: Id,
  checksum?: string, // intentionally not used in sql, putting it on the wire for consistency & observability purposes
  followComputed = true,
  followReferences = true,
): Promise<-1 | object> => {
  const sql = `
    select
      data
    from
      atoms
      inner join snapshots_mtm_atoms mtm
        ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
      inner join snapshots ON mtm.snapshot_address = snapshots.address
      inner join changesets ON changesets.snapshot_address = snapshots.address
    where
      changesets.change_set_id = ? AND
      atoms.kind = ? AND
      atoms.args = ?
    ;`;
  const bind = [changeSetId, kind, id];
  const start = Date.now();
  const atomData = await db.exec({
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
  debug("📄 atom doc", atomDoc);

  // THIS GETS REPLACED WITH AUTO-GEN CODE
  if (!followReferences) return atomDoc;

  const docAndRefs = await getReferences(
    atomDoc,
    workspaceId,
    changeSetId,
    kind,
    id,
    followComputed,
  );
  // NOTE: Whenever we ask for the full list of connections
  // This implementation will not compute the outgoing connections (infinite recursion)
  // You will only get incoming—which is all we need when we ask for the whole list
  if (followComputed && !["IncomingConnectionsList"].includes(kind)) {
    return await getComputed(docAndRefs, workspaceId, changeSetId, kind, id);
  }
  return docAndRefs;
};

let socket: ReconnectingWebSocket;
let bustCacheFn: BustCacheFn;
let bearerToken: string;
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
        try {
          const data = JSON.parse(messageEvent.data) as
            | PatchBatch
            | AtomMessage;

          if (import.meta.env.VITE_LOG_WS) {
            log("🌈 bifrost incoming", data);
          }

          if (!("kind" in data)) span.setAttribute("kindMissing", "no kind");
          else {
            span.setAttribute("messageKind", data.kind);
            if (data.kind === MessageKind.PATCH)
              await handlePatchMessage(data, span);
            else if (data.kind === MessageKind.MJOLNIR)
              await handleHammer(data, span);
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
    await Promise.all([this.initDB(false), this.initSocket()]);
    await this.migrate(false);
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

  get,
  getOutgoingConnectionsByComponentId,
  partialKeyFromKindAndId: partialKeyFromKindAndArgs,
  kindAndIdFromKey: kindAndArgsFromKey,
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
    const row = await db.exec({
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
    const s = db.exec({
      sql: `select snapshots.* from snapshots
            inner join changesets
              on snapshots.address = changesets.snapshot_address
            where changesets.change_set_id = ?;
      `,
      bind: [changeSetId],
      returnValue: "resultRows",
    });
    const m = db.exec({
      sql: `select snapshots_mtm_atoms.* from snapshots_mtm_atoms
            inner join changesets
              on snapshots_mtm_atoms.snapshot_address = changesets.snapshot_address
            where changesets.change_set_id = ?;
      `,
      bind: [changeSetId],
      returnValue: "resultRows",
    });
    const a = db.exec({
      sql: `select atoms.* from atoms
            inner join snapshots_mtm_atoms
              on snapshots_mtm_atoms.kind = atoms.kind
              and snapshots_mtm_atoms.args = atoms.args
              and snapshots_mtm_atoms.checksum = atoms.checksum
            inner join changesets
              on snapshots_mtm_atoms.snapshot_address = changesets.snapshot_address
            where changesets.change_set_id = ?;
      `,
      bind: [changeSetId],
      returnValue: "resultRows",
    });
    const [changesets, snapshots, atoms, mtm] = await Promise.all([c, s, a, m]);
    return { changesets, snapshots, atoms, mtm };
  },
};

Comlink.expose(dbInterface);
