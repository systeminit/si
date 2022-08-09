<template>
  <div class="overflow-auto w-full h-full">
    <SiTabGroup :selected-index="selectedTab" @change="changeTab">
      <template #tabs>
        <SiTabHeader v-for="(funcId, index) in funcList" :key="funcId">{{
          editingFuncs[index].modifiedFunc.name
        }}</SiTabHeader>
      </template>
      <template #panels>
        <TabPanel v-for="(funcId, index) in funcList" :key="funcId">
          <FuncEditor
            :funcId="funcId"
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
import { ref, toRef } from "vue";
import { refFrom, fromRef } from "vuse-rx/src";
import { combineLatest, iif, of } from "rxjs";
import { switchMap, tap } from "rxjs/operators";
import { FuncService } from "@/service/func";
import { GetFuncResponse } from "@/service/func/get_func";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";
import FuncEditor from "@/organisms/FuncEditor.vue";
import {
  EditingFunc,
  editingFuncs$,
  selectedTab$,
} from "@/observable/func_editor";
import isEqual from "lodash/isEqual";

const props = defineProps<{
  selectedFuncId: number;
}>();

const emits = defineEmits<{
  (e: "selectedFunc", v: number): void;
}>();

const selectFunc = (funcId: number) => {
  emits("selectedFunc", funcId);
};

const selectedFuncId = toRef(props, "selectedFuncId", 0);
const selectedFuncId$ = fromRef(selectedFuncId, { immediate: true });
const selectedTab = refFrom<number>(selectedTab$);
const loadedFuncs = ref<{ [key: number]: GetFuncResponse }>({});

const changeTab = (index: number) => {
  selectFunc(funcList.value[index] ?? 0);
  selectedTab$.next(index);
};

const funcList = ref<number[]>([]);

const findTabIndexForFunc = (funcList: EditingFunc[], func: { id: number }) =>
  funcList.findIndex((fn) => fn.id == func.id);

const editingFuncs = refFrom<EditingFunc[]>(
  editingFuncs$.pipe(
    tap((editingFuncs) => {
      const newFuncList = editingFuncs.map((editingFunc) => editingFunc.id);
      if (!isEqual(newFuncList, funcList.value)) {
        funcList.value = [...newFuncList];
      }
    }),
  ),
  [],
);

const updateCodeForFunc = (func: EditingFunc, newCode: string) =>
  updateFunc({
    ...func,
    modifiedFunc: {
      ...func.modifiedFunc,
      code: newCode,
    },
  });

const updateFunc = (func: EditingFunc) => {
  const funcList = [...editingFuncs.value];
  console.log('updateFunc', func, funcList);
  const existingFuncIdx = findTabIndexForFunc(funcList, func);
  if (existingFuncIdx == -1) {
    console.error("Could not find func", func);
    return;
  }

  funcList[existingFuncIdx] = { ...func };
  editingFuncs$.next([...funcList]);
};

const insertFunc = (func: GetFuncResponse) => {
  const funcList = [...editingFuncs.value];
  const existingFuncIdx = findTabIndexForFunc(funcList, func);
  let selectedTab = existingFuncIdx;
  if (existingFuncIdx == -1) {
    funcList.push({
      modifiedFunc: { ...func },
      origFunc: { ...func },
      id: func.id,
    });
    selectedTab = funcList.length - 1;
  } else {
    funcList[existingFuncIdx] = {
      ...funcList[existingFuncIdx],
      origFunc: { ...func },
    };
  }

  editingFuncs$.next([...funcList]);
  selectedTab$.next(selectedTab);
};

refFrom<GetFuncResponse | undefined>(
  combineLatest([selectedFuncId$]).pipe(
    switchMap(([selectedFuncId]) =>
      iif(
        () => selectedFuncId > 0,
        FuncService.getFunc({ id: selectedFuncId }),
        of(undefined),
      ),
    ),
    tap((func) => {
      func && insertFunc(func);
    }),
  ),
  undefined,
);
</script>
