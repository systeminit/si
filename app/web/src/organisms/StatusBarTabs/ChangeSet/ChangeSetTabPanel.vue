<template>
  <div class="flex flex-row h-full w-full">
    <!-- Filter button and list of components -->
    <div
      class="w-80 border-r-[1px] border-shade-100 text-center h-full flex flex-col"
    >
      <!-- Filter button and its dropdown -->
      <SiBarButton
        class="h-10 border-b-[1px] text-shade-0 text-[1rem] border-shade-100"
        dropdown-classes="top-1 left-4"
        tooltip-text="Filter"
        fill-entire-width
      >
        <template #default="{ hovered, open }">
          <div class="flex flex-row justify-center">
            {{ filterTitle }}
            <SiArrow :nudge="hovered || open" class="ml-1 w-4" />
          </div>
        </template>

        <template #dropdownContent>
          <SiDropdownItem
            :checked="filter === 'all'"
            @select="changeFilter('all')"
            >All</SiDropdownItem
          >
          <SiDropdownItem
            :checked="filter === 'added'"
            @select="changeFilter('added')"
            >Added</SiDropdownItem
          >
          <SiDropdownItem
            :checked="filter === 'deleted'"
            @select="changeFilter('deleted')"
            >Deleted</SiDropdownItem
          >
          <SiDropdownItem
            :checked="filter === 'modified'"
            @select="changeFilter('modified')"
            >Modified</SiDropdownItem
          >
        </template>
      </SiBarButton>

      <!-- List of components -->
      <div class="overflow-y-auto flex-expand">
        <div
          v-for="statsGroup in list"
          :key="statsGroup.componentId"
          class="flex flex-col"
        >
          <div
            :class="
              selectedComponent?.componentId === statsGroup.componentId
                ? 'bg-action-500'
                : 'hover:bg-black'
            "
            class="py-2 truncate cursor-pointer flex flex-row justify-between"
            @click="updateSelectedComponent(statsGroup)"
          >
            <div class="text-left text-ellipsis ml-2.5 mr-6">
              {{ statsGroup.componentName }}
            </div>
            <StatusIndicatorIcon
              v-if="statsGroup"
              :status="statsGroup.componentStatus"
              class="w-6 mr-2.5 ml-6 text-right"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Selected component view -->
    <div
      v-if="selectedComponent"
      class="w-full h-full flex flex-col bg-shade-100"
    >
      <div
        v-if="selectedComponent.componentStatus === 'added'"
        class="overflow-y-auto flex flex-row flex-wrap"
      >
        <div class="basis-full overflow-hidden pr-10 pl-1 pt-2">
          <!-- FIXME(nick): make code viewer height malleable if the status bar no longer a fixed size.
               1024 is the "min-width" for "lg" in tailwind, so we (maybe) should use it to detect if we need to add a height. -->
          <CodeViewer
            font-size="13px"
            height="250px"
            :component-id="selectedComponent.componentId"
            class="text-neutral-50 mx-5"
            :code="codeRecord['Current']"
            force-theme="dark"
            :code-language="getCodeLanguage('Current')"
          >
            <template #title>
              <span class="text-lg">Current</span>
            </template>
          </CodeViewer>
        </div>
      </div>
      <div
        v-else-if="selectedComponent.componentStatus === 'deleted'"
        class="flex flex-row items-center text-center w-full h-full"
      >
        <p class="w-full text-3xl text-destructive-300">Component Deleted</p>
      </div>
      <div
        v-else-if="selectedComponent.componentStatus === 'modified'"
        class="overflow-y-auto flex flex-row flex-wrap"
      >
        <div
          v-for="title in ['Current', 'Diff']"
          :key="title"
          class="basis-full lg:basis-1/2 overflow-hidden pr-10 pl-1 pt-2"
        >
          <!-- FIXME(nick): make code viewer height malleable if the status bar no longer a fixed size.
               1024 is the "min-width" for "lg" in tailwind, so we (maybe) should use it to detect if we need to add a height. -->
          <CodeViewer
            font-size="13px"
            height="250px"
            :component-id="selectedComponent.componentId"
            class="text-neutral-50 mx-5"
            :code="codeRecord[title]"
            force-theme="dark"
            :code-language="getCodeLanguage(title)"
          >
            <template #title>
              <span class="text-lg">{{ title }}</span>
            </template>
          </CodeViewer>
        </div>
      </div>
    </div>
    <div
      v-else
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">No Component Selected</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ComponentStats, ComponentStatsGroup } from "@/api/sdf/dal/change_set";
import { lastSelectedNode$ } from "@/observable/selection";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { firstValueFrom, from, switchMap, map } from "rxjs";
import { computed, ref } from "vue";
import { fromRef, refFrom, untilUnmounted } from "vuse-rx";
import { Node } from "@/organisms/SiCanvas/canvas/obj/node";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import CodeViewer from "@/organisms/CodeViewer.vue";
import { combineLatest } from "rxjs";
import { ComponentService } from "@/service/component";
import { ComponentDiff } from "@/api/sdf/dal/component";
import _ from "lodash";
import StatusIndicatorIcon from "@/molecules/StatusIndicatorIcon.vue";

export type ChangeSetTabPanelFilter = "all" | "added" | "deleted" | "modified";

const filter = ref<ChangeSetTabPanelFilter>("all");
const changeFilter = (newFilter: ChangeSetTabPanelFilter) => {
  filter.value = newFilter;
};
const filterTitle = computed(() => {
  if (filter.value === "all") {
    return "All";
  } else if (filter.value === "added") {
    return "Added";
  } else if (filter.value === "deleted") {
    return "Deleted";
  }
  return "Modified";
});

const list = computed((): ComponentStatsGroup[] => {
  if (filter.value !== "all" && stats.value) {
    let list = [];
    for (const statsGroup of stats.value.stats) {
      if (statsGroup.componentStatus === filter.value) {
        list.push(statsGroup);
      }
    }
    return list;
  }
  return stats.value.stats;
});

const stats = ref<ComponentStats>({ stats: [] });

untilUnmounted(ChangeSetService.getStats()).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
  } else {
    stats.value = response.componentStats;
  }
});

const selectedComponent = ref<ComponentStatsGroup | null>(null);
const selectedComponent$ = fromRef(selectedComponent, { immediate: true });
const updateSelectedComponent = (statsGroup: ComponentStatsGroup) => {
  selectedComponent.value = statsGroup;
};

const updateSelection = (node: Node | null) => {
  const componentId = node?.nodeKind?.componentId;

  // Ignores deselection and fake nodes, as they don't have any attributes
  if (!componentId || componentId === -1) return;

  if (stats.value) {
    for (const statsGroup of stats.value.stats) {
      if (statsGroup.componentId === componentId) {
        selectedComponent.value = statsGroup;
        return;
      }
    }
  }

  // Unset the selected component if it is not in the stats list.
  selectedComponent.value = null;
};

lastSelectedNode$
  .pipe(untilUnmounted)
  .subscribe((node) => updateSelection(node));
firstValueFrom(lastSelectedNode$).then((last) => updateSelection(last));

// FIXME(nick): we should be using the "unknown" language if there's no code once mode switching is reactive.
const codeRecord = computed((): Record<string, string> => {
  let code = {
    Diff: "# Waiting for component diff...",
    Current: "# Waiting for component diff...",
  };
  if (componentDiff.value) {
    if (componentDiff.value.diffs.length > 0) {
      // FIXME(nick): allow for multiple diffs.
      code["Diff"] = componentDiff.value.diffs[0].code ?? "# No code found";
    } else {
      // This should never be read, but is here just in case.
      code["Diff"] = "# No diff found";
    }
    code["Current"] = componentDiff.value.current.code ?? "# No code found";
  }
  return code;
});

// FIXME(nick): remove this once reactivity is fixed.
const getCodeLanguage = (title: string) => {
  if (title === "Current") {
    return "json";
  }
  // Default to diff.
  return "diff";
};

const componentDiff = refFrom<ComponentDiff | null>(
  combineLatest([selectedComponent$]).pipe(
    switchMap(([selectedComponent]) => {
      if (selectedComponent) {
        return ComponentService.getDiff({
          componentId: selectedComponent.componentId,
        });
      }
      return from([null]);
    }),
    map((response) => {
      if (response) {
        if (response.error) {
          GlobalErrorService.set(response);
          return null;
        } else {
          return response.componentDiff;
        }
      }
      return null;
    }),
  ),
);
</script>
