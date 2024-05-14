<template>
  <div
    v-if="funcList.length"
    class="flex flex-col overflow-hidden h-full relative"
  >
    <SidebarSubpanelTitle icon="func">
      <template #label>
        <div class="flex flex-row gap-xs">
          <div>Functions</div>
          <PillCounter :count="totalFuncs" />
        </div>
      </template>
      <NewFuncDropdown
        label="Function"
        :fnTypes="CREATE_OPTIONS"
        @selected-func-kind="createNewFunc"
      />
    </SidebarSubpanelTitle>

    <ErrorMessage
      v-if="createFuncReqStatus.isError"
      :requestStatus="createFuncReqStatus"
    />
    <SiSearch autoSearch placeholder="search functions" @search="onSearch" />
    <!-- <div
          class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-xs border-b dark:border-neutral-600"
        >
          Select a function to view or edit it.
        </div> -->

    <FuncList
      :funcsByKind="funcsByKind"
      context="workspace-lab-functions"
      firstOpen
    />
  </div>
  <RequestStatusMessage
    v-else
    :requestStatus="loadFuncsReqStatus"
    loadingMessage="Loading functions..."
  />
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import * as _ from "lodash-es";
import {
  ErrorMessage,
  PillCounter,
  RequestStatusMessage,
} from "@si/vue-lib/design-system";
import SiSearch from "@/components/SiSearch.vue";
import {
  CUSTOMIZABLE_FUNC_TYPES,
  CustomizableFuncKind,
  customizableFuncKindToFuncKind,
} from "@/api/sdf/dal/func";
import NewFuncDropdown from "@/components/NewFuncDropdown.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import FuncList from "./FuncList.vue";
import SidebarSubpanelTitle from "../SidebarSubpanelTitle.vue";

const routeToFunc = useRouteToFunc();
const funcStore = useFuncStore();
const loadFuncsReqStatus = funcStore.getRequestStatus("FETCH_FUNC_LIST");
const { funcList } = storeToRefs(funcStore);

const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const CREATE_OPTIONS = _.mapValues(CUSTOMIZABLE_FUNC_TYPES, "singularLabel");

const filteredList = computed(() => {
  if (!searchString.value) return funcList.value;
  return _.filter(funcList.value, (f) =>
    f.name.toLocaleLowerCase().includes(searchString.value),
  );
});

const funcsByKind = computed(() =>
  _.groupBy(filteredList.value, (f) => f.kind),
);

const createFuncReqStatus = funcStore.getRequestStatus("CREATE_FUNC");

async function createNewFunc(kind: CustomizableFuncKind) {
  const createReq = await funcStore.CREATE_FUNC({
    kind: customizableFuncKindToFuncKind(kind),
  });
  if (createReq.result.success) {
    routeToFunc(createReq.result.data.id);
  }
}

const totalFuncs = computed(() => {
  let count = 0;
  for (const key in funcsByKind.value) {
    count += funcsByKind.value[key]?.length || 0;
  }
  return count;
});
</script>
