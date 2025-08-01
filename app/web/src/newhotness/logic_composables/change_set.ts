import * as _ from "lodash-es";
import { computed, ComputedRef, Ref, ref } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { RouteLocationNormalizedLoadedGeneric, Router } from "vue-router";
import {
  ChangeSet,
  ChangeSetId,
  ChangeSetStatus,
} from "@/api/sdf/dal/change_set";
import { WorkspaceMetadata } from "@/api/sdf/dal/workspace";
import { Context } from "../types";
import { routes, useApi } from "../api_composables";
import { useStatus } from "./status";
import { Ulid, UserId } from "../api_composables/si_id";
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
  const headChangeSetId = ref();
  const defaultApprovers = ref<string[]>([]);

  if (!enabled) enabled = ref(true);

  const changeSetApi = useApi(ctx.value);
  const changeSetQuery = useQuery<Record<string, ChangeSet>>({
    enabled,
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

  // TODO(nick): track for all change sets and not just the current one.
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

  const changeSetsNeedingApproval = computed(() => {
    return Object.values(changeSetQuery.data.value ?? {}).filter(
      (cs) => cs.status === ChangeSetStatus.NeedsApproval,
    );
  });

  const changeSet = computed(() => {
    if (!changeSetQuery.data.value) return;
    return changeSetQuery.data.value[ctx.value.changeSetId.value];
  });

  return {
    openChangeSets,
    changeSet,
    headChangeSetId,
    defaultApprovers,
    changeSetsNeedingApproval,
  };
};

export const useApprovalStatus = (ctx: ComputedRef<Context>) => {
  const approvalStatusApi = useApi(ctx.value);
  const approvalStatusQuery = useQuery<ApprovalData | undefined>({
    // enabled: () => ctx.approvalsEnabled.value && !ctx.workspaceHasOneUser.value,
    queryKey: ["approvalstatus", ctx.value.changeSetId.value],
    queryFn: async () => {
      const call = approvalStatusApi.endpoint<ApprovalData>(
        routes.ChangeSetApprovalStatus,
      );
      const response = await call.get();
      if (approvalStatusApi.ok(response)) {
        return response.data;
      }
      return undefined;
    },
  });
  return computed(() => approvalStatusQuery.data.value);
};

export const useApplyChangeSet = (ctx: Context) => {
  const api = useApi(ctx);

  const loading = computed(() => api.inFlight.value);

  const performApply = async () => {
    const call = api.endpoint(routes.ApplyChangeSet);
    const { req } = await call.post({});
    return { success: api.ok(req) };
  };

  return { loading, performApply };
};

export type ChangeSetApprovalId = string;

export interface ChangeSetApprovalRequirement {
  entityId: Ulid;
  entityKind: string;
  requiredCount: number;
  isSatisfied: boolean;
  applicableApprovalIds: ChangeSetApprovalId[];
  approverGroups: Record<string, string[]>;
  approverIndividuals: string[];
}

export type ApprovalStatus = "Approved" | "Rejected";

export interface ChangeSetApproval {
  id: ChangeSetApprovalId;
  userId: UserId;
  status: ApprovalStatus;
  isValid: boolean; // is this approval "out of date" based on the checksum
}

export interface ApprovalData {
  requirements: ChangeSetApprovalRequirement[];
  latestApprovals: ChangeSetApproval[];
}

export const approverForChangeSet = (
  userId: string,
  approvalData: ApprovalData,
) =>
  approvalData.requirements.some((r) =>
    Object.values(r.approverGroups)
      .flat()
      .concat(r.approverIndividuals)
      .includes(userId),
  );

export const useApproveOrReject = (ctx: Context) => {
  const perform = async (status: ApprovalStatus, changeSetId?: ChangeSetId) => {
    const newCtx = { ...ctx };
    if (changeSetId) {
      newCtx.changeSetId = computed(() => changeSetId);
      newCtx.onHead = computed(() => false); // TODO(nick): handle this in a way that is computed based on the change set id hacked in
    }
    const api = useApi(newCtx);

    const call = api.endpoint(routes.ChangeSetApprove);
    const { req } = await call.post({ status });
    return { success: api.ok(req) };
  };

  return perform;
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
