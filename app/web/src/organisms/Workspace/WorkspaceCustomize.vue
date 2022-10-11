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
    <FuncDetails v-if="!isLoadingFunc" :func-id="funcIdParam" />
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
import { ListedFuncView } from "@/service/func/list_funcs";
import { FuncBackendKind } from "@/api/sdf/dal/func";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import { DevService } from "@/service/dev";
import { useFuncStore, createFuncPromise } from "@/store/funcs.store";

const funcStore = useFuncStore();
const { isLoadingFunc, selectedFuncId } = storeToRefs(funcStore);

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
  const func =
    isDevMode && isBuiltin && !_.isNil(name)
      ? await DevService.createBuiltinFunc({ name, kind })
      : await createFuncPromise({ kind });

  selectFunc(func);
};
</script>
