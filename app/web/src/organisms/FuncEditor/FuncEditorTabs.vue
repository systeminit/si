<template>
  <!-- border-b border-neutral-300 dark:border-neutral-600 -->
  <SiTabGroup :selected-index="selectedTab" @change="changeTab">
    <template #tabs>
      <SiTabHeader v-for="func in funcList" :key="func.id">
        {{ func.name }}
        <button
          class="inline-block rounded-sm w-5 ml-1"
          @click="closeFunc(func)"
        >
          <VueFeather type="x" />
        </button>
      </SiTabHeader>
    </template>
    <template #dropdownitems>
      <SiDropdownItem
        v-for="func in funcList"
        :key="func.id"
        :checked="findTabIndexForFunc(funcList, func) === selectedTab"
        @select="changeTab(findTabIndexForFunc(funcList, func))"
      >
        {{ func.name }}
      </SiDropdownItem>
    </template>
    <template #panels>
      <TabPanel
        v-for="func in funcList"
        :key="func.id"
        class="h-full overflow-auto"
      >
        <FuncEditor :func-id="func.id" />
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { toRef, computed } from "vue";
import { fromRef } from "vuse-rx/src";
import { FuncService } from "@/service/func";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import { TabPanel } from "@headlessui/vue";
import FuncEditor from "@/organisms/FuncEditor/FuncEditor.vue";
import VueFeather from "vue-feather";
import { ListedFuncView, nullListFunc } from "@/service/func/list_funcs";
import { switchMap, take } from "rxjs/operators";
import { of } from "rxjs";
import { funcState, funcById, removeFunc, insertFunc } from "./func_state";

const props = defineProps<{
  selectedFuncId: number;
}>();

const emits = defineEmits<{
  (e: "selectedFunc", v: ListedFuncView): void;
}>();

const selectFunc = (func: ListedFuncView) => {
  emits("selectedFunc", func);
};

const selectedFuncId = toRef(props, "selectedFuncId", 0);
const selectedFuncId$ = fromRef(selectedFuncId, { immediate: true });

const findTabIndexForFunc = (
  funcList: { id: number }[],
  func: { id: number },
) => funcList.findIndex((fn) => fn.id == func.id);

// We need the editingFuncs ref to manage updates to the observable,
// but we also want to map it into a list of functions for managing the
// list of tabs, hence the tap.
const funcList = computed(() =>
  funcState.funcs.map(({ origFunc, modifiedFunc }) => ({
    id: origFunc.id,
    handler: modifiedFunc.handler,
    name: modifiedFunc.name,
    kind: modifiedFunc.kind,
  })),
);

const selectedTab = computed(() =>
  findTabIndexForFunc(funcList.value, { id: selectedFuncId.value }),
);

const changeTab = (index: number) => {
  if (index > funcList.value.length - 1) {
    index--;
  }
  if (funcList.value.length) {
    selectFunc(funcList.value[index]);
  } else {
    selectFunc(nullListFunc);
  }
};

const closeFunc = (func: ListedFuncView) => {
  removeFunc(func);
};

selectedFuncId$
  .pipe(
    switchMap((selectedFuncId) => {
      const existingFunc = funcById(selectedFuncId);
      return existingFunc
        ? of({ ...existingFunc.origFunc })
        : FuncService.getFunc({ id: selectedFuncId }).pipe(take(1));
    }),
  )
  .subscribe(insertFunc);
</script>
