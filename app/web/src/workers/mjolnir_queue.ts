import PQueue from "p-queue";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { BustCacheFn, Id } from "./types/dbinterface";
import { EntityKind } from "./types/entity_kind_types";

const _DEBUG = true; // import.meta.env.VITE_SI_ENV === "local";
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function debug(...args: any | any[]) {
  // eslint-disable-next-line no-console
  if (_DEBUG) console.debug(args);
}

/**
 * ASSUMPTIONS & HOW WE SEE THE SYSTEM BEHAVING
 * AND WHAT THAT MEANS FOR QUEUEING DATA
 *
 * Assumption: now that cold start does hammers in bulk, we expect very few hammers
 * in the general behavior of the system.
 * Follow up: We can further reduce these by taking more care what order we process
 * individual patches in
 *
 * We have observed that when patching, and we throw a hammer, the hammer returns with
 * a index ahead of patches we are currently applying. This breaks the updating scheme.
 * Therefore, we will hold hammers for processing until we have processed patches. (Above:
 * these hammers are mostly due to processing things in the wrong order, subsequent patches
 * in the batch contain the data)
 *
 * We have observed that a batch of patches does not always complete before the next batch
 * of patches, and it breaks the updating scheme.
 *
 * As a result of these operations we will:
 * - process 1 batch of patches at a time (processPatchQueue)
 * - while patches are processing, hold hammers
 * - once patches are done processing, bust cache and process hammers
 * - since we expect few hammers, we will each bust cache with each hammer
 * - fallback: if we have a few things to bust that are just waiting, bust them
 */

// only throw 10 hammers at a time. Increase this when the throwing is over WS
export const mjolnirQueue = new PQueue({ concurrency: 10, autoStart: true });
// holding tank for cold start
const bulkQueue = new PQueue({ concurrency: 10, autoStart: false });

// processing queues for the general lifecycle
export const processPatchQueue = new PQueue({
  concurrency: 1,
  autoStart: true,
  intervalCap: 1,
  carryoverConcurrencyCount: true,
});
export const processMjolnirQueue = new PQueue({
  concurrency: 1,
  autoStart: true,
  intervalCap: 1,
  carryoverConcurrencyCount: true,
});
const bustQueue = new PQueue({ autoStart: false });

// de-dupe queue busting!
const _bustQueue = new Set<string>();
export const bustQueueAdd = (
  workspaceId: string,
  changeSetId: string,
  kind: EntityKind,
  id: string,
  fn: BustCacheFn,
) => {
  const key = `${workspaceId}-${changeSetId}-${kind}-${id}`;
  if (!_bustQueue.has(key)) {
    bustQueue.add(() => {
      fn(workspaceId, changeSetId, kind, id);
      _bustQueue.delete(key);
    });
  }
};

processPatchQueue.on("add", () => {
  debug("⚙️ patches size", processPatchQueue.size);
});
processPatchQueue.on("active", () => {
  debug("⚙️ patches size", processPatchQueue.size);
});
processMjolnirQueue.on("add", () => {
  debug("⚙️ mjolnir size", processMjolnirQueue.size);
});
processMjolnirQueue.on("active", () => {
  debug("⚙️ mjolnir size", processMjolnirQueue.size);
});
bustQueue.on("add", () => {
  debug("🧹 busts size", bustQueue.size);
});
bustQueue.on("active", () => {
  debug("🧹 busts size", bustQueue.size);
});

processPatchQueue.on("empty", () => {
  debug("⚙️ patches processed");
});
processMjolnirQueue.on("empty", () => {
  debug("⚙️ mjolnir processed");
});
bustQueue.on("empty", () => {
  debug("🧹 busts processed");
});

// ensure that when we get patches we process them fully
processPatchQueue.on("active", () => {
  processMjolnirQueue.pause();
  bustQueue.pause();
});
// bust the queue and process any hammers after patches
processPatchQueue.on("idle", () => {
  bustQueue.start();
  processMjolnirQueue.start();
});

// if users are waiting for more than 5 things, bust query caches
// we don't want users wondering where their updates are!
bustQueue.on("add", () => {
  if (bustQueue.size > 5) bustQueue.start();
});

const inflight = new Set<string>();

// When we are running a bulk mjolnir, dont let other hammers fly...
// Hold them in a queue, and if the bulk fails, let them fly...
// if the bulk succeeds, don't fire them...
let _bulkInflight = false;

export const bulkInflight = () => {
  bulkQueue.pause();
  processPatchQueue.pause();
  bustQueue.pause();
  _bulkInflight = true;
};
export const bulkDone = (runQueue = false) => {
  _bulkInflight = false;
  if (runQueue) {
    bulkQueue.start();
  } else bulkQueue.clear();
  if (processPatchQueue.size === 0) bustQueue.start();
  processPatchQueue.start();
};

type Description = {
  workspaceId: string;
  changeSetId: ChangeSetId;
  kind: string;
  id: Id;
};

const descToString = (desc: Description) =>
  `${desc.workspaceId}-${desc.changeSetId}-${desc.kind}-${desc.id}`;

const canThrow = (desc: Description) => {
  const d = descToString(desc);
  if (inflight.has(d)) return false;
  else {
    inflight.add(d);
    debug("INFLIGHT", inflight.size);
    return true;
  }
};

export const hasReturned = (desc: Description) => {
  const d = descToString(desc);
  debug("RETURNED", d, "INFLIGHT", inflight.size);
  inflight.delete(d);
};

export const maybeMjolnir = async (desc: Description, fn: () => void) => {
  if (_bulkInflight) {
    await bulkQueue.add(fn);
    return;
  }
  if (canThrow(desc)) {
    try {
      await mjolnirQueue.add(fn);
    } catch (e) {
      // eslint-disable-next-line no-console
      console.error(`mjolnir job failed: ${e}`, desc, e);
      const d = descToString(desc);
      inflight.delete(d);
    }
  }
};
