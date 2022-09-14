<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">WORKFLOWS</SiTabHeader>
    </template>
    <template #dropdownitems>
      <SiDropdownItem>WORKFLOWS</SiDropdownItem>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto">
        <SiSearch
          placeholder="search workflows"
          auto-search
          @search="onSearch"
        />
        <div
          class="w-full text-neutral-400 dark:text-neutral-300 text-sm p-2 border-b border-neutral-200 dark:border-neutral-600"
        >
          Select a workflow from the lists below to view or edit it.
        </div>
        <ul class="overflow-y-auto">
          <span v-for="(groups, schema) in groupedFilteredList" :key="schema">
            <SiCollapsible
              :label="String(schema)"
              as="li"
              content-as="ul"
              default-open
              class="w-full"
            >
              <span v-for="(workflows, component) in groups" :key="component">
                <template v-if="component === ''">
                  <span v-for="workflow in workflows" :key="workflow.id">
                    <SiWorkflowSprite
                      :name="workflow.title"
                      color="#921ed6"
                      :class="
                        selectedId === workflow.id
                          ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
                          : ''
                      "
                      class="border dark:border-neutral-600 dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
                      @click="select(workflow)"
                    />
                  </span>
                </template>
                <li v-else>
                  <SiCollapsible
                    :label="String(component)"
                    as="li"
                    content-as="ul"
                    default-open
                    class="w-full"
                  >
                    <span v-for="workflow in workflows" :key="workflow.id">
                      <SiWorkflowSprite
                        :name="workflow.title"
                        color="#921ed6"
                        :class="
                          selectedId === workflow.id
                            ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
                            : ''
                        "
                        class="border dark:border-neutral-600 dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
                        @click="select(workflow)"
                      />
                    </span>
                  </SiCollapsible>
                </li>
              </span>
            </SiCollapsible>
          </span>
        </ul>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { TabPanel } from "@headlessui/vue";
import SiWorkflowSprite from "@/molecules/SiWorkflowSprite.vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import {
  ListedWorkflowView,
  ListWorkflowsResponse,
} from "@/service/workflow/list";
import SiSearch from "@/molecules/SiSearch.vue";

const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const props = defineProps<{
  list: ListWorkflowsResponse;
  selectedId: number | null;
}>();

const selected = computed(() =>
  props.list.find((f) => f.id === props.selectedId),
);

const groupedFilteredList = computed(() => {
  const filteredList =
    searchString.value.length > 0
      ? props.list.filter((w) =>
          w.title.toLocaleLowerCase().includes(searchString.value),
        )
      : props.list;

  if (selected.value && !filteredList.includes(selected.value)) {
    filteredList.push(selected.value);
  }
  if (filteredList.length === 0) return {};

  const group: {
    [key: string]: { [key: string]: ListedWorkflowView[] };
  } = { Workspace: { "": [] } };

  for (const el of filteredList) {
    if (el.schemaName) {
      if (!group[el.schemaName]) group[el.schemaName] = {};
      if (el.componentNames.length === 0) {
        continue;
      } else if (!Array.isArray(group[el.schemaName])) {
        for (const componentName of el.componentNames) {
          if (!group[el.schemaName][componentName]) {
            group[el.schemaName][componentName] = [];
          }

          group[el.schemaName][componentName].push(el);
        }
      }
    } else {
      group.Workspace[""].push(el);
    }
  }

  return group;
});

const emits = defineEmits<{
  (e: "selected", v: ListedWorkflowView): void;
}>();

const select = (w: ListedWorkflowView) => {
  emits("selected", w);
};
</script>
