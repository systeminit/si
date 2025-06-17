import * as Comlink from "comlink";
import {
  TabDBInterface,
  BustCacheFn,
  PatchBatch,
  MessageKind,
  AtomMessage,
  NOROW,
} from "@/workers/types/dbinterface";
import { EntityKind } from "@/workers/types/entity_kind_types";

// setup a few things
const workerUrl =
  import.meta.env.VITE_SI_ENV === "local"
    ? "/src/workers/webworker.ts"
    : "webworker.js";

const bustTanStackCache: BustCacheFn = (
  _workspaceId: string,
  _changeSetId: string,
  _kind: string,
  _id: string,
) => {};

/**
 * TEST OUTPUT
 */
const logElm = document.querySelector("#logs > ul");
const errElm = document.querySelector("#errors > ul");

const createMsg = (msg: string) => {
  const li = document.createElement("li");
  li.appendChild(document.createTextNode(msg));
  return li;
};

const log = (msg: string) => {
  logElm?.append(createMsg(msg));
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const assert = (value: any, msg: string) => {
  if (value) return; // if true, don't log anything
  errElm?.append(createMsg(msg));
};
const done = () => {
  const elm = document.getElementById("timestamp");
  const stamp = document.createTextNode(new Date().toString());
  elm?.appendChild(stamp);
};

/**
 * THE TEST
 */

const fullDiagnosticTest = async (db: Comlink.Remote<TabDBInterface>) => {
  log("~~ DIAGNOSTIC STARTED ~~");
  const head = "HEAD";
  const workspace = "W";
  await db.exec({
    sql: `
        INSERT INTO indexes (checksum)
        VALUES (?);
      `,
    bind: ["HEAD"],
  });

  await db.exec({
    sql: `
        INSERT INTO changesets (change_set_id, workspace_id, index_checksum)
        VALUES (?, ?, ?);
      `,
    bind: [head, workspace, "HEAD"],
  });

  const testRecord = "testRecord" as EntityKind;
  await db.exec({
    sql: `
        INSERT INTO atoms (kind, args, checksum, data)
        VALUES (?, ?, ?, ?);
      `,
    bind: [
      testRecord,
      "testId1",
      "tr1",
      await db.encodeDocumentForDB({ id: 1, name: "test record 1" }),
    ],
  });

  await db.exec({
    sql: `
        INSERT INTO index_mtm_atoms (index_checksum, kind, args, checksum)
        VALUES (?, ?, ?, ?);
      `,
    bind: ["HEAD", testRecord, "testId1", "tr1"],
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
      await db.encodeDocumentForDB({ id: 2, name: "test record 2" }),
    ],
  });

  await db.exec({
    sql: `
        INSERT INTO index_mtm_atoms (index_checksum, kind, args, checksum)
        VALUES (?, ?, ?, ?);
      `,
    bind: ["HEAD", testRecord, "testId2", "tr2"],
  });

  const testList = "testList" as EntityKind;
  await db.exec({
    sql: `
        INSERT INTO atoms (kind, args, checksum, data)
        VALUES (?, ?, ?, ?);
      `,
    bind: [
      testList,
      "changeSetId",
      "tl1",
      await db.encodeDocumentForDB({
        list: [`${testRecord}:1:tr1`, `${testRecord}:2:tr2`],
      }),
    ],
  });

  await db.exec({
    sql: `
        INSERT INTO index_mtm_atoms (index_checksum, kind, args, checksum)
        VALUES (?, ?, ?, ?);
      `,
    bind: ["HEAD", testList, "changeSetId", "tl1"],
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
   * */

  const payload1: PatchBatch = {
    meta: {
      workspaceId: "W",
      changeSetId: "new_change_set",
      fromIndexChecksum: "HEAD",
      toIndexChecksum: "test_index_checksum_1",
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
  await db.handlePatchMessage(payload1);

  const confirm1 = await db.exec({
    sql: `SELECT count(index_checksum) FROM index_mtm_atoms WHERE index_checksum = ?;`,
    bind: ["HEAD"],
    returnValue: "resultRows",
  });
  const count_old_snapshot_atoms = await db.oneInOne(confirm1);
  // one for each original atom
  assert(
    count_old_snapshot_atoms === 3,
    `old indexes ${String(count_old_snapshot_atoms)} === 3`,
  );

  const confirm2 = await db.exec({
    sql: `SELECT count(index_checksum) FROM index_mtm_atoms WHERE index_checksum = ?;`,
    bind: ["test_index_checksum_1"],
    returnValue: "resultRows",
  });
  const count_new_snapshot_atoms = await db.oneInOne(confirm2);
  // copied mtm & the patched atom
  assert(
    count_new_snapshot_atoms === 3,
    `payload1 new indexes ${String(count_new_snapshot_atoms)} === 3`,
  );

  const confirm3 = await db.exec({
    sql: `SELECT count(*) FROM atoms;`,
    returnValue: "resultRows",
  });
  const count_atoms = await db.oneInOne(confirm3);
  // three original atoms, plus the new patched atom
  assert(count_atoms === 4, `payload1 atoms ${String(count_atoms)} === 4`);

  const new_atom_data = await db.exec({
    sql: `SELECT data FROM atoms WHERE checksum = ?`,
    bind: ["tr1-new-name"],
    returnValue: "resultRows",
  });
  const data = await db.oneInOne(new_atom_data);
  if (data === NOROW) throw new Error("Expected data, got nothing");
  const doc = await db.decodeDocumentFromDB(data as ArrayBuffer);
  assert(
    doc.id === 1 && doc.name === "new name",
    `Document doesn't match (${JSON.stringify(doc)})`,
  );

  const addressQuery = await db.exec({
    sql: "select index_checksum from changesets where change_set_id = ?;",
    bind: ["new_change_set"],
    returnValue: "resultRows",
  });
  const address = (await db.oneInOne(addressQuery)) as string;
  assert(
    address === "test_index_checksum_1",
    `Changeset address didn't move forward ${address}`,
  );

  log("~~ FIRST PAYLOAD SUCCESS ~~");

  /**
   * Second payload is merging that change to HEAD
   * */

  const payload2: PatchBatch = {
    meta: {
      workspaceId: "W",
      changeSetId: "HEAD",
      fromIndexChecksum: "HEAD",
      toIndexChecksum: "test_index_checksum_2",
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
  await db.handlePatchMessage(payload2);

  const confirm4 = await db.exec({
    sql: `SELECT count(index_checksum) FROM index_mtm_atoms WHERE index_checksum = ?;`,
    bind: ["HEAD"],
    returnValue: "resultRows",
  });
  const count_old_head_snapshot_atoms = await db.oneInOne(confirm4);
  // one for each original atom
  assert(
    count_old_head_snapshot_atoms === 0,
    `old head indexes ${String(count_old_head_snapshot_atoms)} === 0`,
  );

  const confirm5 = await db.exec({
    sql: `SELECT count(index_checksum) FROM index_mtm_atoms WHERE index_checksum != ?;`,
    bind: ["HEAD"],
    returnValue: "resultRows",
  });
  const count_new_snapshot_atoms_again = await db.oneInOne(confirm5);
  // copied mtm & the patched atom, 3 for the changeset, 3 for HEAD
  assert(
    count_new_snapshot_atoms_again === 3 * 2,
    `payload2 new indexes ${String(count_new_snapshot_atoms_again)} === 3*2`,
  );

  const confirm6 = await db.exec({
    sql: `SELECT count(*) FROM atoms;`,
    returnValue: "resultRows",
  });
  const count_atoms_one_more = await db.oneInOne(confirm6);
  // 3 original atoms, 1 patched atom
  assert(
    count_atoms_one_more === 4,
    `payload2 atoms ${String(count_atoms_one_more)} === 4`,
  );

  log("~~ SECOND PAYLOAD SUCCESS ~~");

  /**
   * Third thing that happens, closing out that changeSet
   * WE NEED AN EVENT TO TELL US THIS
   * */

  const removed_record_before = await db.exec({
    sql: `SELECT COUNT(index_checksum) FROM index_mtm_atoms WHERE checksum = ?`,
    bind: ["tr1-new-name"],
    returnValue: "resultRows",
  });
  const before = await db.oneInOne(removed_record_before);

  // there is only 1 mtm for the altered atom
  assert(before === 1, `Before state wrong ${removed_record_before}`);

  await db.pruneAtomsForClosedChangeSet("W", "new_change_set");
  const confirm7 = await db.exec({
    sql: `SELECT count(index_checksum) FROM index_mtm_atoms WHERE index_checksum != ?;`,
    bind: ["HEAD"],
    returnValue: "resultRows",
  });
  const count_snapshots_after_purge = await db.oneInOne(confirm7);
  // 3 for HEAD
  assert(
    count_snapshots_after_purge === 3,
    `remove new indexes ${String(count_snapshots_after_purge)} === 3`,
  );

  const confirm8 = await db.exec({
    sql: `SELECT count(*) FROM atoms;`,
    returnValue: "resultRows",
  });
  const count_atoms_after_purge = await db.oneInOne(confirm8);
  // back to 3 atoms, like original
  assert(
    count_atoms_after_purge === 3,
    `purge atoms ${String(count_atoms_after_purge)} === 3`,
  );

  const removed_record = await db.exec({
    sql: `SELECT COUNT(index_checksum) FROM index_mtm_atoms WHERE checksum = ?`,
    bind: ["tr1-new-name"],
    returnValue: "resultRows",
  });
  const removed = await db.oneInOne(removed_record);
  assert(
    removed === 0,
    `Expected removed is still here: ${removed?.toString()}`,
  );

  log("~~ PURGE SUCCESS ~~");

  /**
   * Fourth thing that happens, add a new view, remove an existing view
   * */

  const payload3: PatchBatch = {
    meta: {
      workspaceId: "W",
      changeSetId: "add_remove",
      fromIndexChecksum: "test_index_checksum_2",
      toIndexChecksum: "test_index_checksum_3",
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
  await db.handlePatchMessage(payload3);

  const added_record = await db.exec({
    sql: `SELECT data FROM atoms WHERE checksum = ?`,
    bind: ["tr3-add"],
    returnValue: "resultRows",
  });
  const added = await db.oneInOne(added_record);
  if (added === NOROW) throw new Error("Expected new record, got nothing");
  const added_doc = await db.decodeDocumentFromDB(added as ArrayBuffer);
  assert(
    added_doc.id === 3 && added_doc.name === "record 3",
    `Added document doesn't match (${JSON.stringify(added_doc)})`,
  );

  const modlist = await db.exec({
    sql: `SELECT data FROM atoms WHERE checksum = ?`,
    bind: ["tl1-add-remove"],
    returnValue: "resultRows",
  });
  const list = await db.oneInOne(modlist);
  if (list === NOROW) throw new Error("Expected list, got nothing");
  const list_doc = await db.decodeDocumentFromDB(list as ArrayBuffer);
  assert(
    list_doc.list[0] === `${testRecord}:2:tr2`,
    `List item 1 is wrong (${JSON.stringify(list_doc)})`,
  );
  assert(
    list_doc.list[1] === `${testRecord}:3:tr3-add`,
    `List item 2 is wrong (${JSON.stringify(list_doc)})`,
  );

  const confirmCount = await db.exec({
    sql: `SELECT count(*) FROM atoms;`,
    returnValue: "resultRows",
  });
  const count_atoms_after_addremove = await db.oneInOne(confirmCount);

  assert(
    count_atoms_after_addremove === 5,
    `after mjolnir atom count ${String(count_atoms_after_addremove)} === 5`,
  );

  log("~~ ADD / REMOVE COMPLETED ~~");

  // test mjolnir!
  const hammer1: AtomMessage = {
    kind: MessageKind.MJOLNIR,
    atom: {
      id: "fb1",
      kind: "foobar" as EntityKind,
      toChecksum: "fb1",
      workspaceId: "W",
      changeSetId: "add_remove",
      fromIndexChecksum: "test_index_checksum_3",
      toIndexChecksum: "test_index_checksum_3",
    },
    data: { foo: "bar" },
  };

  await db.handleHammer(hammer1);

  const query = await db.exec({
    sql: "select args from atoms where kind = ? and args = ? and checksum = ?",
    bind: ["foobar", "fb1", "fb1"],
    returnValue: "resultRows",
  });
  const fb = await db.oneInOne(query);
  assert(fb === "fb1", "Mjolnir atom doesn't exist");

  const confirm9 = await db.exec({
    sql: `SELECT count(*) FROM atoms;`,
    returnValue: "resultRows",
  });
  const count_atoms_after_hammer = await db.oneInOne(confirm9);

  assert(
    count_atoms_after_hammer === 6,
    `after mjolnir atom count ${String(count_atoms_after_hammer)} === 6`,
  );

  const addressQuery2 = await db.exec({
    sql: "select index_checksum from changesets where change_set_id = ?;",
    bind: ["add_remove"],
    returnValue: "resultRows",
  });
  const address2 = (await db.oneInOne(addressQuery2)) as string;
  assert(
    address2 === "test_index_checksum_3",
    `Changeset address didn't move forward ${address2}`,
  );

  log("~~ MJOLNIR COMPLETED ~~");

  try {
    await db.ragnarok("W", "empty_list", true);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } catch (err: any) {
    assert(!err, err.toString());
  }
  log("~~ RAGNAROK COMPLETED ~~");

  log("~~ DIAGNOSTIC COMPLETED ~~");
  done();
};

/**
 * THE INVOCATION
 */

async function go() {
  const worker = new Worker(new URL(workerUrl, import.meta.url), {
    type: "module",
  });
  const db: Comlink.Remote<TabDBInterface> = Comlink.wrap(worker);
  db.addListenerBustCache(Comlink.proxy(bustTanStackCache));
  await db.initDB(true);
  await db.migrate(true);
  fullDiagnosticTest(db);
}

go();
