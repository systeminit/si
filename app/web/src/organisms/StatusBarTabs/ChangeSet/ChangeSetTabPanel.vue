<template>
  <div class="flex flex-row h-full w-full">
    <!-- Filter button and list of components -->
    <div
      class="w-64 border-r-[1px] border-black text-center h-full flex flex-col"
    >
      <!-- Filter button and its dropdown -->
      <SiBarButton
        class="h-10 border-b-[1px] border-black"
        dropdown-classes="top-1 left-4"
        tooltip-text="Filter"
      >
        <template #default="{ hovered, open }">
          <div class="flex-row flex">
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
          :key="statsGroup.component_id"
          class="flex flex-col text-sm"
        >
          <div
            :class="
              selectedComponent?.component_id === statsGroup.component_id
                ? 'bg-action-500'
                : 'hover:bg-black'
            "
            class="py-2 text-left text-ellipsis truncate cursor-pointer flex flex-row"
            @click="updateSelectedComponent(statsGroup)"
          >
            <div class="w-5 ml-1 mr-2">
              <CheckIcon
                v-if="
                  selectedComponent?.component_id === statsGroup.component_id
                "
              />
            </div>
            {{ statsGroup.component_name }}
          </div>
        </div>
      </div>
    </div>

    <!-- Selected component view -->
    <div v-if="selectedComponent" class="w-full h-full flex flex-col">
      <div
        v-if="selectedComponentGroup === 'added'"
        class="flex flex-row items-center text-center w-full h-full"
      >
        <p class="w-full text-3xl text-success-300">Component Added</p>
      </div>
      <div
        v-else-if="selectedComponentGroup === 'deleted'"
        class="flex flex-row items-center text-center w-full h-full"
      >
        <p class="w-full text-3xl text-destructive-300">Component Deleted</p>
      </div>
      <div
        v-else-if="selectedComponentGroup === 'modified'"
        class="overflow-y-auto flex flex-row flex-wrap"
      >
        <div
          v-for="codeView in [diffCodeView, currentCodeView]"
          :key="codeView.title"
          class="basis-full lg:basis-1/2 overflow-hidden pr-10 pl-1 pt-2"
        >
          <!-- FIXME(nick): make code viewer height malleable if the status bar no longer a fixed size.
               1024 is the "min-width" for "lg" in tailwind, so we (maybe) should use it to detect if we need to add a height. -->
          <CodeViewer
            font-size="13px"
            height="260px"
            :component-id="selectedComponent.component_id"
            class="text-neutral-50 mx-5"
            :code="codeView.code"
            force-theme="dark"
            :code-language="codeView.language"
          >
            <template #title>
              <span class="text-lg">{{ codeView.title }}</span>
            </template>
          </CodeViewer>
        </div>
      </div>
    </div>
    <div v-else class="flex flex-row items-center text-center w-full h-full">
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
import { CheckIcon } from "@heroicons/vue/solid";
import { CodeLanguage } from "@/api/sdf/dal/code_view";

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

const list = computed(() => {
  if (filter.value === "all") {
    return total.value;
  } else if (filter.value === "added") {
    return stats.value.added;
  } else if (filter.value === "deleted") {
    return stats.value.deleted;
  }
  return stats.value.modified;
});

const total = computed(() => {
  return stats.value.added.concat(
    stats.value.deleted.concat(stats.value.modified),
  );
});

const stats = ref<ComponentStats>({
  added: [],
  deleted: [],
  modified: [],
});

const selectedComponentGroup = computed(
  (): "added" | "deleted" | "modified" | false => {
    if (selectedComponent.value) {
      for (const statsGroup of stats.value.added) {
        if (statsGroup.component_id === selectedComponent.value.component_id) {
          return "added";
        }
      }
      for (const statsGroup of stats.value.deleted) {
        if (statsGroup.component_id === selectedComponent.value.component_id) {
          return "deleted";
        }
      }
      for (const statsGroup of stats.value.modified) {
        if (statsGroup.component_id === selectedComponent.value.component_id) {
          return "modified";
        }
      }
    }
    return false;
  },
);

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

  selectedComponent.value = {
    component_id: componentId,
    component_name: node?.name,
  };
};

lastSelectedNode$
  .pipe(untilUnmounted)
  .subscribe((node) => updateSelection(node));
firstValueFrom(lastSelectedNode$).then((last) => updateSelection(last));

// We do not extend "CodeView" because the "code" field is required.
interface CodeViewWithTitle {
  code: string;
  language: CodeLanguage;
  title: string;
}

const currentCodeView = computed((): CodeViewWithTitle => {
  if (componentDiff.value) {
    if (componentDiff.value.current.code) {
      return {
        code: componentDiff.value.current.code,
        language: "json",
        title: "Current",
      };
    }
    return {
      code: "# No code found",
      language: "unknown",
      title: "Current",
    };
  }
  return {
    code: "# Waiting for component diff...",
    language: "unknown",
    title: "Current",
  };
});

// FIXME(nick): allow for multiple diffs.
const diffCodeView = computed((): CodeViewWithTitle => {
  if (componentDiff.value) {
    if (componentDiff.value.diffs[0].code) {
      return {
        code: componentDiff.value.diffs[0].code,
        language: "diff",
        title: "Diff",
      };
    }
    return {
      code: "# No code found",
      language: "unknown",
      title: "Diff",
    };
  }
  return {
    code: "# Waiting for component diff...",
    language: "unknown",
    title: "Diff",
  };
});

const componentDiff = refFrom<ComponentDiff | null>(
  combineLatest([selectedComponent$]).pipe(
    switchMap(([selectedComponent]) => {
      // Only collect the diff for modified components.
      if (selectedComponent && selectedComponentGroup.value === "modified") {
        return ComponentService.getDiff({
          componentId: selectedComponent.component_id,
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
