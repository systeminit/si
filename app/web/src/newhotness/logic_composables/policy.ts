import { useQuery } from "@tanstack/vue-query";
import { computed, ref } from "vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { routes, useApi } from "../api_composables";
import { useContext } from "./context";

export interface PagedResponse {
  pageSize: number;
  pageNumber: number;
  totalPageCount: number;
}

export interface PagedPolicies extends PagedResponse {
  reports: Policy[];
  totalReportCount: number;
}

export interface Policy {
  id: string;
  createdAt: IsoDateString;
  name: string;
  result: "Fail" | "Pass";
  // these are markdown
  policy: string;
  report: string;
}

export const usePolicy = () => {
  const ctx = useContext();
  const ffStore = useFeatureFlagsStore();

  const page = ref(1);

  const changeSetApi = useApi(ctx);

  const queryKey = computed(() => ["policies", page.value]);

  const policyQuery = useQuery<PagedPolicies | null>({
    enabled: ffStore.SHOW_POLICIES,
    queryKey,
    staleTime: 5000,
    queryFn: async () => {
      const call = changeSetApi.endpoint<PagedPolicies>(routes.PolicyReports);
      const params = new URLSearchParams();
      params.append("page", page.value.toString());
      const response = await call.get(params);
      if (changeSetApi.ok(response)) {
        return response.data;
      }
      return null;
    },
  });

  const policyReports = computed<Policy[]>(() =>
    [...(policyQuery.data.value?.reports || [])].sort((a, b) => {
      const ad = new Date(a.createdAt).getTime();
      const bd = new Date(b.createdAt).getTime();
      return ad - bd;
    }),
  );

  const maxPages = computed(() => policyQuery.data.value?.totalPageCount || 0);

  return { policyReports, page, maxPages };
};
