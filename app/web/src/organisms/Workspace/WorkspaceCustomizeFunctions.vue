<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="func-picker" side="left" :min-size="300">
    <div class="flex flex-col h-full">
      <ChangeSetPanel />
      <div class="relative flex-grow">
        <FuncListPanel />
      </div>
    </div>
  </SiPanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 text-lg font-semi-bold flex flex-col relative"
  >
    <div class="inset-2 bottom-0 absolute">
      <FuncEditorTabs :func-id="funcId" />
    </div>
  </div>
  <SiPanel remember-size-key="func-details" side="right" :min-size="200">
    <FuncDetails :func-id="funcId" />
  </SiPanel>
</template>

<script lang="ts" setup>
import { toRef, watch } from "vue";
import _ from "lodash";
import { storeToRefs } from "pinia";
import SiPanel from "@/atoms/SiPanel.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import FuncListPanel from "@/organisms/FuncEditor/FuncListPanel.vue";
import FuncEditorTabs from "@/organisms/FuncEditor/FuncEditorTabs.vue";
import FuncDetails from "@/organisms/FuncEditor/FuncDetails.vue";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import { useFuncStore } from "@/store/func/funcs.store";

const funcStore = useFuncStore();
const { selectedFuncId } = storeToRefs(funcStore);

const props = defineProps<{
  funcId?: string;
  workspaceId: string;
  changeSetId: string;
}>();

function nilId(): string {
  return "00000000000000000000000000";
}
const funcIdParam = toRef(props, "funcId", nilId());

const routeToFunc = useRouteToFunc();

watch(
  () => funcIdParam.value,
  (funcIdParam) => {
    let funcId = funcIdParam ?? nilId();
    if (funcId === "") {
      selectedFuncId.value = nilId();
      return;
    }

    if (funcId === nilId()) {
      if (selectedFuncId.value !== nilId()) {
        routeToFunc(selectedFuncId.value);
        return;
      } else {
        funcId = nilId();
      }
    }
    funcStore.SELECT_FUNC(funcId);
  },
  { immediate: true },
);
</script>
