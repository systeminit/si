<template>
  <SiTabGroup
    :selected-index="selectedFuncIndex"
    selected-tab-to-front
    :tab-width-maximum="0.3"
    no-start-margin
    no-after-margin
    :top-margin="0"
    @change="routeToFuncByIndex"
  >
    <template #tabs>
      <SiTabHeader v-for="func in openFuncsList" :key="func.id">
        {{ func.name }}
        <template #icon>
          <button
            class="inline-block rounded-sm rounded-3xl text-neutral-400 ml-1"
            :class="
              clsx(
                themeClasses(
                  'hover:text-white hover:bg-neutral-400',
                  'hover:text-neutral-800 hover:bg-neutral-400',
                ),
              )
            "
            @click="closeFunc(func.id)"
          >
            <Icon name="x" size="xs" />
          </button>
        </template>
      </SiTabHeader>
    </template>
    <template #dropdownitems>
      <SiDropdownItem
        v-for="func in openFuncsList"
        :key="func.id"
        :checked="func.id === selectedFuncId"
        @select="routeToFunc(func.id)"
      >
        {{ func.name }}
      </SiDropdownItem>
    </template>
    <template #panels>
      <TabPanel
        v-for="func in openFuncsList"
        :key="func.id"
        class="h-full overflow-auto"
      >
        <FuncEditor :func-id="func.id" />
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { storeToRefs } from "pinia";
import clsx from "clsx";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import FuncEditor from "@/organisms/FuncEditor/FuncEditor.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import { themeClasses } from "@/ui-lib/theme_tools";

const routeToFunc = useRouteToFunc();
const funcStore = useFuncStore();
const {
  openFuncsList,
  selectedFuncId,
  selectedFuncIndex,
  getFuncByIndex,
  getIndexForFunc,
} = storeToRefs(funcStore);

const closeFunc = (funcId: number) => {
  const funcIndex = getIndexForFunc.value(funcId);
  if (funcId === selectedFuncId.value) {
    const newIndex = funcIndex - 1;
    routeToFuncByIndex(newIndex < 0 ? 0 : newIndex);
  }
  funcStore.CLOSE_FUNC(funcId);
};

const routeToFuncByIndex = (index: number) => {
  const func = getFuncByIndex.value(index);
  routeToFunc(func.id);
};
</script>
