<template>
  <div ref="outlineRef" class="flex flex-col">
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle icon="bullet-list-indented">
          <template #label>
            <div class="flex flex-row gap-xs items-center">
              <div>Diagram Outline</div>
              <PillCounter :count="componentCount" borderTone="action" />
            </div>
          </template>
          <Icon
            v-if="fetchComponentsReq.isPending || actionsAreRunning"
            name="loader"
            size="md"
          />
        </SidebarSubpanelTitle>

        <!-- search bar - dont need to show if no components -->
        <SiSearch
          v-if="rootComponents.length"
          ref="searchRef"
          autoSearch
          :filters="searchFiltersWithCounts"
          @search="onSearchUpdated"
        />
      </template>

      <template v-if="fetchComponentsReq.isError">
        <ErrorMessage :requestStatus="fetchComponentsReq" />
      </template>
      <template v-else>
        <!-- filtered / search mode -->
        <template v-if="filterModeActive">
          <ComponentOutlineNode
            v-for="component in filteredComponents"
            :key="component.id"
            :componentId="component.id"
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
            <ComponentOutlineNode
              v-for="component in rootComponents"
              :key="component.id"
              :componentId="component.id"
            />
          </template>
        </template>
      </template>
    </ScrollArea>
  </div>
</template>

<script lang="ts">
type ComponentOutlineRootCtx = {
  filterModeActive: ComputedRef<boolean>;
  itemClickHandler: (e: MouseEvent, id: ComponentId, tabSlug?: string) => void;
};

export const ComponentOutlineCtxInjectionKey: InjectionKey<ComponentOutlineRootCtx> =
  Symbol("ComponentOutlineContext");

export function useComponentOutlineContext() {
  const ctx = inject(ComponentOutlineCtxInjectionKey, null);
  if (!ctx)
    throw new Error(
      "<ComponentOutlineNode> should only be used within a <ComponentOutline>",
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
} from "vue";
import * as _ from "lodash-es";
import {
  ErrorMessage,
  Icon,
  PillCounter,
  ScrollArea,
} from "@si/vue-lib/design-system";
import SiSearch, { Filter } from "@/components/SiSearch.vue";
import {
  ComponentId,
  useComponentsStore,
  FullComponent,
} from "@/store/components.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";

import { useQualificationsStore } from "@/store/qualifications.store";
import ComponentOutlineNode from "./ComponentOutlineNode.vue";
import EmptyStateIcon from "../EmptyStateIcon.vue";

defineProps<{ actionsAreRunning: boolean }>();

const searchRef = ref<InstanceType<typeof SiSearch>>();
const outlineRef = ref<HTMLElement>();

const emit = defineEmits<{
  // while we've avoided events for most things (selection, panning, etc)
  // we still have an emit for this one because the parent (WorkspaceModelAndView) owns the right click menu
  // and needs the raw MouseEvent
  (
    e: "right-click-item",
    ev: { mouse: MouseEvent; component: FullComponent },
  ): void;
}>();

const componentsStore = useComponentsStore();
const qualificationsStore = useQualificationsStore();

const fetchComponentsReq =
  componentsStore.getRequestStatus("FETCH_DIAGRAM_DATA");

const rootComponents = computed(() => {
  return _.filter(componentsStore.allComponents, (c) => !c.parentId);
});

const componentsTreeFlattened = computed(() => {
  const flat: Array<FullComponent> = [];
  const addAllChildren = (component: FullComponent) => {
    flat.push(component);
    const children = componentsStore.componentsByParentId[component.id];
    if (children) {
      children.forEach((child) => {
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

const filterComponentArrayBySearchString = (components: FullComponent[]) => {
  return _.filter(components, (c) => {
    if (c.displayName.toLowerCase().includes(searchStringCleaned.value))
      return true;
    if (c.schemaName.toLowerCase().includes(searchStringCleaned.value))
      return true;
    return false;
  });
};

const filterComponentArrayBySearchStringAndFilters = (
  components: FullComponent[],
) => {
  let filteredComponents = filterComponentArrayBySearchString(components);

  if (searchRef.value?.filteringActive) {
    searchFiltersWithCounts.value.forEach((filter, index) => {
      if (searchRef.value?.activeFilters[index]) {
        filteredComponents = _.filter(filteredComponents, (component) =>
          filterArrays[index]?.value.includes(component),
        ) as FullComponent[];
      }
    });
  }
  return filteredComponents;
};

const filteredComponents = computed(() => {
  if (!filterModeActive.value) return [];
  return filterComponentArrayBySearchStringAndFilters(
    componentsStore.allComponents,
  );
});

function onSearchUpdated(newFilterString: string) {
  searchString.value = newFilterString;
}

const newComponents = computed(() =>
  componentsTreeFlattened.value.filter(
    (component) => component.changeStatus === "added",
  ),
);

const diffComponents = computed(() =>
  componentsTreeFlattened.value.filter(
    (component) => component.changeStatus === "modified",
  ),
);

const failedQualificationComponents = computed(() =>
  componentsTreeFlattened.value.filter(
    (component) =>
      qualificationsStore.qualificationStatusByComponentId[component.id] ===
      "failure",
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
    // TODO - Add filter for resource status
    // { name: "Resources", iconTone: "destructive", iconName: "x-hex", count: 0 },
  ];

  return searchFilters;
});

const filterArrays = [
  newComponents,
  diffComponents,
  failedQualificationComponents,
];

function itemClickHandler(e: MouseEvent, id: ComponentId, tabSlug?: string) {
  const component = componentsStore.componentsById[id];
  if (!component) throw new Error("component not found");

  const shiftKeyBehavior = () => {
    const selectedComponentIds = componentsStore.selectedComponentIds;

    if (selectedComponentIds.length === 0) {
      // If nothing is selected, select the current component
      componentsStore.setSelectedComponentId(id);
    } else if (
      selectedComponentIds.length === 1 &&
      selectedComponentIds[0] === id
    ) {
      // If there's only one component selected and you clicked it, deselect it
      componentsStore.setSelectedComponentId(null);
    } else {
      // Otherwise, attempt to select components between
      let components = componentsTreeFlattened.value;
      if (filterModeActive.value) {
        components = filteredComponents.value;
      }

      let indexFrom = components.findIndex((c) =>
        componentsStore.selectedComponentIds.includes(c.id),
      );
      const indexTo = components.findIndex((c) => c.id === id);

      if (indexFrom > indexTo) {
        indexFrom = _.findLastIndex(components, (c) =>
          componentsStore.selectedComponentIds.includes(c.id),
        );
        const selection = components
          .slice(indexTo, indexFrom + 1)
          .map((component) => component.id);
        componentsStore.setSelectedComponentId(selection);
      } else if (indexFrom < indexTo) {
        const selection = components
          .slice(indexFrom, indexTo + 1)
          .map((component) => component.id);
        componentsStore.setSelectedComponentId(selection);
      } else {
        componentsStore.setSelectedComponentId(id);
      }
    }
  };

  // right click
  if (e.button === 2 || e.ctrlKey) {
    e.preventDefault();
    if (e.shiftKey) {
      shiftKeyBehavior();
    } else if (!componentsStore.selectedComponentIds.includes(id)) {
      if (e.metaKey) {
        componentsStore.setSelectedComponentId(id, { toggle: true });
      } else {
        componentsStore.setSelectedComponentId(id);
      }
    }
    emit("right-click-item", { mouse: e, component });
  } else if (e.shiftKey) {
    e.preventDefault();
    shiftKeyBehavior();
  } else if (e.metaKey) {
    e.preventDefault();
    componentsStore.setSelectedComponentId(id, { toggle: true });
  } else if (e.type === "dblclick") {
    componentsStore.eventBus.emit("panToComponent", {
      componentId: id,
      center: true,
    });
  } else {
    componentsStore.setSelectedComponentId(id, { detailsTab: tabSlug });
    componentsStore.eventBus.emit("panToComponent", {
      componentId: id,
    });
  }
}

// this object gets provided to all child ComponentOutlineNode instances
// so we dont have to deal with propogating stuff through the tree
const rootCtx = {
  filterModeActive,
  itemClickHandler,
};
provide(ComponentOutlineCtxInjectionKey, rootCtx);

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
    const selectedComponentId = _.last(componentsStore.selectedComponentIds);
    if (!selectedComponentId) return;
    e.preventDefault();

    const componentOutlineNodes = outlineRef.value?.querySelectorAll(
      ".component-outline-node",
    );
    const componentIds = _.map(componentOutlineNodes, (node) =>
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
      componentsStore.setSelectedComponentId(toSelect);
    }
  }
};
</script>
