import { computed, inject } from "vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { assertIsDefined, Context } from "../types";
import { routes, useApi } from "../api_composables";
import { useStatus } from "./status";

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

const useApplyChangeSetInner = () => {
  const api = useApi();

  const loading = computed(() => api.inFlight.value);

  const performApply = async () => {
    const call = api.endpoint(routes.ApplyChangeSet);
    const { req } = await call.post({});
    return { success: api.ok(req) };
  };

  return { loading, performApply };
};

const useDisableApplyChangeSetInner = () => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);

  const currentChangeSet = useCurrentChangeSet();
  const status = useStatus();

  const changeSet = computed(() => currentChangeSet.value);

  return computed(
    () =>
      changeSet.value?.status !== ChangeSetStatus.Open ||
      ctx.onHead.value ||
      status.value === "syncing",
  );
};

export const useApplyChangeSet = () => {
  const applyChangeSet = useApplyChangeSetInner();
  const disableApplyChangeSet = useDisableApplyChangeSetInner();
  return { applyChangeSet, disableApplyChangeSet };
};
