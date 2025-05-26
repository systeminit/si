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

export const mjolnirQueue = new PQueue({ concurrency: 10, autoStart: true });

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
