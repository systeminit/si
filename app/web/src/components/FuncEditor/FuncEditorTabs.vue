<template>
  <TabGroup
    ref="tabGroupRef"
    no-start-margin
    no-after-margin
    closeable
    @close-tab="closeFunc"
    @update:selected-tab="onTabChange"
  >
    <template #noTabs>
      <div class="p-2 text-center text-neutral-400 dark:text-neutral-300">
        <RequestStatusMessage
          v-if="loadFuncsReqStatus.isPending || funcReqStatus.isPending"
          :request-status="loadFuncsReqStatus"
          show-loader-without-message
        />
        <template v-else-if="loadFuncsReqStatus.isSuccess">
          <template v-if="funcId"
            >Function "{{ funcId }}" does not exist!</template
          >
          <template v-else>Select a function to edit it.</template>
        </template>
      </div>
    </template>
    <TabGroupItem
      v-for="func in openFuncsList"
      :key="func.id"
      :label="func.name"
      :slug="func.id"
    >
      <FuncEditor :func-id="func.id" />
    </TabGroupItem>
  </TabGroup>
  <!-- <SiTabGroup
    v-else
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
            class="inline-block rounded-full text-neutral-400 ml-1"
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
    <template #dropdownContent>
      <DropdownMenuItem
        v-for="func in openFuncsList"
        :key="func.id"
        :checked="func.id === selectedFuncId"
        @select="routeToFunc(func.id)"
      >
        {{ func.name }}
      </DropdownMenuItem>
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
  </SiTabGroup> -->
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { storeToRefs } from "pinia";
import clsx from "clsx";
import { watch, ref } from "vue";
import SiTabGroup from "@/components/SiTabGroup.vue";
import SiTabHeader from "@/components/SiTabHeader.vue";
import FuncEditor from "@/components/FuncEditor/FuncEditor.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import { themeClasses } from "@/ui-lib/theme_tools";
import DropdownMenuItem from "@/ui-lib/menus/DropdownMenuItem.vue";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import TabGroup from "@/ui-lib/tabs/TabGroup.vue";
import TabGroupItem from "@/ui-lib/tabs/TabGroupItem.vue";

const props = defineProps({
  funcId: { type: String },
});

const tabGroupRef = ref<InstanceType<typeof TabGroup>>();

const routeToFunc = useRouteToFunc();
const funcStore = useFuncStore();
const {
  openFuncsList,
  selectedFuncId,
  selectedFuncIndex,
  getFuncByIndex,
  getIndexForFunc,
} = storeToRefs(funcStore);
const loadFuncsReqStatus = funcStore.getRequestStatus("FETCH_FUNC_LIST");
const funcReqStatus = funcStore.getRequestStatus("FETCH_FUNC");

const closeFunc = (funcId: string) => {
  const funcIndex = getIndexForFunc.value(funcId);
  if (funcId === selectedFuncId.value) {
    const newIndex = funcIndex - 1;
    const index = newIndex < 0 ? 0 : newIndex;
    routeToFuncByIndex(index);
  }
  funcStore.CLOSE_FUNC(funcId);
};

const routeToFuncByIndex = (index: number) => {
  let func = getFuncByIndex.value(index);

  // TODO(Wendy) - this ugly fix is to prevent a bug where closing the final tab doesn't select another open tab
  let i = index;
  while (func === undefined && i > -1) {
    i--;
    if (i > -1) {
      func = getFuncByIndex.value(i);
    }
  }

  routeToFunc(func?.id);
};

function nilId(): string {
  return "00000000000000000000000000";
}

const onTabChange = (slug: string | undefined) => {
  if (slug) {
    routeToFunc(slug);
  }
};

const selectTabFromURL = () => {
  if (tabGroupRef.value) {
    console.log("SELECTING BY URL");
    tabGroupRef.value.selectTab(props.funcId);
  }
};

watch([() => props.funcId], () => {
  selectTabFromURL();
  console.log("WATCHER GO NOW!");
});
</script>
