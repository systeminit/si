<template>
  <TabGroup
    ref="tabGroupRef"
    closeable
    first-tab-margin-left="none"
    @close-tab="closeFunc"
    @update:selected-tab="onTabChange"
  >
    <template #noTabs>
      <div class="p-2 text-center text-neutral-400 dark:text-neutral-300">
        <RequestStatusMessage
          v-if="loadFuncsReqStatus.isPending"
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
      v-for="openFuncId in openFuncIds"
      :key="openFuncId"
      :slug="openFuncId"
    >
      <template #label>{{
        funcStore.funcsById[openFuncId]?.name ?? openFuncId
      }}</template>
      <FuncEditor :func-id="openFuncId" />
    </TabGroupItem>
  </TabGroup>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { watch, ref, nextTick } from "vue";
import {
  RequestStatusMessage,
  TabGroup,
  TabGroupItem,
} from "@si/vue-lib/design-system";
import FuncEditor from "@/components/FuncEditor/FuncEditor.vue";
import { useFuncStore, FuncId } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";

const props = defineProps({
  funcId: { type: String },
});

const tabGroupRef = ref<InstanceType<typeof TabGroup>>();

const routeToFunc = useRouteToFunc();
const funcStore = useFuncStore();
const loadFuncsReqStatus = funcStore.getRequestStatus("FETCH_FUNC_LIST");

const openFuncIds = ref<FuncId[]>([]);

const closeFunc = (funcId: string) => {
  openFuncIds.value = _.without(openFuncIds.value, funcId);
};

const onTabChange = (tabSlug: string | undefined) => {
  // tabSlugs are just func ids here
  routeToFunc(tabSlug);
};

// this is responsible for watching the selected func (which is based on the URL)
// and adding it to the open list when it changes - and then selecting that tab
watch(
  () => funcStore.selectedFuncId,
  (newFuncId) => {
    if (typeof newFuncId === "undefined") {
      return;
    }

    if (!openFuncIds.value.includes(newFuncId)) {
      openFuncIds.value.push(newFuncId);
    }

    // TODO: maybe we can make TabGroup deal with this instead?
    nextTick(() => {
      tabGroupRef.value?.selectTab(newFuncId);
    });
  },
  { immediate: true },
);
</script>
