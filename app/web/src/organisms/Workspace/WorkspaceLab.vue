<template>
  <div class="flex flex-row w-full bg-transparent">
    <SiSidebar side="left">
      <ChangeSetPanel class="border-b-2 dark:border-neutral-500 mb-2" />
      <TertiaryNeutralButtonXSmall label="Create new function" />
      <FuncPicker
        :func-list="funcList"
        :selected-func-id="selectedFunc.id"
        @selected-func="selectFunc"
      />
    </SiSidebar>
    <div
      class="grow h-screen place-items-center dark:bg-neutral-800 dark:text-white text-lg font-semibold overflow-hidden p-2"
    >
      <FuncEditorTabs
        v-if="selectedFunc.id > 0"
        :selected-func-id="selectedFunc.id"
        @selected-func="selectFunc"
        @updated-code="({ func, code }) => updateCodeForFunc(func, code)"
      />
      <div v-else class="p-2 text-center text-neutral-400">
        Select a function to edit it.
      </div>
    </div>
    <SiSidebar :hidden="false" side="right">
      <!-- if hiding is added later, condition is selectedFuncId < 1 -->
      <FunctionDetails
        :func="selectedFunc"
        @updated-handler="
          (handler) => updateHandlerForFunc(selectedFunc, handler)
        "
        @updated-name="(name) => updateNameForFunc(selectedFunc, name)"
      />
    </SiSidebar>
  </div>
</template>

<script lang="ts" setup>
import SiSidebar from "@/atoms/SiSidebar.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import FuncPicker from "@/organisms/FuncPicker.vue";
import FuncEditorTabs from "@/organisms/FuncEditorTabs.vue";
import { FuncService } from "@/service/func";
import {
  ListedFuncView,
  ListFuncsResponse,
  nullListFunc,
} from "@/service/func/list_funcs";
import { ref } from "vue";
import { refFrom } from "vuse-rx/src";
import TertiaryNeutralButtonXSmall from "@/molecules/TertiaryNeutralButtonXSmall.vue";
import FunctionDetails from "@/organisms/FunctionDetails.vue";
import { EditingFunc, editingFuncs$ } from "@/observable/func_editor";
import { tap } from "rxjs/operators";

const selectedFunc = ref<ListedFuncView>(nullListFunc);
const selectFunc = (func: ListedFuncView) => {
  selectedFunc.value = func;
};

const funcList = refFrom<ListFuncsResponse>(FuncService.listFuncs(), {
  qualifications: [],
});

const editingFuncs = refFrom<EditingFunc[]>(
  editingFuncs$.pipe(tap(console.log)),
  [],
);

const updateHandlerForFunc = (func: EditingFunc, newHandler: string) =>
  updateFunc({
    ...func,
    modifiedFunc: {
      ...func.modifiedFunc,
      handler: newHandler,
    },
  });

const updateNameForFunc = (func: EditingFunc, newName: string) =>
  updateFunc({
    ...func,
    modifiedFunc: {
      ...func.modifiedFunc,
      name: newName,
    },
  });

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
  const existingFuncIdx = funcList.findIndex((f) => func.id === f.id);
  if (existingFuncIdx == -1) {
    console.error("Could not find func", func);
    return;
  }

  funcList[existingFuncIdx] = { ...func };
  editingFuncs$.next([...funcList]);
};
</script>
