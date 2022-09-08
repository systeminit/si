<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">FUNCTIONS</SiTabHeader>
    </template>
    <template #dropdownitems>
      <SiDropdownItem>FUNCTIONS</SiDropdownItem>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto flex flex-col">
        <div class="w-full p-2 border-b dark:border-neutral-600">
          <Menu>
            <div class="block ml-auto w-fit">
              <MenuButton>
                <VButton
                  button-rank="primary"
                  button-type="success"
                  icon="plus"
                  icon-right="chevron--down"
                  label="Function"
                  size="sm"
                />
              </MenuButton>

              <MenuItems
                class="z-30 absolute mt-2 rounded bg-white dark:bg-black shadow-lg border focus:outline-none overflow-hidden"
              >
                <MenuItem
                  v-for="(kindName, kind) in funcCreateTypes"
                  :key="kind"
                  as="a"
                  class="flex flex-row relative items-center whitespace-nowrap py-2 px-4 cursor-pointer gap-2 hover:bg-action-500 hover:text-white"
                  @click="createFunc(kind)"
                >
                  <FuncSkeleton />

                  {{ kindName }}
                </MenuItem>
              </MenuItems>
            </div>
          </Menu>
        </div>
        <SiSearch
          auto-search
          placeholder="search functions"
          @search="onSearch"
        />
        <div
          class="w-full text-neutral-400 dark:text-neutral-300 text-sm p-2 border-b dark:border-neutral-600"
        >
          Select a function from the lists below to view or edit it.
        </div>
        <ul class="overflow-y-auto">
          <SiCollapsible
            v-for="(kindName, kind) in funcTypes"
            :key="kind"
            as="li"
            class="w-full"
            content-as="ul"
            default-open
          >
            <template #label>
              <div class="flex items-center gap-2">
                <FuncSkeleton />
                <span> {{ kindName }} </span>
              </div>
            </template>
            <template #default>
              <li
                v-for="func in filteredList.filter((f) => f.kind === kind)"
                :key="func.id"
              >
                <SiFuncSprite
                  :class="
                    selectedFuncId === func.id
                      ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
                      : ''
                  "
                  :is-builtin="func.isBuiltin"
                  :name="func.name"
                  class="border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
                  color="#921ed6"
                  @click="selectFunc(func)"
                />
              </li>
            </template>
          </SiCollapsible>
        </ul>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import {
  Menu,
  MenuButton,
  MenuItem,
  MenuItems,
  TabPanel,
} from "@headlessui/vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiFuncSprite from "@/molecules/SiFuncSprite.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import { ListedFuncView, ListFuncsResponse } from "@/service/func/list_funcs";
import SiSearch from "@/molecules/SiSearch.vue";
import VButton from "@/molecules/VButton.vue";
import FuncSkeleton from "@/atoms/FuncSkeleton.vue";
import { FuncBackendKind } from "@/api/sdf/dal/func";

const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const funcTypes = {
  [FuncBackendKind.JsQualification]: "Qualifications",
  [FuncBackendKind.JsAttribute]: "Attributes",
};

const funcCreateTypes = {
  [FuncBackendKind.JsQualification]: "Qualification",
  [FuncBackendKind.JsAttribute]: "Attribute",
};

const props = defineProps<{
  funcList: ListFuncsResponse;
  selectedFuncId: number;
}>();

const selectedFunc = computed(() =>
  props.funcList.funcs.find((f) => f.id === props.selectedFuncId),
);

const filteredList = computed(() => {
  const filteredList =
    searchString.value.length > 0
      ? props.funcList.funcs.filter((f) =>
          f.name.toLocaleLowerCase().includes(searchString.value),
        )
      : props.funcList.funcs;

  if (selectedFunc.value && !filteredList.includes(selectedFunc.value)) {
    filteredList.push(selectedFunc.value);
  }

  return filteredList;
});

const emits = defineEmits<{
  (e: "selectedFunc", v: ListedFuncView): void;
  (e: "createFunc", v: FuncBackendKind): void;
}>();

const selectFunc = (func: ListedFuncView) => {
  emits("selectedFunc", func);
};

const createFunc = (kind: FuncBackendKind) => {
  console.log(kind);
  emits("createFunc", kind);
};
</script>
