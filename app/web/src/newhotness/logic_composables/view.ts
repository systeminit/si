import { computed, ComputedRef } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { getComponentsInViews, getComponentsInOnlyOneView, useMakeKey } from "@/store/realtime/heimdall";
import { EntityKind } from "@/workers/types/entity_kind_types";
import { ViewId } from "@/api/sdf/dal/views";
import { ComponentId } from "@/api/sdf/dal/component";
import { useContext } from "./context";

type UseComponentsAndViews = {
  componentsInViews: ComputedRef<Record<string, Set<string>>>;
  componentsInOnlyOneView: ComputedRef<Record<string, string>>;
};

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
    componentsInOnlyOneView: computed(() => componentsInOnlyOneViewQuery.data.value ?? {}),
  };
};

export interface AvailableViewListOptions {
  addToView: { label: string; value: string }[];
  removeFromView: { label: string; value: string }[];
}

export const availableViewListOptionsForComponentIds = (
  componentIds: ComponentId[],
  viewListOptions: { label: string; value: string }[],
  componentsAndViews: UseComponentsAndViews,
  showInvalidOptions = false,
) => {
  const unprocessedOptions = viewListOptions;
  unprocessedOptions.sort((a, b) => a.label.toLowerCase().localeCompare(b.label.toLowerCase()));
  const options: AvailableViewListOptions = {
    addToView: [],
    removeFromView: [],
  };
  for (const unprocessedOption of unprocessedOptions) {
    const viewId = unprocessedOption.value;
    const componentsInView = componentsAndViews.componentsInViews.value[viewId] ?? new Set();
    if (showViewInAddToViewMenuOptions(componentsInView, componentIds)) options.addToView.push(unprocessedOption);
    if (
      showViewInRemoveFromViewMenuOptions(
        componentsInView,
        viewId,
        componentIds,
        componentsAndViews,
        showInvalidOptions,
      )
    )
      options.removeFromView.push(unprocessedOption);
  }
  return options;
};
const showViewInAddToViewMenuOptions = (componentIdsInView: Set<ComponentId>, componentIds: Array<ComponentId>) => {
  // If there's nothing in the view, you always add to it.
  if (componentIdsInView.size < 1) return true;

  // For the selected components, if at least one of them is not in the view, we can add to it.
  for (const componentId of componentIds) {
    if (!componentIdsInView.has(componentId)) {
      return true;
    }
  }

  // If all of the components are in the view, we cannot add any of them to it.
  return false;
};
const showViewInRemoveFromViewMenuOptions = (
  componentIdsInView: Set<ComponentId>,
  viewId: ViewId,
  componentIds: Array<ComponentId>,
  componentsAndViews: UseComponentsAndViews,
  showInvalidOptions: boolean,
) => {
  // If there's nothing in the view, there's nothing to remove from it.
  if (componentIdsInView.size < 1) return false;

  // For the selected components, only show the option if all of them are in the view and that view
  // isn't the final view for any of the components.
  for (const componentId of componentIds) {
    const soleViewIdForCurrentComponent = componentsAndViews.componentsInOnlyOneView.value[componentId];
    if (!componentIdsInView.has(componentId) || (soleViewIdForCurrentComponent === viewId && !showInvalidOptions))
      return false;
  }

  // If we don't hit the exit clause, then we are good to include this option.
  return true;
};
