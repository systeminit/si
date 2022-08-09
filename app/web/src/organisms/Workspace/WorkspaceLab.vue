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
      class="grow h-screen place-items-center dark:bg-neutral-800 dark:text-white text-lg font-semibold overflow-hidden"
    >
      <FuncEditorTabs
        v-if="selectedFunc.id > 0"
        :selected-func-id="selectedFunc.id"
        @selected-func="selectFunc"
      />
      <div v-else>Pick a function to edit</div>
    </div>
    <SiSidebar :hidden="false" side="right">
      <!-- if hiding is added later, condition is selectedFuncId < 1 -->
      <FunctionDetails :func="selectedFunc" />
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

const selectedFunc = ref<ListedFuncView>(nullListFunc);
const selectFunc = (func: ListedFuncView) => {
  selectedFunc.value = func;
};

const funcList = refFrom<ListFuncsResponse>(FuncService.listFuncs(), {
  qualifications: [],
});

const createFunction = () => {
  FuncService.createFunc().subscribe(console.log);
};
</script>
