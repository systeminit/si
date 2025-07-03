import { computed, inject } from "vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { assertIsDefined, Context } from "../types";

export const useCurrentChangeSet = () => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);

  // TODO(nick): yeet this! However, keeping it sandboxed here may help reduce usages in newhotness
  // components since we only want it for limited, surgical use.
  const changeSetsStore = useChangeSetsStore();

  // TODO(nick): do not rely on the change set store. Why do we do this instead of grabbing the
  // "selectedChangeSet"? As we move away from the old stores, we do not want to assume that the
  // "selectedChangeSet" will be in sync with the new UI. However, searching through open change
  // sets, like we are doing below, may be the safer approach as they are populated from simple fetch
  // calls and reactions to WsEvents. This should be less prone to drift than relying on the
  // "selectedChangeSet" being accurate.
  return computed(() =>
    changeSetsStore.openChangeSets.find((c) => c.id === ctx.changeSetId.value),
  );
};
