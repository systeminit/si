<template>
  <SiTabGroup
    :key="tabGroupRerenderKey"
    :selected-index="selectedTab"
    selected-tab-to-front
    :tab-width-maximum="0.3"
    :start-margin="0"
    :after-margin="0"
    :top-margin="0"
    @change="changeTab"
  >
    <template #tabs>
      <SiTabHeader v-for="func in funcList" :key="func.id">
        {{ func.name }}
        <template #icon>
          <button
            class="inline-block rounded-sm w-5 ml-1"
            @click="closeFunc(func)"
          >
            <Icon name="x" />
          </button>
        </template>
      </SiTabHeader>
    </template>
    <template #dropdownitems>
      <SiDropdownItem
        v-for="func in funcList"
        :key="func.id"
        :checked="findTabIndexForFunc(funcList, func) === selectedTab"
        @select="changeTab(findTabIndexForFunc(funcList, func))"
      >
        {{ func.name }}
      </SiDropdownItem>
    </template>
    <template #panels>
      <TabPanel
        v-for="func in funcList"
        :key="func.id"
        class="h-full overflow-auto"
      >
        <FuncEditor :func-id="func.id" />
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { toRef, computed, ref } from "vue";
import { fromRef } from "vuse-rx/src";
import { TabPanel } from "@headlessui/vue";
import { switchMap, take } from "rxjs/operators";
import { of } from "rxjs";
import { FuncService } from "@/service/func";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import FuncEditor from "@/organisms/FuncEditor/FuncEditor.vue";
import { ListedFuncView, nullListFunc } from "@/service/func/list_funcs";
import Icon from "@/ui-lib/Icon.vue";
import { funcState, funcById, removeFunc, insertFunc } from "./func_state";

const props = defineProps<{
  selectedFuncId: number;
}>();

const emits = defineEmits<{
  (e: "selectedFunc", v: ListedFuncView): void;
}>();

const selectFunc = (func: ListedFuncView) => {
  emits("selectedFunc", func);
};

const selectedFuncId = toRef(props, "selectedFuncId", 0);
const selectedFuncId$ = fromRef(selectedFuncId, { immediate: true });

const findTabIndexForFunc = (
  funcList: { id: number }[],
  func: { id: number },
) => funcList.findIndex((fn) => fn.id === func.id);

const funcList = computed(() =>
  funcState.funcs.map(({ origFunc, modifiedFunc }) => ({
    id: origFunc.id,
    handler: modifiedFunc.handler,
    name: modifiedFunc.name,
    kind: modifiedFunc.kind,
    isBuiltin: origFunc.isBuiltin,
  })),
);

const selectedTab = computed(() =>
  findTabIndexForFunc(funcList.value, { id: selectedFuncId.value }),
);

const changeTab = (index: number) => {
  if (index < 0) {
    index = 0;
  }

  if (index > funcList.value.length - 1) {
    index--;
  }
  if (funcList.value.length) {
    selectFunc(funcList.value[index]);
  } else {
    selectFunc(nullListFunc);
  }
};

const tabGroupRerenderKey = ref(0);

const closeFunc = (func: ListedFuncView) => {
  const funcTab = findTabIndexForFunc(funcList.value, func);
  const currentTab = selectedTab.value;
  removeFunc(func);
  if (funcTab === currentTab) {
    changeTab(funcTab - 1);
  }
  tabGroupRerenderKey.value += 1;
};

selectedFuncId$
  .pipe(
    switchMap((selectedFuncId) => {
      const existingFunc = funcById(selectedFuncId);
      return existingFunc
        ? of({ ...existingFunc.origFunc })
        : FuncService.getFunc({ id: selectedFuncId }).pipe(take(1));
    }),
  )
  .subscribe(insertFunc);
</script>
