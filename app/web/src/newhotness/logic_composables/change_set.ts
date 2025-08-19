import * as _ from "lodash-es";
import { computed, ComputedRef, Ref, ref } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { RouteLocationNormalizedLoadedGeneric, Router } from "vue-router";
import { ChangeSetId, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { WorkspaceMetadata } from "@/api/sdf/dal/workspace";
import { ApprovalData, Context, UserId } from "../types";
import { routes, useApi } from "../api_composables";
import { reset } from "./navigation_stack";

/**
 * USED in Workspace.vue
 * Which is why we are passing `ctx` and not using `inject`
 * Because Workspace is the one who `provides` it (catch-22)
 */
export const useChangeSets = (
  ctx: ComputedRef<Context>,
  enabled?: ComputedRef<boolean> | Ref<boolean>,
) => {
  if (!enabled) enabled = ref(true);
  const changeSetApi = useApi(ctx.value);
  const changeSetQuery = useQuery<WorkspaceMetadata | null>({
    enabled,
    queryKey: ["changesets"],
    staleTime: 5000,
    queryFn: async () => {
      const call = changeSetApi.endpoint<WorkspaceMetadata>(routes.ChangeSets);
      const response = await call.get();
      if (changeSetApi.ok(response)) {
        return response.data;
      }
      return null;
    },
  });

  const headChangeSetId = computed(() => {
    return changeSetQuery.data.value?.defaultChangeSetId ?? "";
  });
  const defaultApprovers = computed(
    () => changeSetQuery.data.value?.approvers || [],
  );

  const openChangeSets = computed(() => {
    const changeSets = _.keyBy(changeSetQuery.data.value?.changeSets, "id");
    return Object.values(changeSets).filter((cs) =>
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
    const changeSets = _.keyBy(changeSetQuery.data.value?.changeSets, "id");
    return changeSets[ctx.value.changeSetId.value];
  });

  return { openChangeSets, changeSet, headChangeSetId, defaultApprovers };
};

export const useApplyChangeSet = (ctx: Context) => {
  const api = useApi(ctx);

  const applyInFlight = computed(() => api.inFlight.value);

  const performApply = async () => {
    const call = api.endpoint(routes.ApplyChangeSet);
    const { req } = await call.post({});
    return { success: api.ok(req) };
  };

  return { performApply, applyInFlight };
};

export const navigateToExistingChangeSet = async (
  changeSetId: ChangeSetId,
  route: RouteLocationNormalizedLoadedGeneric,
  router: Router,
) => {
  const name = route.name;
  await router.push({
    name,
    params: {
      ...route.params,
      changeSetId,
    },
    query: route.query,
  });
  reset();
};

export const approverForChangeSet = (
  userId: UserId,
  approvalData: ApprovalData,
) =>
  approvalData.requirements.some((r) =>
    Object.values(r.approverGroups)
      .flat()
      .concat(r.approverIndividuals)
      .includes(userId),
  );
