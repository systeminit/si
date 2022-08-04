<template>
  <div class="flex flex-row w-full bg-transparent">
    <SiSidebar side="left">
      <ChangeSetPanel class="border-b-2 dark:border-neutral-500 mb-2" />
      <FuncPicker :func-list="funcList" />
    </SiSidebar>
    <div
      class="grow h-screen w-full place-items-center dark:bg-neutral-800 dark:text-white text-lg font-semibold"
    >
      Workspace Lab
    </div>
  </div>
</template>

<script lang="ts" setup>
import SiSidebar from "@/atoms/SiSidebar.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import FuncPicker from "@/organisms/FuncPicker.vue";
import { FuncService } from "@/service/func";
import { ListFuncsResponse } from "@/service/func/list_funcs";
import { refFrom } from "vuse-rx/src";

const funcList = refFrom<ListFuncsResponse>(FuncService.listFuncs(), {
  qualifications: [],
});
</script>
