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
  EddaSecret,
  SecretDefinition,
  BifrostSecret,
  EddaSecretDefinitionList,
  BifrostSecretDefinitionList,
  EddaSecretList,
  BifrostSecretList,
} from "./types/entity_kind_types";
import { hasReturned, maybeMjolnir } from "./mjolnir_queue";

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
 * SNAPSHOT LOGIC
 *
 */

const atomExistsOnSnapshots = async (
  atom: Atom,
  checksum: Checksum,
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
    bind: [atom.kind, atom.id, checksum],
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
  await createAtom(atom, afterDoc, span);
  return afterDoc;
};

const createAtom = async (atom: Atom, doc: object, _span?: Span) => {
  debug("createAtom", atom, doc);

  const encodedDoc = await encodeDocumentForDB(doc);
  try {
    await db.exec({
      sql: `insert into atoms
        (kind, checksum, args, data)
          VALUES
        (?, ?, ?, ?)
        ON CONFLICT (kind, checksum, args)
        DO UPDATE SET data=excluded.data
      ;`,
      bind: [atom.kind, atom.toChecksum, atom.id, encodedDoc],
    });
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

const bustCacheAndReferences: BustCacheFn = async (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
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
        ref_kind as EntityKind,
        ref_id as string,
      );
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

  await updateChangeSetWithNewSnapshot(msg.atom);
  await removeOldSnapshot();

  await bustCacheAndReferences(
    msg.atom.workspaceId,
    msg.atom.changeSetId,
    msg.atom.kind,
    msg.atom.id,
  );

  if (COMPUTED_KINDS.includes(msg.atom.kind))
    await updateComputed(
      msg.atom.workspaceId,
      msg.atom.changeSetId,
      msg.atom.kind,
      msg.data,
    );
};

const insertAtomMTM = async (atom: Atom, toSnapshotAddress: Checksum) => {
  try {
    const bind = [toSnapshotAddress, atom.kind, atom.id, atom.toChecksum];
    const exists = await db.exec({
      sql: `select snapshot_address, kind, args, checksum from snapshots_mtm_atoms
        where snapshot_address = ? and kind = ? and args = ? and checksum = ?
      ;`,
      bind,
      returnValue: "resultRows",
    });
    if (exists.length > 0) {
      return false; // no-op
    }

    await db.exec({
      sql: `insert into snapshots_mtm_atoms
        (snapshot_address, kind, args, checksum)
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
    throw new Ragnarok(
      "From Snapshot Doesn't Exist",
      workspaceId,
      changeSetId,
      fromSnapshotAddress,
      snapshotFromAddress,
    );

  if (snapshotExists === NOROW)
    // contains INSERT INTO SELECT FROM snapshot_address_mtm
    await newSnapshot(meta, snapshotFromAddress);

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
      span?.addEvent("ragnarok", {
        patchBatch: JSON.stringify(data),
        fromSnapAddress: err.fromSnapshotAddress,
        snapshotFromAddress: err.snapshotFromAddress,
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
  if (!toSnapshotAddress)
    throw new Error(`Expected snapshot for ${data.meta.snapshotToAddress}`);

  // non-list atoms
  // non-connections (e.g. components need to go before connections)
  const atomsToBust = await Promise.all(
    atoms
      .filter(
        (a) =>
          !a.kind.includes("List") && !a.kind.includes("IncomingConnection"),
      )
      .map(async (atom) => {
        return applyPatch(atom, toSnapshotAddress);
      }),
  );

  atomsToBust.forEach((atom) => {
    if (atom)
      bustCacheAndReferences(
        atom.workspaceId,
        atom.changeSetId,
        atom.kind,
        atom.id,
      );
  });

  // connections
  const connAtomsToBust = await Promise.all(
    atoms
      .filter((a) => a.kind.includes("IncomingConnection"))
      .map(async (atom) => {
        return await applyPatch(atom, toSnapshotAddress);
      }),
  );
  connAtomsToBust.forEach((atom) => {
    if (atom)
      bustCacheAndReferences(
        atom.workspaceId,
        atom.changeSetId,
        atom.kind,
        atom.id,
      );
  });

  // list items
  const listAtomsToBust = await Promise.all(
    atoms
      .filter((a) => a.kind.includes("List"))
      .map(async (atom) => {
        return applyPatch(atom, toSnapshotAddress);
      }),
  );

  listAtomsToBust.forEach((atom) => {
    if (atom)
      bustCacheAndReferences(
        atom.workspaceId,
        atom.changeSetId,
        atom.kind,
        atom.id,
      );
  });

  await updateChangeSetWithNewSnapshot(data.meta);
  await removeOldSnapshot();
};

const applyPatch = async (
  atom: Required<Atom>,
  toSnapshotAddress: Checksum,
) => {
  return await tracer.startActiveSpan("applyPatch", async (span) => {
    span.setAttribute("atom", JSON.stringify(atom));

    // if we have the change already don't do anything
    const snapshots = await atomExistsOnSnapshots(atom, atom.toChecksum);
    span.setAttribute("toChecksumSnapshots", JSON.stringify(snapshots));
    if (snapshots.includes(atom.snapshotToAddress)) {
      span.addEvent("noop");
      span.end();
      return;
    }

    // do we have a snapshot with the fromChecksum (without we cannot patch)
    const previousSnapshots = await atomExistsOnSnapshots(
      atom,
      atom.fromChecksum,
    );
    span.setAttribute("previousSnapshots", JSON.stringify(previousSnapshots));
    const exists = previousSnapshots.length > 0;
    span.setAttribute("exists", exists);

    let needToInsertMTM = false;
    let bustCache = false;
    let doc;
    if (atom.fromChecksum === "0") {
      if (!exists) {
        // if i already have it, this is a NOOP
        span.setAttribute("createAtomFromPatch", true);
        doc = await createAtomFromPatch(atom, span);
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
        doc = await patchAtom(atom);
        needToInsertMTM = true;
        bustCache = true;
      }
      // otherwise, fire the small hammer to get the full object
      else {
        span.addEvent("mjolnir", {
          atom: JSON.stringify(atom),
          previousSnapshots: JSON.stringify(previousSnapshots),
          toChecksumSnapshots: JSON.stringify(snapshots),
          source: "applyPatch",
        });
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
    span.setAttribute("needToInsertMTM", needToInsertMTM);
    if (needToInsertMTM) {
      const inserted = await insertAtomMTM(atom, toSnapshotAddress);
      span.setAttribute("insertedMTM", inserted);
    }
    span.end();

    if (doc && COMPUTED_KINDS.includes(atom.kind))
      await updateComputed(atom.workspaceId, atom.changeSetId, atom.kind, doc);

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
  return afterDoc;
};

const mjolnir = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: string,
  id: Id,
  checksum?: Checksum,
) => {
  maybeMjolnir({ workspaceId, changeSetId, kind, id }, () => {
    inFlight(changeSetId, `${kind}.${id}`);
    return mjolnirJob(workspaceId, changeSetId, kind, id, checksum);
  });
};

const mjolnirJob = async (
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

  returned(changeSetId, `${kind}.${id}`);
  hasReturned({
    workspaceId,
    changeSetId,
    kind,
    id,
  });

  if (!req) return; // 404

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
  await tracer.startActiveSpan("removeOldSnapshot", async (span) => {
    const deleteSnapshots = await db.exec({
      sql: `
        DELETE FROM snapshots
        WHERE address NOT IN (
          SELECT snapshot_address FROM changesets
        ) RETURNING *;
      `,
      returnValue: "resultRows",
    });
    const deleteAtoms = await db.exec({
      sql: `
        DELETE FROM atoms
        WHERE (kind, args, checksum) NOT IN (
          SELECT  kind, args, checksum  FROM snapshots_mtm_atoms
        ) returning atoms.kind, atoms.args, atoms.checksum;
      `,
      returnValue: "resultRows",
    });
    span.setAttributes({
      snapshots: JSON.stringify(deleteSnapshots),
      atoms: JSON.stringify(deleteAtoms),
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

/**
 *
 * LIFECYCLE EVENTS
 *
 */

const niflheim = async (workspaceId: string, changeSetId: ChangeSetId) => {
  await tracer.startActiveSpan("niflheim", async (span: Span) => {
    // build connections list based on data we have in the DB
    // connections list will rebuild as data comes in
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
        span.addEvent("mjolnir", {
          workspaceId,
          changeSetId,
          kind,
          id,
          checksum,
          source: "niflheim",
        });
        mjolnir(workspaceId, changeSetId, kind, id, checksum);
        numHammers++;
      }
    });
    compare.setAttribute("numHammers", numHammers);
    compare.end();

    span.end();
  });
};

const ragnarok = async (
  workspaceId: string,
  changeSetId: string,
  noColdStart = false,
) => {
  // get rid of the snapshots we have for this changeset
  await db.exec({
    sql: `delete from snapshots
          where address IN (
            select snapshot_address
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
  await db.exec({
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
  await db.exec({
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
    await db.exec({
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
    changeSetId,
  )) as BifrostComponentList | -1;

  if (data === -1) return;

  await Promise.all(
    data.components.map((c) =>
      updateComputed(workspaceId, changeSetId, EntityKind.Component, c, false),
    ),
  );
  // bust everything all at once on cold start
  await bustCacheAndReferences(
    workspaceId,
    changeSetId,
    EntityKind.PossibleConnections,
    changeSetId,
  );

  const list = (await get(
    workspaceId,
    changeSetId,
    EntityKind.IncomingConnectionsList,
    changeSetId,
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
    changeSetId,
  );
};

const updateComputed = async (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  doc: AtomDocument,
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
      false,
    );
    doc = result[0];
  }

  if (kind === EntityKind.IncomingConnections) {
    const data = doc as BifrostComponentConnections;
    data.incoming.forEach((incoming) => {
      const id = `${incoming.toAttributeValueId}-${incoming.fromAttributeValueId}`;
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
      if (av.value && av.path && prop) {
        const conn: PossibleConnection = {
          attributeValueId: av.id,
          value: av.value,
          path: av.path,
          name: prop.name,
          componentId: component.id,
          componentName: component.name,
          schemaName: component.schemaName,
          annotation: prop.kind,
        };
        conns[av.id] = conn;
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
      { kind: EntityKind.OutgoingConnections, args: changeSetId },
      { kind, args: id },
    );
    return atomDoc;
  }

  debug("🔗 computed operation", kind, id);

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
      { kind: "OutgoingConnections", args: changeSetId },
      { kind, args: id },
    );
    return data;
  } else if (kind === EntityKind.Component) {
    const data = atomDoc as BifrostComponent;
    data.outputCount = Object.values(connectionsById.get(id)).length;
    clearWeakReferences(changeSetId, { kind, args: id });
    weakReference(
      changeSetId,
      { kind: "OutgoingConnections", args: changeSetId },
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

  debug("🔗 reference query", kind, id);

  let hasReferenceError = false;

  if (kind === EntityKind.Secret) {
    const data = atomDoc as EddaSecret;
    const sd = (await get(
      workspaceId,
      changeSetId,
      data.definitionId.kind,
      data.definitionId.id,
      undefined,
      followComputed,
    )) as SecretDefinition | -1;

    if (sd === -1) {
      hasReferenceError = true;
      span.addEvent("mjolnir", {
        workspaceId,
        changeSetId,
        kind: data.definitionId.kind,
        id: data.definitionId.id,
        source: "getReferences",
        sourceKind: kind,
      });
      mjolnir(
        workspaceId,
        changeSetId,
        data.definitionId.kind,
        data.definitionId.id,
      );
    }
    weakReference(
      changeSetId,
      { kind: data.definitionId.kind, args: data.definitionId.id },
      { kind, args: data.id },
    );
    const secret: BifrostSecret = {
      ...data,
      definition: sd !== -1 ? sd : ({} as SecretDefinition),
    };
    span.end();
    return [secret, hasReferenceError];
  } else if (kind === EntityKind.SecretDefinitionList) {
    const data = atomDoc as EddaSecretDefinitionList;
    const maybeSDs = await Promise.all(
      data.secretDefinitions.map(async (d) => {
        const maybeDoc = (await get(
          workspaceId,
          changeSetId,
          d.kind,
          d.id,
          undefined,
          followComputed,
        )) as SecretDefinition | -1;
        if (maybeDoc === -1) {
          hasReferenceError = true;
          span.addEvent("mjolnir", {
            workspaceId,
            changeSetId,
            kind: d.kind,
            id: d.id,
            source: "getReferences",
            sourceKind: kind,
          });
          mjolnir(workspaceId, changeSetId, d.kind, d.id);
        }
        weakReference(
          changeSetId,
          { kind: d.kind, args: d.id },
          { kind, args: data.id },
        );
        return maybeDoc;
      }),
    );
    const secretDefinitions = maybeSDs.filter(
      (v): v is SecretDefinition => v !== -1 && v && "id" in v,
    );
    const list: BifrostSecretDefinitionList = {
      id: data.id,
      secretDefinitions,
    };
    span.end();
    return [list, hasReferenceError];
  } else if (kind === EntityKind.SecretList) {
    const data = atomDoc as EddaSecretList;
    const maybeS = await Promise.all(
      data.secrets.map(async (d) => {
        const maybeDoc = (await get(
          workspaceId,
          changeSetId,
          d.kind,
          d.id,
          undefined,
          followComputed,
        )) as BifrostSecret | -1;
        if (maybeDoc === -1) {
          hasReferenceError = true;
          span.addEvent("mjolnir", {
            workspaceId,
            changeSetId,
            kind: d.kind,
            id: d.id,
            source: "getReferences",
            sourceKind: kind,
          });
          mjolnir(workspaceId, changeSetId, d.kind, d.id);
        }
        weakReference(
          changeSetId,
          { kind: d.kind, args: d.id },
          { kind, args: data.id },
        );
        return maybeDoc;
      }),
    );
    const secrets = maybeS.filter(
      (v): v is BifrostSecret => v !== -1 && v && "id" in v,
    );
    const list: BifrostSecretList = {
      id: data.id,
      secrets,
    };
    span.end();
    return [list, hasReferenceError];
  } else if (kind === EntityKind.Component) {
    const data = atomDoc as EddaComponent;
    const sv = (await get(
      workspaceId,
      changeSetId,
      data.schemaVariantId.kind,
      data.schemaVariantId.id,
      data.schemaVariantId.checksum,
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
    const component: BifrostComponent = {
      ...data,
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
        const maybeDoc = (await get(workspaceId, changeSetId, c.kind, c.id)) as
          | BifrostComponentConnections
          | -1;
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

  try {
    const [docAndRefs, hasReferenceError] = await getReferences(
      atomDoc,
      workspaceId,
      changeSetId,
      kind,
      id,
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
            if (data.kind === MessageKind.PATCH) {
              if (!data.meta.snapshotFromAddress)
                // eslint-disable-next-line no-console
                console.error(
                  "ATTEMPTING TO PATCH BUT FROM SNAPSHOT IS MISSING",
                  data.meta,
                );
              await handlePatchMessage(data, span);
            } else if (data.kind === MessageKind.MJOLNIR) {
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
              await handleHammer(data, span);
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

  async addListenerInFlight(cb: RainbowFn) {
    inFlight = cb;
  },
  async addListenerReturned(cb: RainbowFn) {
    returned = cb;
  },

  get,
  getOutgoingConnectionsByComponentId,
  getConnectionByAnnotation,
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
  async linkNewChangeset(
    workspaceId,
    headChangeSet,
    changeSetId,
    workspaceSnapshotAddress,
  ): Promise<void> {
    try {
      const headRows = db.exec({
        sql: "select snapshot_address from changesets where workspace_id = ? and change_set_id = ? ;",
        bind: [workspaceId, headChangeSet],
        returnValue: "resultRows",
      });
      const headRow = oneInOne(headRows);
      if (headRow === NOROW) throw new Error("HEAD is missing");
      const currentSnapshotAddress = headRow;

      if (currentSnapshotAddress === workspaceSnapshotAddress) {
        debug("~ new change set, no-op");
        return; // NO-OP
      }

      await db.exec({
        sql: `INSERT INTO snapshots (address) VALUES (?) ON CONFLICT DO NOTHING;`,
        bind: [workspaceSnapshotAddress],
      });

      await db.exec({
        sql: `INSERT INTO snapshots_mtm_atoms
        SELECT
          ?, kind, args, checksum
        FROM snapshots_mtm_atoms
        WHERE
          snapshot_address = ?
        ON CONFLICT DO NOTHING`,
        bind: [workspaceSnapshotAddress, currentSnapshotAddress],
      });

      await db.exec({
        sql: "insert into changesets (change_set_id, workspace_id, snapshot_address) VALUES (?, ?, ?) ON CONFLICT DO NOTHING;",
        bind: [changeSetId, workspaceId, workspaceSnapshotAddress],
      });
    } catch (err) {
      // NOTE: all the `on conflict do nothing` can be removed
      // once a new change set only returns after its index has been created
      // this runs *clean* "in the background" (e.g. on a second client that didn't make the change set)
      // eslint-disable-next-line no-console
      console.error("linkNewChangeset", err);
    }

    // hit the index to populate it
    // also shouldn't need this
    const pattern = [
      "v2",
      "workspaces",
      { workspaceId },
      "change-sets",
      { changeSetId },
      "index",
    ] as URLPattern;

    const [url, _desc] = describePattern(pattern);
    await sdf<IndexObjectMeta>({
      method: "get",
      url,
    });
  },
};

Comlink.expose(dbInterface);
