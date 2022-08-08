<template>
  <div class="flex flex-row w-full bg-transparent">
    <SiSidebar side="left">
      <ChangeSetPanel class="border-b-2 dark:border-neutral-500 mb-2" />
      <FuncPicker
        :func-list="funcList"
        :selected-func-id="selectedFuncId"
        @selected-func="
          (id) => {
            selectedFuncId = id;
          }
        "
      />
    </SiSidebar>
    <div
      class="grow h-screen w-full place-items-center dark:bg-neutral-800 dark:text-white text-lg font-semibold"
    >
      <FuncEditor
        v-if="selectedFuncId > 0"
        :selected-func-id="selectedFuncId"
      />
      <div v-else>Pick a function to edit</div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import SiSidebar from "@/atoms/SiSidebar.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import FuncPicker from "@/organisms/FuncPicker.vue";
import FuncEditor from "@/organisms/FuncEditor.vue";
import { FuncService } from "@/service/func";
import { ListFuncsResponse } from "@/service/func/list_funcs";
import { ref, watch } from "vue";
import { refFrom, fromRef } from "vuse-rx/src";

const selectedFuncId = ref<number>(0);

const funcList = refFrom<ListFuncsResponse>(FuncService.listFuncs(), {
  qualifications: [],
});
</script>
