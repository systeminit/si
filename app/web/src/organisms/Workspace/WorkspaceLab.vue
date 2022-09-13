<template>
  <div class="flex flex-row w-full h-full bg-transparent overflow-hidden">
    <SiPanel
      remember-size-key="func-picker"
      side="left"
      class="h-full pb-12"
      size-classes="shrink-0 w-96"
      :min-resize="300"
    >
      <ChangeSetPanel class="border-b-2 dark:border-neutral-500 mb-2" />
      <FuncPicker
        :func-list="funcList"
        :selected-func-id="selectedFuncId"
        @selected-func="selectFunc"
        @create-func="createFunc"
      />
    </SiPanel>
    <div
      class="grow overflow-x-hidden overflow-y-hidden dark:bg-neutral-800 dark:text-white text-lg font-semi-bold px-2 pt-2 flex flex-col"
    >
      <FuncEditorTabs
        v-if="selectedFuncId > 0 && !isLoading"
        :selected-func-id="selectedFuncId"
        @selected-func="selectFunc"
      />
      <div
        v-else
        class="p-2 text-center text-neutral-400 dark:text-neutral-300"
      >
        Select a function to edit it.
      </div>
    </div>
    <SiPanel
      remember-size-key="func-details"
      :hidden="isLoading"
      side="right"
      class="h-full pb-12"
      size-classes="shrink-0 w-80"
      :min-resize="200"
    >
      <!-- if hiding is added later, condition is selectedFuncId < 1 -->
      <FuncDetails :func-id="selectedFuncId" />
    </SiPanel>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { untilUnmounted } from "vuse-rx/src";
import { bufferTime } from "rxjs/operators";
import { firstValueFrom } from "rxjs";
import _ from "lodash";
import SiPanel from "@/atoms/SiPanel.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import FuncPicker from "@/organisms/FuncEditor/FuncPicker.vue";
import FuncEditorTabs from "@/organisms/FuncEditor/FuncEditorTabs.vue";
import { FuncService } from "@/service/func";
import { SaveFuncRequest } from "@/service/func/save_func";
import FuncDetails from "@/organisms/FuncEditor/FuncDetails.vue";
import { ListedFuncView, ListFuncsResponse } from "@/service/func/list_funcs";
import { visibility$ } from "@/observable/visibility";
import { saveFuncToBackend$ } from "@/observable/func";
import { eventChangeSetWritten$ } from "@/observable/change_set";
import { FuncBackendKind } from "@/api/sdf/dal/func";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import { DevService } from "@/service/dev";
import { clearFuncs } from "../FuncEditor/func_state";

const isDevMode = import.meta.env.DEV;

const props = defineProps<{ funcId?: string }>();

const selectedFuncId = computed(() => {
  const funcId = parseInt(props.funcId ?? "");
  if (Number.isNaN(funcId)) {
    return -1;
  }
  return funcId;
});

const routeToFunc = useRouteToFunc();
const selectFunc = (func: ListedFuncView) => {
  routeToFunc(func.id);
};

const isLoading = ref(true);
const funcList = ref<ListFuncsResponse>({ funcs: [] });

FuncService.listFuncs().subscribe((funcs) => {
  funcList.value = funcs;
  isLoading.value = false;
});

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
      : await FuncService.createFunc({ kind });

  await firstValueFrom(eventChangeSetWritten$);
  selectFunc(func);
};

visibility$.subscribe(() => {
  clearFuncs();
  routeToFunc(); // route to no func
});

saveFuncToBackend$
  .pipe(untilUnmounted, bufferTime(2000))
  .subscribe((saveRequests) =>
    Object.values(
      saveRequests.reduce(
        (acc, saveReq) => ({ ...acc, [saveReq.id]: saveReq }),
        {} as { [key: number]: SaveFuncRequest },
      ),
    ).forEach((saveReq) => {
      if (isDevMode && saveReq.isBuiltin) DevService.saveBuiltinFunc(saveReq);
      else FuncService.saveFunc(saveReq);
    }),
  );
</script>
