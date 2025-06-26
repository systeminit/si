import { computed, inject, reactive } from "vue";
import { DefaultMap } from "@/utils/defaultmap";
import { assertIsDefined, Context } from "../types";

const queueByChangeSet = new DefaultMap<string, Set<string>>(() => {
  return reactive(new Set<string>());
});

export const add = (changeSetId: string, desc: string) => {
  const queue = queueByChangeSet.get(changeSetId);
  if (queue) queue.add(desc);
};

export const remove = (changeSetId: string, desc: string) => {
  const queue = queueByChangeSet.get(changeSetId);
  if (queue) queue.delete(desc);
};

export const useRainbow = () => {
  return computed(() => {
    try {
      const ctx = inject<Context>("CONTEXT");
      assertIsDefined(ctx);

      const queue = queueByChangeSet.get(ctx.changeSetId.value);

      /**
       * This is a global "stuff is happening" counter
       * When its > 0 the system is waiting for data
       */
      return { count: computed(() => queue?.size ?? 0) };
    } catch (err) {
      return { count: 0 };
    }
  });
};
