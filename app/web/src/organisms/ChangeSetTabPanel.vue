<template>
  <div class="flex flex-row h-full">
    <!-- Filter button and list of components -->
    <div
      class="w-32 border-r-[1px] border-black text-center h-full flex flex-col"
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
          v-for="group in list"
          :key="group.component_id"
          class="flex flex-col text-sm"
        >
          <div
            v-if="selectedComponentId === group.component_id"
            class="bg-action-500 py-2"
            @click="updateSelectedComponentId(group.component_id)"
          >
            {{ group.component_name }}
          </div>

          <div
            v-else
            class="hover:bg-black py-2"
            @click="updateSelectedComponentId(group.component_id)"
          >
            {{ group.component_name }}
          </div>
        </div>
      </div>
    </div>

    <!-- Selected component view -->
    <div v-if="selectedComponentId">
      <div
        v-if="selectedComponentGroup === 'added'"
        class="text-success-300 text-center text-lg p-2 ml-4"
      >
        Component ID {{ selectedComponentId }} Added
      </div>
      <div
        v-else-if="selectedComponentGroup === 'deleted'"
        class="text-destructive-300 text-center text-lg px-2 py-1 ml-4"
      >
        Component ID {{ selectedComponentId }} Deleted
      </div>
      <div
        v-else-if="selectedComponentGroup === 'modified'"
        class="flex flex-row"
      >
        <CodeViewer
          font-size="12px"
          :component-id="selectedComponentId"
          class="text-neutral-50 mx-5"
          :code="diffCode"
          force-theme="dark"
          code-language="diff"
        />

        <CodeViewer
          font-size="12px"
          :component-id="selectedComponentId"
          class="text-neutral-50 mx-5"
          :code="currentCode"
          force-theme="dark"
          code-language="json"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ComponentStats } from "@/api/sdf/dal/change_set";
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

const filter = ref<"all" | "added" | "deleted" | "modified">("all");
const changeFilter = (newFilter: "all" | "added" | "deleted" | "modified") => {
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
    if (selectedComponentId.value) {
      for (const group of stats.value.added) {
        if (group.component_id === selectedComponentId.value) {
          return "added";
        }
      }
      for (const group of stats.value.deleted) {
        if (group.component_id === selectedComponentId.value) {
          return "deleted";
        }
      }
      for (const group of stats.value.modified) {
        if (group.component_id === selectedComponentId.value) {
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

const selectedComponentId = ref<number | null>(null);
const selectedComponentId$ = fromRef(selectedComponentId, { immediate: true });
const updateSelectedComponentId = (componentId: number) => {
  selectedComponentId.value = componentId;
};

const updateSelection = (node: Node | null) => {
  const componentId = node?.nodeKind?.componentId;

  // Ignores deselection and fake nodes, as they don't have any attributes
  if (!componentId || componentId === -1) return;

  selectedComponentId.value = componentId;
};
lastSelectedNode$
  .pipe(untilUnmounted)
  .subscribe((node) => updateSelection(node));
firstValueFrom(lastSelectedNode$).then((last) => updateSelection(last));

const currentCode = computed((): string => {
  if (componentDiff.value) {
    return componentDiff.value.current.code ?? "# No code found";
  }
  return "# Waiting for component diff...";
});

// FIXME(nick): allow for multiple diffs.
const diffCode = computed((): string => {
  if (componentDiff.value) {
    return componentDiff.value.diffs[0].code ?? "# No code found";
  }
  return "# Waiting for component diff...";
});

const componentDiff = refFrom<ComponentDiff | null>(
  combineLatest([selectedComponentId$]).pipe(
    switchMap(([selectedComponentId]) => {
      // Only collect the diff for modified components.
      if (selectedComponentId && selectedComponentGroup.value === "modified") {
        return ComponentService.getDiff({
          componentId: selectedComponentId,
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
