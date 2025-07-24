import { computed, ComputedRef, inject, ref } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { FuncKind, FuncRun } from "../api_composables/func_run";
import { ManagementFuncJobState } from "../api_composables/management_func_job_state";
import { useApi, routes } from "../api_composables";
import { assertIsDefined, Context } from "../types";

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
