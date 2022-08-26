<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">FUNCTIONS</SiTabHeader>
    </template>
    <template #dropdownitems>
      <SiDropdownItem>FUNCTIONS</SiDropdownItem>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-hidden flex flex-col">
        <div class="w-full p-2 border-b dark:border-neutral-600">
          <VButton
            button-rank="primary"
            button-type="success"
            class="ml-auto"
            icon="plus"
            icon-right="chevron-down"
            label="Function"
            size="sm"
            @click="createFunc"
          />
        </div>
        <SiSearch
          auto-search
          placeholder="search functions"
          @search="onSearch"
        />
        <div class="w-full text-neutral-400 dark:text-neutral-300 text-sm p-2">
          Select a function from the lists below to view or edit it.
        </div>
        <div class="overflow-auto">
          <ul class="overflow-y-auto">
            <SiCollapsible
              as="li"
              class="w-full"
              content-as="ul"
              default-open
              label="Qualification Functions"
            >
              <li v-for="func in filteredList" :key="func.id">
                <SiFuncSprite
                  :class="
                    selectedFuncId === func.id
                      ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
                      : ''
                  "
                  :is-builtin="func.isBuiltin"
                  :name="func.name"
                  class="border dark:border-neutral-600 dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
                  color="#921ed6"
                  @click="selectFunc(func)"
                />
              </li>
            </SiCollapsible>
          </ul>
        </div>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiFuncSprite from "@/molecules/SiFuncSprite.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import { ListedFuncView, ListFuncsResponse } from "@/service/func/list_funcs";
import SiSearch from "@/molecules/SiSearch.vue";
import { TabPanel } from "@headlessui/vue";
import VButton from "@/molecules/VButton.vue";

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
