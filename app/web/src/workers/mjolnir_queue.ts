import PQueue from "p-queue";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { Id } from "./types/dbinterface";

const _DEBUG = true; // import.meta.env.VITE_SI_ENV === "local";
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function debug(...args: any | any[]) {
  // eslint-disable-next-line no-console
  if (_DEBUG) console.debug(args);
}

const inflight = new Set<string>();

// When we are running a bulk mjolnir, dont let other hammers fly...
// Hold them in a queue, and if the bulk fails, let them fly...
// if the bulk succeeds, don't fire them...
let _bulkInflight = false;

export const bulkInflight = () => {
  bulkQueue.pause();
  _bulkInflight = true;
};
export const bulkDone = (runQueue = false) => {
  _bulkInflight = false;
  if (runQueue) {
    bulkQueue.start();
  } else bulkQueue.clear();
};

export const mjolnirQueue = new PQueue({ concurrency: 10, autoStart: true });
const bulkQueue = new PQueue({ concurrency: 10, autoStart: false });

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
