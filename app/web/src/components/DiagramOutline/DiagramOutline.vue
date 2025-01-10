<template>
  <div ref="outlineRef" class="flex flex-col diagram-outline">
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle
          icon="bullet-list-indented"
          iconIsButton
          :selected="leftDrawerOpen"
          @click="toggleDrawer"
        >
          <template #label>
            <div class="flex flex-row gap-xs items-center">
              <div>{{ viewStore.outlinerView?.name ?? "Diagram" }} Outline</div>
              <PillCounter
                :count="componentCount"
                hideIfZero
                :paddingX="componentCount < 10 ? 'xs' : '2xs'"
              />
            </div>
          </template>
          <div class="flex flex-row gap-xs items-center">
            <Icon
              v-if="
                initialFetch.isPending ||
                actionsAreRunning ||
                viewFetch.isPending
              "
              tone="action"
              name="loader"
              size="md"
            />
          </div>
        </SidebarSubpanelTitle>

        <!-- search bar - dont need to show if no components -->
        <SiSearch
          v-if="rootComponents.length"
          ref="searchRef"
          placeholder="search components"
          :filters="searchFiltersWithCounts"
          @search="onSearchUpdated"
        />
      </template>

      <!-- filtered / search mode -->
      <template v-if="filterModeActive">
        <DiagramOutlineNode
          v-for="component in filteredComponents"
          :key="component.def.id"
          :component="component"
        />
      </template>

      <!-- tree mode -->
      <template v-else>
        <div v-if="!rootComponents.length" class="flex flex-col items-center">
          <div class="w-52">
            <EmptyStateIcon name="no-components" />
          </div>
          <div class="text-xl text-neutral-400 dark:text-neutral-300 mt-2">
            Drag & Drop
          </div>
          <div class="text-sm px-xs pt-3 text-neutral-400 text-center italic">
            Drag & Drop assets on to the canvas and start modeling your
            infrastructure
          </div>
          <div class="text-sm px-xs pt-3 text-neutral-400 text-center italic">
            Assets are reusable infrastructure components such as key pairs,
            docker images EC2 instances etc.
          </div>
        </div>
        <template v-else>
          <DiagramOutlineNode
            v-for="component in rootComponents"
            :key="component.def.id"
            :component="component"
          />
        </template>
      </template>
    </ScrollArea>
  </div>
</template>

<script lang="ts">
type DiagramOutlineRootCtx = {
  filterModeActive: ComputedRef<boolean>;
  itemClickHandler: (
    e: MouseEvent,
    component: DiagramNodeData | DiagramGroupData,
    tabSlug?: string,
  ) => void;
};

export const DiagramOutlineCtxInjectionKey: InjectionKey<DiagramOutlineRootCtx> =
  Symbol("DiagramOutlineContext");

export function useDiagramOutlineContext() {
  const ctx = inject(DiagramOutlineCtxInjectionKey, null);
  if (!ctx)
    throw new Error(
      "<DiagramOutlineNode> should only be used within a <DiagramOutline>",
    );
  return ctx;
}
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import {
  computed,
  ComputedRef,
  inject,
  InjectionKey,
  onBeforeUnmount,
  onMounted,
  provide,
  ref,
  watch,
} from "vue";
import * as _ from "lodash-es";
import {
  Icon,
  PillCounter,
  ScrollArea,
  SiSearch,
  Filter,
} from "@si/vue-lib/design-system";
import { windowListenerManager } from "@si/vue-lib";
import { useComponentsStore } from "@/store/components.store";
import { useViewsStore } from "@/store/views.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";

import { useQualificationsStore } from "@/store/qualifications.store";
import { ComponentId } from "@/api/sdf/dal/component";
import DiagramOutlineNode from "./DiagramOutlineNode.vue";
import EmptyStateIcon from "../EmptyStateIcon.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "../ModelingDiagram/diagram_types";

defineProps<{
  actionsAreRunning: boolean;
  toggleDrawer: () => void;
  leftDrawerOpen: boolean;
}>();

const searchRef = ref<InstanceType<typeof SiSearch>>();
const outlineRef = ref<HTMLElement>();

const emit = defineEmits<{
  // while we've avoided events for most things (selection, panning, etc)
  // we still have an emit for this one because the parent (WorkspaceModelAndView) owns the right click menu
  // and needs the raw MouseEvent
  (
    e: "right-click-item",
    ev: { mouse: MouseEvent; component: Component },
  ): void;
}>();

const componentsStore = useComponentsStore();
const viewStore = useViewsStore();
const qualificationsStore = useQualificationsStore();

const viewId = computed(
  () => viewStore.outlinerViewId || viewStore.selectedViewId,
);

const initialFetch = viewStore.getRequestStatus("FETCH_VIEW");
const viewFetch = viewStore.getRequestStatus("FETCH_VIEW_GEOMETRY", viewId);

const viewComponentIds = computed<ComponentId[] | null>(() => {
  if (viewId.value) {
    return Object.keys(
      viewStore.viewsById[viewId.value]?.components || [],
    ).concat(Object.keys(viewStore.viewsById[viewId.value]?.groups || []));
  } else return null;
});

const rootComponents = computed(() => {
  return Object.values(componentsStore.allComponentsById).filter((c) => {
    if (viewComponentIds.value !== null) {
      const ancestorsInView = c.def.ancestorIds?.some((a) =>
        viewComponentIds.value?.includes(a),
      );
      return (
        viewComponentIds.value.includes(c.def.id) &&
        (!c.def.parentId || !ancestorsInView)
      );
    } else return !c.def.parentId;
  });
});

type Component = DiagramNodeData | DiagramGroupData;

const componentsTreeFlattened = computed(() => {
  const flat: Array<Component> = [];
  const addAllChildren = (component: Component) => {
    if (viewComponentIds.value === null) flat.push(component);
    else if (viewComponentIds.value.includes(component.def.id))
      flat.push(component);

    const children = componentsStore.componentsByParentId[component.def.id];
    if (children) {
      children.forEach((child) => {
        if (
          viewComponentIds.value === null ||
          viewComponentIds.value.includes(child.def.id)
        )
          addAllChildren(child);
      });
    }
  };
  rootComponents.value.forEach((component) => {
    addAllChildren(component);
  });
  return flat;
});

const componentCount = computed(() => componentsTreeFlattened.value.length);

const searchString = ref("");
const searchStringCleaned = computed(() =>
  searchString.value.trim().toLowerCase(),
);
const filterModeActive = computed(
  () => !!(searchStringCleaned.value || searchRef.value?.filteringActive),
);

const filterComponentArrayBySearchString = (components: Component[]) => {
  return _.filter(components, (c) => {
    if (c.def.displayName.toLowerCase().includes(searchStringCleaned.value))
      return true;
    if (c.def.schemaName.toLowerCase().includes(searchStringCleaned.value))
      return true;
    return false;
  });
};

const filterComponentArrayBySearchStringAndFilters = (
  components: Component[],
) => {
  let _filteredComponents = filterComponentArrayBySearchString(components);

  if (searchRef.value?.filteringActive) {
    for (
      let index = 0;
      index < searchRef.value?.activeFilters.length;
      index++
    ) {
      if (searchRef.value?.activeFilters[index]) {
        _filteredComponents = _.filter(_filteredComponents, (component) =>
          filterArrays[index]?.value.includes(component),
        ) as Component[];
      }
    }
  }
  return _filteredComponents;
};

const filteredComponents = computed(() => {
  if (!filterModeActive.value) return [];
  return filterComponentArrayBySearchStringAndFilters(
    Object.values(componentsStore.allComponentsById).filter((c) =>
      viewComponentIds.value?.includes(c.def.id),
    ),
  );
});

function onSearchUpdated(newFilterString: string) {
  searchString.value = newFilterString;
}

const newComponents = computed(() =>
  componentsTreeFlattened.value.filter(
    (component) => component.def.changeStatus === "added",
  ),
);

const diffComponents = computed(() =>
  componentsTreeFlattened.value.filter(
    (component) => component.def.changeStatus === "modified",
  ),
);

const failedQualificationComponents = computed(() =>
  componentsTreeFlattened.value.filter(
    (component) =>
      qualificationsStore.qualificationStatusByComponentId[component.def.id] ===
      "failure",
  ),
);

const upgradableComponents = computed(() =>
  componentsTreeFlattened.value.filter(
    (component) => component.def.canBeUpgraded,
  ),
);

const searchFiltersWithCounts = computed(() => {
  const searchFilters: Array<Filter> = [
    {
      name: "New",
      iconTone: "success",
      iconName: "plus",
      count: filterComponentArrayBySearchStringAndFilters(newComponents.value)
        .length,
    },
    {
      name: "Diff",
      iconTone: "warning",
      iconName: "tilde",
      count: filterComponentArrayBySearchStringAndFilters(diffComponents.value)
        .length,
    },
    {
      name: "Qualifications",
      iconTone: "destructive",
      iconName: "x-hex-outline",
      count: filterComponentArrayBySearchStringAndFilters(
        failedQualificationComponents.value,
      ).length,
    },
    {
      name: "Upgrades Available",
      iconTone: "action",
      iconName: "bolt",
      count: filterComponentArrayBySearchStringAndFilters(
        upgradableComponents.value,
      ).length,
    },
    // TODO - Add filter for resource status
    // { name: "Resources", iconTone: "destructive", iconName: "x-hex", count: 0 },
  ];

  return searchFilters;
});

const filterArrays = [
  newComponents,
  diffComponents,
  failedQualificationComponents,
  upgradableComponents,
];

watch(
  () => viewStore.selectedComponentId,
  () => {
    if (!viewStore.selectedComponentId) return;
    const el = document.getElementById(
      `diagram-outline-node-${viewStore.selectedComponentId}`,
    );
    el?.scrollIntoView({ behavior: "smooth", block: "nearest" });
  },
);

function itemClickHandler(
  e: MouseEvent,
  component: Component,
  tabSlug?: string,
) {
  const shiftKeyBehavior = () => {
    const selectedComponentIds = viewStore.selectedComponentIds;

    if (selectedComponentIds.length === 0) {
      // If nothing is selected, select the current component
      viewStore.setSelectedComponentId(component.def.id);
    } else if (
      selectedComponentIds.length === 1 &&
      selectedComponentIds[0] === component.def.id
    ) {
      // If there's only one component selected and you clicked it, deselect it
      viewStore.setSelectedComponentId(null);
    } else {
      // Otherwise, attempt to select components between
      let components = componentsTreeFlattened.value;
      if (filterModeActive.value) {
        components = filteredComponents.value;
      }

      let indexFrom = components.findIndex((c) =>
        viewStore.selectedComponentIds.includes(c.def.id),
      );
      const indexTo = components.findIndex(
        (c) => c.def.id === component.def.id,
      );

      if (indexFrom > indexTo) {
        indexFrom = _.findLastIndex(components, (c) =>
          viewStore.selectedComponentIds.includes(c.def.id),
        );
        const selection = components
          .slice(indexTo, indexFrom + 1)
          .map((component) => component.def.id);
        viewStore.setSelectedComponentId(selection);
      } else if (indexFrom < indexTo) {
        const selection = components
          .slice(indexFrom, indexTo + 1)
          .map((component) => component.def.id);
        viewStore.setSelectedComponentId(selection);
      } else {
        viewStore.setSelectedComponentId(component.def.id);
      }
    }
  };

  // right click
  if (e.button === 2 || e.ctrlKey) {
    e.preventDefault();
    if (e.shiftKey) {
      shiftKeyBehavior();
    } else if (!viewStore.selectedComponentIds.includes(component.def.id)) {
      if (e.metaKey) {
        viewStore.setSelectedComponentId(component.def.id, {
          toggle: true,
        });
      } else {
        viewStore.setSelectedComponentId(component.def.id);
      }
    }
    emit("right-click-item", { mouse: e, component });
  } else if (e.shiftKey) {
    e.preventDefault();
    shiftKeyBehavior();
  } else if (e.metaKey) {
    e.preventDefault();
    viewStore.setSelectedComponentId(component.def.id, { toggle: true });
  } else if (e.type === "dblclick") {
    componentsStore.panTargetComponentId = component.def.id;
  } else {
    viewStore.setSelectedComponentId(component.def.id, {
      detailsTab: tabSlug,
    });
    componentsStore.panTargetComponentId = component.def.id;
  }
}

// this object gets provided to all child DiagramOutlineNode instances
// so we dont have to deal with propogating stuff through the tree
const rootCtx = {
  filterModeActive,
  itemClickHandler,
};
provide(DiagramOutlineCtxInjectionKey, rootCtx);

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});

const onKeyDown = (e: KeyboardEvent) => {
  if (document?.activeElement?.tagName !== "BODY") return;

  // Tab goes forwards, Shift-Tab goes backwards
  if (e.key === "Tab") {
    const selectedComponentId = _.last(viewStore.selectedComponentIds);
    if (!selectedComponentId) return;
    e.preventDefault();

    const diagramOutlineNodes = outlineRef.value?.querySelectorAll(
      ".diagram-outline-node",
    );
    const componentIds = _.map(diagramOutlineNodes, (node) =>
      node.getAttribute("data-component-id"),
    );
    const selectedIndex = componentIds.indexOf(selectedComponentId);

    let toSelectIndex = selectedIndex + (e.shiftKey ? -1 : 1);

    if (toSelectIndex < 0) {
      toSelectIndex = componentIds.length - 1;
    } else if (toSelectIndex === componentIds.length) {
      toSelectIndex = 0;
    }
    const toSelect = componentIds[toSelectIndex];
    if (toSelect) {
      viewStore.setSelectedComponentId(toSelect);
    }
  }
};

const onAddKeyDown = (e: KeyboardEvent) => {
  if (
    (e.key === "Escape" || e.key === "Backspace") &&
    viewStore.addComponentId
  ) {
    viewStore.cancelAdd();
    e.stopPropagation();
  }
};

onMounted(() => {
  windowListenerManager.addEventListener("keydown", onAddKeyDown, 5);
});

onBeforeUnmount(() => {
  windowListenerManager.removeEventListener("keydown", onAddKeyDown);
});
</script>
