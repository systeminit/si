<template>
  <SiPanel remember-size-key="func-picker" side="left" :min-size="300">
    <div class="flex flex-col h-full">
      <ChangeSetPanel
        class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
      />
      <div class="relative flex-grow">
        <FuncPicker @create-func="createFunc" />
      </div>
    </div>
  </SiPanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 text-lg font-semi-bold flex flex-col relative"
  >
    <div class="inset-2 bottom-0 absolute">
      <FuncEditorTabs v-if="selectedFuncId > 0" />
      <div
        v-else
        class="p-2 text-center text-neutral-400 dark:text-neutral-300"
      >
        Select a function to edit it.
      </div>
    </div>
  </div>
  <SiPanel remember-size-key="func-details" side="right" :min-size="200">
    <FuncDetails v-if="!funcReqStatus.isPending" />
  </SiPanel>
</template>

<script lang="ts" setup>
import { toRef, watch } from "vue";
import _ from "lodash";
import { storeToRefs } from "pinia";
import SiPanel from "@/atoms/SiPanel.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import FuncPicker from "@/organisms/FuncEditor/FuncPicker.vue";
import FuncEditorTabs from "@/organisms/FuncEditor/FuncEditorTabs.vue";
import FuncDetails from "@/organisms/FuncEditor/FuncDetails.vue";
import { ListedFuncView } from "@/store/func/requests/list_funcs";
import { FuncBackendKind } from "@/api/sdf/dal/func";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import { useFuncStore } from "@/store/func/funcs.store";

const funcStore = useFuncStore();
const funcReqStatus = funcStore.getRequestStatus("FETCH_FUNC");
const { selectedFuncId } = storeToRefs(funcStore);

const isDevMode = import.meta.env.DEV;

const props = defineProps<{
  funcId?: number;
}>();

const funcIdParam = toRef(props, "funcId", -1);

watch(
  () => funcIdParam.value,
  (funcIdParam) => {
    let funcId = funcIdParam ?? -1;
    if (Number.isNaN(funcIdParam)) {
      funcId = -1;
    }
    funcStore.SELECT_FUNC(funcId);
  },
  { immediate: true },
);

const routeToFunc = useRouteToFunc();
const selectFunc = (func: ListedFuncView) => {
  routeToFunc(func.id);
};

const createFunc = async ({
  isBuiltin,
  kind,
  name,
}: {
  kind: FuncBackendKind;
  isBuiltin: boolean;
  name?: string;
}) => {
  let func;

  if (isDevMode && isBuiltin && !_.isNil(name)) {
    const res = await funcStore.CREATE_BUIILTIN_FUNC({ name, kind });
    func = res.result.success && res.result.data;
  } else {
    const res = await funcStore.CREATE_FUNC({ kind });
    func = res.result.success && res.result.data;
  }

  if (func) {
    selectFunc(func);
  }
};
</script>
