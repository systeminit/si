import PQueue from "p-queue";
import { context, trace } from "@opentelemetry/api";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { BustCacheFn, Id } from "./types/dbinterface";
import { EntityKind } from "./types/entity_kind_types";

const _DEBUG = import.meta.env.VITE_SI_ENV === "local";
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
 * - once patches are done processing, process hammers
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
const bustQueue = new PQueue({ concurrency: 10, autoStart: true });

// de-dupe queue busting!
// except... we're never waiting to bust, so it doesnt really de-dupe
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
    _bustQueue.add(key);
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
let msgFlag = 0;
bustQueue.on("add", () => {
  if (msgFlag === 0 || msgFlag === -1) debug("🧹 busts queued", bustQueue.size);
  msgFlag = 1;
});
bustQueue.on("active", () => {
  if (msgFlag === 0 || msgFlag === -1) debug("🧹 busts queued", bustQueue.size);
  msgFlag = 1;
});

processPatchQueue.on("empty", () => {
  debug("⚙️ patches processed");
  const ctx = context.active();
  const span = trace.getSpan(ctx);
  if (span) span.setAttribute("processPatchQueueEmpty", true);
  // the queue may either be paused or running
  bustQueue.start();
});
processMjolnirQueue.on("empty", () => {
  debug("⚙️ mjolnir processed");
  const ctx = context.active();
  const span = trace.getSpan(ctx);
  if (span) span.setAttribute("processMjolnirQueueEmpty", true);
  // the queue may either be paused or running
  bustQueue.start();
});
bustQueue.on("empty", () => {
  if (msgFlag === 0 || msgFlag === 1) {
    debug("🧹 busts processed");
    const ctx = context.active();
    const span = trace.getSpan(ctx);
    if (span) span.setAttribute("bustQueueEmpty", true);
  }
  msgFlag = -1;
});

// ensure that when we get patches we process them fully
processPatchQueue.on("active", () => {
  processMjolnirQueue.pause();
});
// process any hammers after patches
processPatchQueue.on("idle", () => {
  processMjolnirQueue.start();
});

const inflight = new Set<string>();

// When we are running a bulk mjolnir, dont let other hammers fly...
// Hold them in a queue, and if the bulk fails, let them fly...
// if the bulk succeeds, don't fire them...
const _bulkInflight: Record<string, Set<string>> = {};

export const bulkInflight = (args: {
  workspaceId: string;
  changeSetId: string;
}) => {
  debug("BULK IN FLIGHT", args);
  bulkQueue.pause();
  processPatchQueue.pause();
  bustQueue.pause();
  let inflight = _bulkInflight[args.workspaceId];
  if (!inflight) {
    inflight = new Set<string>();
    _bulkInflight[args.workspaceId] = inflight;
  }
  inflight.add(args.changeSetId);
};
export const bulkDone = (
  args: { workspaceId: string; changeSetId: string },
  runQueue = false,
) => {
  debug("BULK DONE", args);
  let inflight = _bulkInflight[args.workspaceId];
  if (!inflight) {
    inflight = new Set<string>();
    _bulkInflight[args.workspaceId] = inflight;
  }
  inflight.delete(args.changeSetId);
  if (inflight.size === 0) {
    if (runQueue) {
      bulkQueue.start();
    } else bulkQueue.clear();
    if (processPatchQueue.size === 0) bustQueue.start();
    processPatchQueue.start();
  }
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
  const bulk = _bulkInflight[desc.workspaceId];
  if (bulk && bulk.size === 0) {
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
