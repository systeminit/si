import { computed, ComputedRef, inject, ref } from "vue";
import { useInfiniteQuery, useQuery } from "@tanstack/vue-query";
import { ComponentId } from "@/api/sdf/dal/component";
import { FuncKind, FuncRun } from "../api_composables/func_run";
import { ManagementFuncJobState } from "../api_composables/management_func_job_state";
import { useApi, routes, funcRunTypes } from "../api_composables";
import { assertIsDefined, Context } from "../types";
import { useContext } from "./context";

export const useManagementFuncJobState = (
  funcRun: ComputedRef<FuncRun | undefined>,
) => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);

  const api = useApi();
  const pollInterval = ref<number | false>(0); // initial calls

  const { data } = useQuery<ManagementFuncJobState | undefined>({
    enabled: () => funcRun.value?.functionKind === FuncKind.Management,
    queryKey: computed(() => [
      ctx.changeSetId.value,
      "managementFuncJobStateByFuncRunId",
      funcRun.value?.id,
    ]),
    queryFn: async () => {
      if (!funcRun.value) return undefined;
      const call = api.endpoint<ManagementFuncJobState>(
        routes.MgmtFuncGetJobState,
        {
          funcRunId: funcRun.value.id,
        },
      );
      const req = await call.get();
      if (api.ok(req)) {
        pollInterval.value = ["executing", "operating", "pending"].includes(
          req.data.state,
        )
          ? 5000
          : false;
        return req.data;
      }
    },
    refetchInterval: () => pollInterval.value,
  });

  return computed(() => data);
};

export const useFuncRuns = () => {
  const ctx = useContext();
  const api = useApi();
  const pageSize = 50;

  const { data } = useInfiniteQuery({
    queryKey: [ctx.changeSetId, "paginatedFuncRuns"],
    queryFn: async ({
      pageParam = undefined,
    }): Promise<funcRunTypes.GetFuncRunsPaginatedResponse> => {
      const call = api.endpoint<funcRunTypes.GetFuncRunsPaginatedResponse>(
        routes.GetFuncRunsPaginated,
      );
      const params = new URLSearchParams();
      params.append("limit", pageSize.toString());
      if (pageParam) {
        params.append("cursor", pageParam);
      }
      const req = await call.get(params);
      if (api.ok(req)) {
        return req.data;
      }
      return {
        funcRuns: [],
        nextCursor: null,
      };
    },
    initialPageParam: undefined,
    getNextPageParam: (lastPage: funcRunTypes.GetFuncRunsPaginatedResponse) => {
      return lastPage.nextCursor ?? undefined;
    },
  });

  // Flatten the pages of function runs for display
  return computed<FuncRun[]>(() => {
    if (!data.value) return [];
    return data.value.pages.flatMap((page) => page.funcRuns);
  });
};

export const useLatestManagementComponentRuns = () => {
  const allFuncRuns = useFuncRuns();
  const managementFuncRuns = computed(() =>
    allFuncRuns.value.filter((r) => r.functionKind === FuncKind.Management),
  );
  return computed(() => {
    const result = {} as { [key in ComponentId]?: FuncRun };
    for (const run of managementFuncRuns.value) {
      result[run.componentId ?? "none"] ??= run;
    }
    return result;
  });
};
