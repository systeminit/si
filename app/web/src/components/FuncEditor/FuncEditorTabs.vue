<template>
  <TabGroup
    ref="tabGroupRef"
    closeable
    firstTabMarginLeft="none"
    rememberSelectedTabKey="func-editor"
    :startSelectedTabSlug="funcStore.urlSelectedFuncId"
    @close-tab="closeFunc"
    @update:selected-tab="onTabChange"
  >
    <template #noTabs>
      <WorkspaceCustomizeEmptyState
        :requestStatus="loadFuncsReqStatus"
        loadingMessage="Loading functions..."
      />
    </template>

    <TabGroupItem
      v-for="openFuncId in funcStore.openFuncIds"
      :key="openFuncId"
      :slug="openFuncId"
    >
      <template #label>
        {{ funcStore.funcsById[openFuncId]?.name || "error" }}
      </template>

      <FuncEditor
        v-if="funcStore.funcsById[openFuncId]"
        :key="openFuncId"
        :funcId="openFuncId"
        @close="closeTab(openFuncId)"
      />
      <template v-else>
        <div
          class="p-sm border border-t-0 border-neutral-300 dark:border-neutral-600"
        >
          <ErrorMessage
            >Function "{{ openFuncId }}" does not exist</ErrorMessage
          >
        </div>
      </template>
    </TabGroupItem>
  </TabGroup>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { watch, ref, nextTick } from "vue";
import {
  ErrorMessage,
  TabGroup,
  TabGroupItem,
} from "@si/vue-lib/design-system";
import FuncEditor from "@/components/FuncEditor/FuncEditor.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import WorkspaceCustomizeEmptyState from "../WorkspaceCustomizeEmptyState.vue";

const tabGroupRef = ref<InstanceType<typeof TabGroup>>();

const routeToFunc = useRouteToFunc();
const funcStore = useFuncStore();
const loadFuncsReqStatus = funcStore.getRequestStatus("FETCH_FUNC_LIST");

const closeFunc = (funcId: string) => {
  funcStore.setOpenFuncId(funcId, false);
};

// when the tab component emits a new selected tab event, we update the URL to match if it doesn't already
const onTabChange = (tabSlug: string | undefined) => {
  // tabSlugs are just func ids here
  if (funcStore.urlSelectedFuncId !== tabSlug) {
    routeToFunc(tabSlug);
  }
};

const closeTab = (slug: string) => {
  if (tabGroupRef.value) {
    tabGroupRef.value.closeTabBySlug(slug);
  }
};

// when the url changes, the tab gets automatically added to the open list in the store
// but we need to tell the tab component to select it once it is rendered
watch(
  () => funcStore.urlSelectedFuncId,
  () => {
    nextTick(() => {
      tabGroupRef.value?.selectTab(funcStore.urlSelectedFuncId);
    });
  },
);
</script>
