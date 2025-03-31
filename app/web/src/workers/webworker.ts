import * as Comlink from "comlink";
import { applyPatch as applyOperations } from "fast-json-patch";
import sqlite3InitModule, {
  Database,
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

const start = async (sqlite3: Sqlite3Static) => {
  db =
    "opfs" in sqlite3
      ? new sqlite3.oo1.OpfsDb("/si.sqlite3")
      : new sqlite3.oo1.DB("/si.sqlite3", "ct");
  debug(
    "opfs" in sqlite3
      ? `OPFS is available, created persisted database at ${db.filename}`
      : `OPFS is not available, created transient database ${db.filename}`,
  );
  await db.exec({ sql: "PRAGMA foreign_keys = ON;" });
};

const initializeSQLite = async () => {
  try {
    const sqlite3 = await sqlite3InitModule({ print: log, printErr: error });
    await start(sqlite3);
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
  `;
  return await db.exec({ sql });
};

// INTEGER is 8 bytes, not large enough to store ULIDs
// we'll go with string, though reading that putting the bytes as BLOBs would save space
const ensureTables = async () => {
  if (_START_FRESH) await dropTables();
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

  updateChangeSetWithNewSnapshot(msg.atom);
  removeOldSnapshot();

  if (bustCacheFn)
    bustCacheFn(
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
  await Promise.all(
    atoms.map(async (atom) => {
      await applyPatch(atom, toSnapshotAddress);
    }),
  );

  updateChangeSetWithNewSnapshot(data.meta);
  removeOldSnapshot();
};

const applyPatch = async (
  atom: Required<Atom>,
  toSnapshotAddress: Checksum,
) => {
  await tracer.startActiveSpan("applyPatch", async (span) => {
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
    if (bustCache && bustCacheFn)
      bustCacheFn(atom.workspaceId, atom.changeSetId, atom.kind, atom.id);
    span.end();
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

  tracer.startActiveSpan(`GET ${desc}`, async (span) => {
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
    span.end();
  });
};

const ragnarok = async (workspaceId: string, changeSetId: string) => {
  // get rid of the snapshots we have for this changeset
  await db.exec({
    sql: `delete from snapshots where address IN (select snapshot_address from changesets where workspace_id = ? and changeset_id = ? );`,
    bind: [workspaceId, changeSetId],
  });
  // remove the atoms we have for this change set
  await pruneAtomsForClosedChangeSet(workspaceId, changeSetId);
  // call for a cold start to re-populate
  await niflheim(workspaceId, changeSetId);
};

const get = async (
  workspaceId: string,
  changeSetId: ChangeSetId,
  kind: string,
  id: Id,
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
  const atomData = await db.exec({
    sql,
    bind,
    returnValue: "resultRows",
  });
  const data = oneInOne(atomData);
  debug("❓ sql get", bind, " returns ?", !(data === NOROW));
  if (data === NOROW) {
    mjolnir(workspaceId, changeSetId, kind, id);
    return -1;
  }
  const atomDoc = decodeDocumentFromDB(data as ArrayBuffer);
  debug("📄 atom doc", atomDoc);

  // THIS GETS REPLACED WITH AUTO-GEN CODE
  if (kind === "ViewList") {
    const rawList = atomDoc as RawViewList;
    const maybeViews = await Promise.all(
      rawList.views.map(async (v) => {
        return await get(workspaceId, changeSetId, v.kind, v.id);
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
  } else return atomDoc;
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
  async initDB() {
    return initializeSQLite();
  },

  async migrate() {
    const result = ensureTables();
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
    await Promise.all([this.initDB(), this.initSocket()]);
    await this.migrate();
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
  partialKeyFromKindAndId: partialKeyFromKindAndArgs,
  kindAndIdFromKey: kindAndArgsFromKey,
  mjolnir,
  atomChecksumsFor,
  pruneAtomsForClosedChangeSet,
  niflheim,

  changeSetExists: async (workspaceId: string, changeSetId: ChangeSetId) => {
    const row = await db.exec({
      sql: "select change_set_id from changesets where workspace_id = ? and change_set_id = ?",
      returnValue: "resultRows",
      bind: [workspaceId, changeSetId],
    });
    const cId = oneInOne(row);
    return cId === changeSetId;
  },

  async odin(): Promise<object> {
    const c = db.exec({
      sql: "select * from changesets;",
      returnValue: "resultRows",
    });
    const s = db.exec({
      sql: "select * from snapshots;",
      returnValue: "resultRows",
    });
    const a = db.exec({
      sql: "select * from atoms;",
      returnValue: "resultRows",
    });
    const m = db.exec({
      sql: "select * from snapshots_mtm_atoms;",
      returnValue: "resultRows",
    });
    const [changesets, snapshots, atoms, mtm] = await Promise.all([c, s, a, m]);
    return { changesets, snapshots, atoms, mtm };
  },

  /*
  async fullDiagnosticTest() {
    log("~~ DIAGNOSTIC STARTED ~~");
    const head = "HEAD";
    const workspace = "W";
    const checksum = "HEAD";
    await db.exec({
      sql: `
        INSERT INTO snapshots (address)
        VALUES (?);
      `,
      bind: [checksum],
    });

    await db.exec({
      sql: `
        INSERT INTO changesets (change_set_id, workspace_id, snapshot_address)
        VALUES (?, ?, ?);
      `,
      bind: [head, workspace, checksum],
    });

    const testRecord = "testRecord";
    await db.exec({
      sql: `
        INSERT INTO atoms (kind, args, checksum, data)
        VALUES (?, ?, ?, ?);
      `,
      bind: [
        testRecord,
        "testId1",
        "tr1",
        await encodeDocumentForDB({ id: 1, name: "test record 1" }),
      ],
    });

    await db.exec({
      sql: `
        INSERT INTO snapshots_mtm_atoms (snapshot_address, kind, args, checksum)
        VALUES (?, ?, ?, ?);
      `,
      bind: [checksum, testRecord, "testId1", "tr1"],
    });

    await db.exec({
      sql: `
        INSERT INTO atoms (kind, args, checksum, data)
        VALUES (?, ?, ?, ?);
      `,
      bind: [
        testRecord,
        "testId2",
        "tr2",
        await encodeDocumentForDB({ id: 2, name: "test record 2" }),
      ],
    });

    await db.exec({
      sql: `
        INSERT INTO snapshots_mtm_atoms (snapshot_address, kind, args, checksum)
        VALUES (?, ?, ?, ?);
      `,
      bind: [checksum, testRecord, "testId2", "tr2"],
    });

    const testList = "testList";
    await db.exec({
      sql: `
        INSERT INTO atoms (kind, args, checksum, data)
        VALUES (?, ?, ?, ?);
      `,
      bind: [
        testList,
        "changeSetId",
        "tl1",
        await encodeDocumentForDB({
          list: [`${testRecord}:1:tr1`, `${testRecord}:2:tr2`],
        }),
      ],
    });

    await db.exec({
      sql: `
        INSERT INTO snapshots_mtm_atoms (snapshot_address, kind, args, checksum)
        VALUES (?, ?, ?, ?);
      `,
      bind: [checksum, testList, "changeSetId", "tl1"],
    });
    log("~~ FIXTURE COMPLETED ~~");

    /**
     * OK, the above code gives us 3 atoms that represent a list and two items within it
     * all hooked up to the snapshot and changeset tables
     *
     * Let's craft expected payloads over the web socket wire, and only call handle event
     * and assert we have the rows we expect to have!
     *
     * First payload is changing the name of a view

    const payload1: PatchBatch = {
      meta: {
        workspaceId: "W",
        changeSetId: "new_change_set",
        snapshotFromAddress: "HEAD",
        snapshotToAddress: "new_change_set",
      },
      kind: MessageKind.PATCH,
      patches: [
        {
          kind: testRecord,
          fromChecksum: "tr1",
          toChecksum: "tr1-new-name",
          patch: [{ op: "replace", path: "/name", value: "new name" }],
          id: "testId1",
        },
      ],
    };
    await tracer.startActiveSpan("handleEvent", async (span) => {
      await handlePatchMessage(payload1, span);
      span.end();
    });
    await assertUniqueAtoms();

    const confirm1 = await db.exec({
      sql: `SELECT count(snapshot_address) FROM snapshots_mtm_atoms WHERE snapshot_address = ?;`,
      bind: ["HEAD"],
      returnValue: "resultRows",
    });
    const count_old_snapshot_atoms = oneInOne(confirm1);
    // one for each original atom
    console.assert(
      count_old_snapshot_atoms === 3,
      `old snapshots ${String(count_old_snapshot_atoms)} === 3`,
    );

    const confirm2 = await db.exec({
      sql: `SELECT count(snapshot_address) FROM snapshots_mtm_atoms WHERE snapshot_address = ?;`,
      bind: ["new_change_set"],
      returnValue: "resultRows",
    });
    const count_new_snapshot_atoms = oneInOne(confirm2);
    // copied mtm & the patched atom
    console.assert(
      count_new_snapshot_atoms === 3,
      `new snapshots ${String(count_new_snapshot_atoms)} === 3`,
    );

    const confirm3 = await db.exec({
      sql: `SELECT count(*) FROM atoms;`,
      returnValue: "resultRows",
    });
    const count_atoms = oneInOne(confirm3);
    // three original atoms, plus the new patched atom
    console.assert(
      count_atoms === 4,
      `payload1 atoms ${String(count_atoms)} === 4`,
    );

    const new_atom_data = await db.exec({
      sql: `SELECT data FROM atoms WHERE checksum = ?`,
      bind: ["tr1-new-name"],
      returnValue: "resultRows",
    });
    const data = oneInOne(new_atom_data);
    if (data === NOROW) throw new Error("Expected data, got nothing");
    const doc = decodeDocumentFromDB(data as ArrayBuffer);
    console.assert(
      doc.id === 1 && doc.name === "new name",
      `Document doesn't match (${JSON.stringify(doc)})`,
    );

    const addressQuery = await db.exec({
      sql: "select snapshot_address from changesets where change_set_id = ?;",
      bind: ["new_change_set"],
      returnValue: "resultRows",
    });
    const address = oneInOne(addressQuery) as string;
    console.assert(
      address === "new_change_set",
      `Changeset address didn't move forward ${address}`,
    );

    log("~~ FIRST PAYLOAD SUCCESS ~~");

    /**
     * Second payload is merging that change to HEAD

    const payload2: PatchBatch = {
      meta: {
        workspaceId: "W",
        changeSetId: "HEAD",
        snapshotFromAddress: "HEAD",
        snapshotToAddress: "new_change_set_on_head",
      },
      kind: MessageKind.PATCH,
      patches: [
        {
          kind: testRecord,
          fromChecksum: "tr1",
          toChecksum: "tr1-new-name",
          patch: [{ op: "replace", path: "/name", value: "new name" }],
          id: "testId1",
        },
      ],
    };
    await tracer.startActiveSpan("handleEvent", async (span) => {
      await handlePatchMessage(payload2);
      span.end();
    });
    await assertUniqueAtoms();

    const confirm4 = await db.exec({
      sql: `SELECT count(snapshot_address) FROM snapshots_mtm_atoms WHERE snapshot_address = ?;`,
      bind: [checksum],
      returnValue: "resultRows",
    });
    const count_old_head_snapshot_atoms = oneInOne(confirm4);
    // one for each original atom
    console.assert(
      count_old_head_snapshot_atoms === 0,
      `old head snapshots ${String(count_old_head_snapshot_atoms)} === 0`,
    );

    const confirm5 = await db.exec({
      sql: `SELECT count(snapshot_address) FROM snapshots_mtm_atoms WHERE snapshot_address != ?;`,
      bind: [checksum],
      returnValue: "resultRows",
    });
    const count_new_snapshot_atoms_again = oneInOne(confirm5);
    // copied mtm & the patched atom, 3 for the changeset, 3 for HEAD
    console.assert(
      count_new_snapshot_atoms_again === 3 * 2,
      `new snapshots ${String(count_new_snapshot_atoms_again)} === 3*2`,
    );

    const confirm6 = await db.exec({
      sql: `SELECT count(*) FROM atoms;`,
      returnValue: "resultRows",
    });
    const count_atoms_no_change = oneInOne(confirm6);
    // same number of atoms no change
    console.assert(
      count_atoms_no_change === 3,
      `payload2 atoms ${String(count_atoms_no_change)} === 3`,
    );

    log("~~ SECOND PAYLOAD SUCCESS ~~");

    /**
     * Third thing that happens, closing out that changeSet
     * WE NEED AN EVENT TO TELL US THIS

    const removed_record_before = await db.exec({
      sql: `SELECT COUNT(snapshot_address) FROM snapshots_mtm_atoms WHERE checksum = ?`,
      bind: ["tr1-new-name"],
      returnValue: "resultRows",
    });
    const before = oneInOne(removed_record_before);

    console.assert(before === 2, `Before state wrong ${removed_record_before}`);

    await pruneAtomsForClosedChangeSet("W", "new_change_set");
    const confirm7 = await db.exec({
      sql: `SELECT count(snapshot_address) FROM snapshots_mtm_atoms WHERE snapshot_address != ?;`,
      bind: [checksum],
      returnValue: "resultRows",
    });
    const count_snapshots_after_purge = oneInOne(confirm7);
    // 3 for HEAD
    console.assert(
      count_snapshots_after_purge === 3,
      `new snapshots ${String(count_snapshots_after_purge)} === 3`,
    );

    const confirm8 = await db.exec({
      sql: `SELECT count(*) FROM atoms;`,
      returnValue: "resultRows",
    });
    const count_atoms_after_purge = oneInOne(confirm8);
    // back to 3 atoms, like original
    console.assert(
      count_atoms_after_purge === 3,
      `purge atoms ${String(count_atoms_after_purge)} === 3`,
    );

    const removed_record = await db.exec({
      sql: `SELECT COUNT(snapshot_address) FROM snapshots_mtm_atoms WHERE checksum = ?`,
      bind: ["tr1-new-name"],
      returnValue: "resultRows",
    });
    const removed = oneInOne(removed_record);
    console.assert(removed === 1, "Expected removed is still here");

    log("~~ PURGE SUCCESS ~~");

    /**
     * Fourth thing that happens, add a new view, remove an existing view


    const payload3: PatchBatch = {
      meta: {
        workspaceId: "W",
        changeSetId: "add_remove",
        snapshotFromAddress: "new_change_set_on_head",
        snapshotToAddress: "add_remove_1",
      },
      kind: MessageKind.PATCH,
      patches: [
        {
          kind: testRecord,
          fromChecksum: "0",
          toChecksum: "tr3-add",
          patch: [
            { op: "add", path: "/name", value: "record 3" },
            { op: "add", path: "/id", value: 3 },
          ],
          id: "testId3",
        },
        {
          kind: testRecord,
          fromChecksum: "tr1-new-name",
          toChecksum: "0",
          patch: [],
          id: "testId1",
        },
        {
          kind: testList,
          fromChecksum: "tl1",
          toChecksum: "tl1-add-remove",
          patch: [
            { op: "remove", path: "/list/0" },
            { op: "add", path: "/list/2", value: `${testRecord}:3:tr3-add` },
          ],
          id: "changeSetId",
        },
      ],
    };
    await tracer.startActiveSpan("handleEvent", async (span) => {
      await handlePatchMessage(payload3);
      span.end();
    });
    await assertUniqueAtoms();

    const added_record = await db.exec({
      sql: `SELECT data FROM atoms WHERE checksum = ?`,
      bind: ["tr3-add"],
      returnValue: "resultRows",
    });
    const added = oneInOne(added_record);
    if (added === NOROW) throw new Error("Expected new record, got nothing");
    const added_doc = decodeDocumentFromDB(added as ArrayBuffer);
    console.assert(
      added_doc.id === 3 && added_doc.name === "record 3",
      `Added document doesn't match (${JSON.stringify(added_doc)})`,
    );

    const modlist = await db.exec({
      sql: `SELECT data FROM atoms WHERE checksum = ?`,
      bind: ["tl1-add-remove"],
      returnValue: "resultRows",
    });
    const list = oneInOne(modlist);
    if (list === NOROW) throw new Error("Expected list, got nothing");
    const list_doc = decodeDocumentFromDB(list as ArrayBuffer);
    console.assert(
      list_doc.list[0] === `${testRecord}:2:tr2`,
      `List item 1 is wrong (${JSON.stringify(list_doc)})`,
    );
    console.assert(
      list_doc.list[1] === `${testRecord}:3:tr3-add`,
      `List item 2 is wrong (${JSON.stringify(list_doc)})`,
    );

    const confirmCount = await db.exec({
      sql: `SELECT count(*) FROM atoms;`,
      returnValue: "resultRows",
    });
    const count_atoms_after_addremove = oneInOne(confirmCount);

    console.assert(
      count_atoms_after_addremove === 5,
      `after mjolnir atom count ${String(count_atoms_after_addremove)} === 5`,
    );

    log("~~ ADD / REMOVE COMPLETED ~~");

    // test mjolnir!
    const hammer1: AtomMessage = {
      kind: MessageKind.MJOLNIR,
      atom: {
        id: "fb1",
        kind: "foobar",
        toChecksum: "fb1",
        workspaceId: "W",
        changeSetId: "add_remove",
        snapshotToAddress: "add_remove_1",
      },
      data: { foo: "bar" },
    };

    await tracer.startActiveSpan("handleEvent", async (span) => {
      await handleHammer(hammer1);
      span.end();
    });
    await assertUniqueAtoms();

    const query = await db.exec({
      sql: "select args from atoms where kind = ? and args = ? and checksum = ?",
      bind: ["foobar", "fb1", "fb1"],
      returnValue: "resultRows",
    });
    const fb = oneInOne(query);
    console.assert(fb === "fb1", "Mjolnir atom doesn't exist");

    const confirm9 = await db.exec({
      sql: `SELECT count(*) FROM atoms;`,
      returnValue: "resultRows",
    });
    const count_atoms_after_hammer = oneInOne(confirm9);

    console.assert(
      count_atoms_after_hammer === 6,
      `after mjolnir atom count ${String(count_atoms_after_hammer)} === 6`,
    );

    const addressQuery2 = await db.exec({
      sql: "select snapshot_address from changesets where change_set_id = ?;",
      bind: ["add_remove"],
      returnValue: "resultRows",
    });
    const address2 = oneInOne(addressQuery2) as string;
    console.assert(
      address2 === "add_remove_1",
      `Changeset address didn't move forward ${address2}`,
    );

    log("~~ MJOLNIR COMPLETED ~~");

    log("~~ DIAGNOSTIC COMPLETED ~~");
  },
  */
};

Comlink.expose(dbInterface);
