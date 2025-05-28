import { computed, inject, reactive } from "vue";
import { DefaultMap } from "@/utils/defaultmap";
import { assertIsDefined, Context } from "../types";

const q = new DefaultMap<string, Set<string>>(() => new Set<string>());
const queueByChangeSet = reactive(q);

export const add = (changeSetId: string, desc: string) => {
  const queue = queueByChangeSet.get(changeSetId);
  if (queue) queue.add(desc);
};

export const remove = (changeSetId: string, desc: string) => {
  const queue = queueByChangeSet.get(changeSetId);
  if (queue) queue.delete(desc);
};

export const useRainbow = () => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);

  const queue = queueByChangeSet.get(ctx.changeSetId.value);

  /**
   * This is a global "stuff is happening" counter
   * When its > 0 the system is waiting for data
   */
  return { rainbowCount: computed(() => queue?.size ?? 0) };
};
