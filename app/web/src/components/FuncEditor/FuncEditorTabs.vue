<template>
  <div>
    <RequestStatusMessage
      v-if="!loadFuncsReqStatus.isPending && !funcStore.funcList.length"
      :request-status="loadFuncsReqStatus"
      :loading-message="`Loading function ${funcId}`"
    />
    <template v-else-if="loadFuncsReqStatus.isSuccess">
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
          v-for="func in openFuncsList"
          :key="func.id"
          :label="func.name"
          :slug="func.id"
        >
          <FuncEditor :func-id="func.id" />
        </TabGroupItem>
      </TabGroup>
    </template>
  </div>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { watch, ref, computed, nextTick } from "vue";
import FuncEditor from "@/components/FuncEditor/FuncEditor.vue";
import { useFuncStore, FuncId } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import TabGroup from "@/ui-lib/tabs/TabGroup.vue";
import TabGroupItem from "@/ui-lib/tabs/TabGroupItem.vue";

const props = defineProps({
  funcId: { type: String },
});

const tabGroupRef = ref<InstanceType<typeof TabGroup>>();

const routeToFunc = useRouteToFunc();
const funcStore = useFuncStore();
const loadFuncsReqStatus = funcStore.getRequestStatus("FETCH_FUNC_LIST");

const openFuncIds = ref([] as FuncId[]);
const openFuncsList = computed(() => {
  return _.map(openFuncIds.value, (id) => funcStore.funcsById[id]);
});

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
  [() => funcStore.selectedFuncSummary],
  () => {
    const funcId = funcStore.selectedFuncSummary?.id;
    if (!funcId) return;
    if (!openFuncIds.value.includes(funcId)) {
      openFuncIds.value.push(funcId);
    }
    // have to wait for the new tab to be rendered before we can select it
    // TODO: maybe we can make TabGroup deal with this instead?
    nextTick(() => {
      tabGroupRef.value?.selectTab(funcId);
    });
  },
  { immediate: true },
);
</script>
