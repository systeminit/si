<template>
  <SiTabGroup
    :selected-index="selectedTab"
    tab-list-classes="h-11 flex shrink-0 w-full border-b dark:bg-neutral-800 border-neutral-300 dark:border-neutral-600 sticky top-0 z-50"
    @change="changeTab"
  >
    <template #tabs>
      <SiTabHeader
        v-for="func in funcList"
        :key="func.id"
        classes="border-x border-t border-x-neutral-300 border-t-neutral-300 dark:border-x-neutral-600 dark:border-t-neutral-600 h-11 px-2 text-sm inline-flex items-center z-50"
        selected-classes="border-b-white dark:border-b-neutral-800 border-b-2"
      >
        {{ func.name }}
        <button
          class="inline-block rounded-sm w-5 ml-1"
          @click="closeFunc(func)"
        >
          <VueFeather type="x" />
        </button>
      </SiTabHeader>
    </template>
    <template #panels>
      <TabPanel v-for="func in funcList" :key="func.id" class="h-full">
        <FuncEditor :func-id="func.id" />
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { toRef, computed } from "vue";
import { refFrom, fromRef } from "vuse-rx/src";
import { FuncService } from "@/service/func";
import { GetFuncResponse } from "@/service/func/get_func";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";
import FuncEditor from "@/organisms/FuncEditor/FuncEditor.vue";
import VueFeather from "vue-feather";
import { funcStream$, funcEdit$ } from "@/observable/func_editor";
import { ListedFuncView } from "@/service/func/list_funcs";
import { map, switchMap } from "rxjs/operators";

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
const funcList = refFrom<ListedFuncView[]>(
  funcStream$.pipe(
    map((editingFuncs) =>
      editingFuncs.map(({ origFunc }) => ({
        id: origFunc.id,
        handler: origFunc.handler,
        name: origFunc.name,
        kind: origFunc.kind,
      })),
    ),
  ),
  [],
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
  }
};

const closeFunc = (func: ListedFuncView) => {
  // Handle unsaved functions here with modal...  or dispatch a save on close?
  funcEdit$.next({
    type: "remove",
    func: func,
  });
};

const insertFunc = (func: GetFuncResponse) => {
  funcEdit$.next({
    type: "insert",
    func: func,
  });
};

// Inserts a function into the funcstream when we fetch it from the backend. Insert is idempotent
selectedFuncId$
  .pipe(
    switchMap((selectedFuncId) => FuncService.getFunc({ id: selectedFuncId })),
  )
  .subscribe(insertFunc);
</script>
