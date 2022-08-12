<template>
  <SiTabGroup
    :selected-index="0"
    tab-list-classes="h-11 flex shrink-0 w-full bg-white dark:bg-neutral-800 sticky top-0 z-50 overflow-hidden"
  >
    <template #tabs>
      <div
        class="w-2 border-b border-neutral-300 dark:border-neutral-600"
      ></div>
      <SiTabHeader
        :key="0"
        classes="border-x border-t border-x-neutral-300 border-t-neutral-300 dark:border-x-neutral-600 dark:border-t-neutral-600 h-11 px-4 text-sm inline-flex items-center rounded-t"
        selected-classes="border-b-white dark:border-b-neutral-800 border-b-2"
        default-classes="text-gray-400 border-b border-neutral-300 dark:border-neutral-600"
      >
        FUNCTIONS
      </SiTabHeader>
      <div
        class="grow border-b border-neutral-300 dark:border-neutral-600"
      ></div>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto">
        <SiSearch placeholder="search functions" />
        <div class="w-full text-neutral-400 text-sm p-2">
          Give this qualification a name, EntryPoint and brief description
          below.
        </div>
        <ul class="overflow-y-auto">
          <SiCollapsible
            label="Qualification Functions"
            as="li"
            content-as="ul"
          >
            <li v-for="func in funcList.qualifications" :key="func.id">
              <SiFuncSprite
                :name="func.name"
                color="#921ed6"
                :class="selectedFuncId == func.id ? 'bg-action-500' : ''"
                class="border-b-2 dark:border-neutral-600 hover:bg-action-500 dark:text-white hover:text-white hover:cursor-pointer"
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
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiFuncSprite from "@/molecules/SiFuncSprite.vue";
import SiButton from "@/atoms/SiButton.vue";
import { ListedFuncView, ListFuncsResponse } from "@/service/func/list_funcs";
import SiSearch from "@/molecules/SiSearch.vue";
import { TabPanel } from "@headlessui/vue";

defineProps<{
  funcList: ListFuncsResponse;
  selectedFuncId: number;
}>();

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
