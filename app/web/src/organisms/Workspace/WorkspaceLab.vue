<template>
  <div class="flex flex-row w-full h-full bg-transparent overflow-hidden">
    <SiSidebar side="left" class="h-full pb-12">
      <ChangeSetPanel class="border-b-2 dark:border-neutral-500 mb-2" />
      <FuncPicker
        :func-list="funcList"
        :selected-func-id="selectedFunc.id"
        @selected-func="selectFunc"
        @create-func="createFunc"
      />
    </SiSidebar>
    <div
      class="grow overflow-x-hidden overflow-y-auto lace-items-center dark:bg-neutral-800 dark:text-white text-lg font-semibold p-0"
    >
      <FuncEditorTabs
        v-if="selectedFunc.id > 0"
        :selected-func-id="selectedFunc.id"
        @selected-func="selectFunc"
      />
      <div v-else class="p-2 text-center text-neutral-400">
        Select a function to edit it.
      </div>
    </div>
    <SiSidebar :hidden="false" side="right" class="h-full pb-12 min-w-[230px]">
      <!-- if hiding is added later, condition is selectedFuncId < 1 -->
      <FuncDetails :func-id="selectedFunc.id" />
    </SiSidebar>
  </div>
</template>

<script lang="ts" setup>
import SiSidebar from "@/atoms/SiSidebar.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import FuncPicker from "@/organisms/FuncEditor/FuncPicker.vue";
import FuncEditorTabs from "@/organisms/FuncEditor/FuncEditorTabs.vue";
import { FuncService } from "@/service/func";
import FuncDetails from "@/organisms/FuncEditor/FuncDetails.vue";
import {
  ListedFuncView,
  ListFuncsResponse,
  nullListFunc,
} from "@/service/func/list_funcs";
import { ref } from "vue";
import { refFrom } from "vuse-rx/src";
import { visibility$ } from "@/observable/visibility";
import { clearFuncs } from "./../FuncEditor/func_state";
import { debounceTime, tap } from "rxjs/operators";
import { saveFuncToBackend$ } from "@/observable/func";

const selectedFunc = ref<ListedFuncView>(nullListFunc);
const selectFunc = (func: ListedFuncView) => {
  selectedFunc.value = func;
};

const funcList = refFrom<ListFuncsResponse>(FuncService.listFuncs(), {
  qualifications: [],
});

const createFunc = async () => {
  const func = await FuncService.createFunc();
  const newFunc = {
    id: func.id,
    kind: func.kind,
    name: func.name,
    handler: func.handler,
  };

  funcList.value.qualifications.push(newFunc);

  selectFunc(newFunc);
};

visibility$.subscribe(() => {
  clearFuncs();
  selectFunc(nullListFunc);
});

saveFuncToBackend$
  .pipe(
    debounceTime(75),
    tap((saveFuncRequest) => FuncService.saveFunc(saveFuncRequest)),
  )
  .subscribe();
</script>
