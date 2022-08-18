<template>
  <SiTabGroup :selected-index="0" :start-margin="4">
    <template #tabs>
      <SiTabHeader :key="0">FUNCTIONS</SiTabHeader>
    </template>
    <template #dropdownitems>
      <SiDropdownItem>FUNCTIONS</SiDropdownItem>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto">
        <SiSearch
          placeholder="search functions"
          auto-search
          @search="onSearch"
        />
        <div class="w-full text-neutral-400 dark:text-neutral-300 text-sm p-2">
          Select a function from the lists below to view or edit it.
        </div>
        <ul class="overflow-y-auto">
          <SiCollapsible
            label="Qualification Functions"
            as="li"
            content-as="ul"
            default-open
            class="w-full"
          >
            <li v-for="func in filteredList" :key="func.id">
              <SiFuncSprite
                :name="func.name"
                color="#921ed6"
                :class="
                  selectedFuncId == func.id
                    ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
                    : ''
                "
                class="border dark:border-neutral-600 dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
                :is-builtin="func.isBuiltin"
                @click="selectFunc(func)"
              />
            </li>
          </SiCollapsible>
        </ul>
      </TabPanel>
    </template>
  </SiTabGroup>
  <div
    class="absolute bottom-0 w-full h-12 text-right p-2 border-t border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800"
  >
    <SiButton
      icon="plus"
      kind="save"
      label="Create Function"
      size="lg"
      @click="createFunc"
    />
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiFuncSprite from "@/molecules/SiFuncSprite.vue";
import SiButton from "@/atoms/SiButton.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import { ListedFuncView, ListFuncsResponse } from "@/service/func/list_funcs";
import SiSearch from "@/molecules/SiSearch.vue";
import { TabPanel } from "@headlessui/vue";

const searchString = ref("");

const onSearch = (search: string) =>
  (searchString.value = search.trim().toLocaleLowerCase());

const props = defineProps<{
  funcList: ListFuncsResponse;
  selectedFuncId: number;
}>();

const selectedFunc = computed(() =>
  props.funcList.qualifications.find((f) => f.id === props.selectedFuncId),
);

const filteredList = computed(() => {
  const filteredList =
    searchString.value.length > 0
      ? props.funcList.qualifications.filter((f) =>
          f.name.toLocaleLowerCase().includes(searchString.value),
        )
      : props.funcList.qualifications;

  if (selectedFunc.value && !filteredList.includes(selectedFunc.value)) {
    filteredList.push(selectedFunc.value);
  }

  return filteredList;
});

const emits = defineEmits<{
  (e: "selectedFunc", v: ListedFuncView): void;
  (e: "createFunc"): void;
}>();

const selectFunc = (func: ListedFuncView) => {
  emits("selectedFunc", func);
};

const createFunc = () => {
  emits("createFunc");
};
</script>
