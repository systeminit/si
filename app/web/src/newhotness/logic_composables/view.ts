import { computed } from "vue";
import { useQuery } from "@tanstack/vue-query";
import {
  getComponentsInViews,
  getComponentsInOnlyOneView,
  useMakeKey,
} from "@/store/realtime/heimdall";
import { EntityKind } from "@/workers/types/entity_kind_types";
import { ViewId } from "@/api/sdf/dal/views";
import { ComponentId } from "@/api/sdf/dal/component";
import { useContext } from "./context";

export const useComponentsAndViews = () => {
  const ctx = useContext();
  const key = useMakeKey();
  const args = computed(() => ({
    workspaceId: ctx.workspacePk.value,
    changeSetId: ctx.changeSetId.value,
  }));

  const componentsInViewsQuery = useQuery<Record<ViewId, Set<ComponentId>>>({
    queryKey: key(EntityKind.ComponentsInViews),
    queryFn: async () => await getComponentsInViews(args.value),
  });
  const componentsInOnlyOneViewQuery = useQuery<Record<ComponentId, ViewId>>({
    queryKey: key(EntityKind.ComponentsInOnlyOneView),
    queryFn: async () => await getComponentsInOnlyOneView(args.value),
  });

  return {
    componentsInViews: computed(() => componentsInViewsQuery.data.value ?? {}),
    componentsInOnlyOneView: computed(
      () => componentsInOnlyOneViewQuery.data.value ?? {},
    ),
  };
};
