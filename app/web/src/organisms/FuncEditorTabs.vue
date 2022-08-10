<template>
  <div class="overflow-hidden w-full h-full">
    <SiTabGroup
      :selected-index="selectedTab"
      tab-list-classes="h-11 flex shrink-0 w-full"
      @change="changeTab"
    >
      <template #tabs>
        <SiTabHeader
          v-for="(func, index) in funcList"
          :key="func.id"
          class="border-x border-t border-neutral-300 dark:border-neutral-600 h-11 px-2"
        >
          {{ editingFuncs[index].origFunc.name }}
          <button
            class="inline-block rounded-sm w-5 ml-1"
            @click="closeFunc(func.id)"
          >
            <VueFeather type="x" />
          </button>
        </SiTabHeader>
      </template>
      <template #panels>
        <TabPanel
          v-for="(func, index) in funcList"
          :key="func.id"
          class="w-full"
        >
          <FuncEditor
            :func-id="func.id"
            @updated-code="
              (code) => updateCodeForFunc(editingFuncs[index], code)
            "
          />
        </TabPanel>
      </template>
    </SiTabGroup>
  </div>
</template>

<script lang="ts" setup>
import { toRef, computed } from "vue";
import { refFrom, fromRef } from "vuse-rx/src";
import { FuncService } from "@/service/func";
import { GetFuncResponse } from "@/service/func/get_func";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";
import FuncEditor from "@/organisms/FuncEditor.vue";
import VueFeather from "vue-feather";
import { EditingFunc, editingFuncs$ } from "@/observable/func_editor";
import { ListedFuncView } from "@/service/func/list_funcs";

const props = defineProps<{
  selectedFuncId: number;
}>();

const emits = defineEmits<{
  (e: "selectedFunc", v: ListedFuncView): void;
  (e: "updatedCode", v: { func: EditingFunc; code: string }): void;
}>();

const updateCodeForFunc = (func: EditingFunc, code: string) =>
  emits("updatedCode", { func, code });

const selectFunc = (func: ListedFuncView) => {
  emits("selectedFunc", func);
};

const selectedFuncId = toRef(props, "selectedFuncId", 0);
const selectedFuncId$ = fromRef(selectedFuncId, { immediate: true });

const findTabIndexForFunc = (funcList: EditingFunc[], func: { id: number }) =>
  funcList.findIndex((fn) => fn.id == func.id);

// We need the editingFuncs ref to manage updates to the observable,
// but we also want to map it into a list of functions for managing the
// list of tabs, hence the tap.
const editingFuncs = refFrom<EditingFunc[]>(editingFuncs$, []);
const funcList = computed(() =>
  editingFuncs.value.map(({ origFunc }) => ({
    id: origFunc.id,
    handler: origFunc.handler,
    name: origFunc.name,
    kind: origFunc.kind,
  })),
);

const selectedTab = computed(() =>
  findTabIndexForFunc(editingFuncs.value, { id: selectedFuncId.value }),
);

const changeTab = (index: number) => {
  if (index > funcList.value.length - 1) {
    index--;
  }
  selectFunc(funcList.value[index] ?? 0);
};

const closeFunc = (funcId: number) => {
  const funcList = [...editingFuncs.value].filter((f) => f.id !== funcId);
  // Handle unsaved functions here with modal...  or dispatch a save on close?
  editingFuncs$.next([...funcList]);
};

const insertFunc = (func: GetFuncResponse) => {
  const funcList = [...editingFuncs.value];
  const existingFuncIdx = findTabIndexForFunc(funcList, func);
  if (existingFuncIdx == -1) {
    funcList.push({
      modifiedFunc: { ...func },
      origFunc: { ...func },
      id: func.id,
    });
  } else {
    funcList[existingFuncIdx] = {
      ...funcList[existingFuncIdx],
      origFunc: { ...func },
    };
  }

  editingFuncs$.next([...funcList]);
};

selectedFuncId$.subscribe((selectedFuncId) =>
  FuncService.getFunc({ id: selectedFuncId }).subscribe((func) =>
    insertFunc(func),
  ),
);
</script>
