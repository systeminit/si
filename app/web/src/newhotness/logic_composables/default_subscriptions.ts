import { computed } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { getDefaultSubscriptions, useMakeKey } from "@/store/realtime/heimdall";
import {
  emptyDefaultSubs,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { useContext } from "./context";

export const useDefaultSubscription = () => {
  const ctx = useContext();
  const key = useMakeKey();
  const args = computed(() => ({
    workspaceId: ctx.workspacePk.value,
    changeSetId: ctx.changeSetId.value,
  }));

  const query = useQuery({
    queryKey: key(EntityKind.DefaultSubscriptions),
    enabled: ctx.queriesEnabled,
    queryFn: async () => await getDefaultSubscriptions(args.value),
  });

  return computed(() => query.data.value ?? emptyDefaultSubs);
};
