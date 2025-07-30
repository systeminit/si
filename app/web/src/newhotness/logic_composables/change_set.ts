import * as _ from "lodash-es";
import { computed, ComputedRef, inject, Ref, ref } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { WorkspaceMetadata } from "@/api/sdf/dal/workspace";
import { assertIsDefined, Context } from "../types";
import { routes, useApi } from "../api_composables";
import { useStatus } from "./status";

/**
 * USED in Workspace.vue
 * Which is why we are passing `ctx` and not using `inject`
 * Because Workspace is the one who `provides` it (catch-22)
 */
export const useChangeSets = (
  ctx: ComputedRef<Context>,
  enabled?: ComputedRef<boolean> | Ref<boolean>,
) => {
  const headChangeSetId = ref();
  const defaultApprovers = ref<string[]>([]);

  if (!enabled) enabled = ref(true);
  const changeSetApi = useApi(ctx.value);
  const changeSetQuery = useQuery<Record<string, ChangeSet>>({
    enabled: enabled.value,
    queryKey: ["changesets"],
    staleTime: 5000,
    queryFn: async () => {
      const call = changeSetApi.endpoint<WorkspaceMetadata>(routes.ChangeSets);
      const response = await call.get();
      if (changeSetApi.ok(response)) {
        const changeSets = _.keyBy(response.data.changeSets, "id");
        const head = changeSets[response.data.defaultChangeSetId];
        headChangeSetId.value = response.data.defaultChangeSetId;
        defaultApprovers.value = response.data.approvers;
        if (head) head.isHead = true;
        return changeSets;
      }
      return {} as Record<string, ChangeSet>;
    },
  });

  const openChangeSets = computed(() => {
    return Object.values(changeSetQuery.data.value ?? {}).filter((cs) =>
      [
        ChangeSetStatus.Open,
        ChangeSetStatus.NeedsApproval,
        ChangeSetStatus.NeedsAbandonApproval,
        ChangeSetStatus.Rejected,
        ChangeSetStatus.Approved,
      ].includes(cs.status),
    );
  });

  const changeSet = computed(() => {
    if (!changeSetQuery.data.value) return;
    return changeSetQuery.data.value[ctx.value.changeSetId.value];
  });

  return { openChangeSets, changeSet, headChangeSetId, defaultApprovers };
};

// other components use the Context that is populated with the above data
const useApplyChangeSetInner = (ctx: Context) => {
  const api = useApi(ctx);

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

  const status = useStatus();

  return computed(
    () =>
      ctx.changeSet.value?.status !== ChangeSetStatus.Open ||
      ctx.onHead.value ||
      status.value === "syncing",
  );
};

export const useApplyChangeSet = () => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);
  const applyChangeSet = useApplyChangeSetInner(ctx);
  const disableApplyChangeSet = useDisableApplyChangeSetInner();
  return { applyChangeSet, disableApplyChangeSet };
};
