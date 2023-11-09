<template>
  <div>
    <RequestStatusMessage
      v-if="!funcList.length"
      :requestStatus="loadFuncsReqStatus"
      loadingMessage="Loading functions..."
    />
    <ScrollArea v-if="funcList.length">
      <template #top>
        <div
          class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
        >
          <NewFuncDropdown
            label="Function"
            :fnTypes="CREATE_OPTIONS"
            @selected-func-variant="createNewFunc"
          />
        </div>
        <ErrorMessage
          v-if="createFuncReqStatus.isError"
          :requestStatus="createFuncReqStatus"
        />
        <SiSearch
          autoSearch
          placeholder="search functions"
          @search="onSearch"
        />
        <div
          class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-2 border-b dark:border-neutral-600"
        >
          Select a function to view or edit it.
        </div>
      </template>

      <ul class="overflow-y-auto min-h-[200px]">
        <Collapsible
          v-for="(label, variant) in CUSTOMIZABLE_FUNC_TYPES"
          :key="variant"
          as="li"
          class="w-full"
          contentAs="ul"
          defaultOpen
        >
          <template #label>
            <div class="flex items-center gap-2">
              <FuncSkeleton />
              <span> {{ label.pluralLabel }} </span>
            </div>
          </template>
          <template #default>
            <li v-for="func in funcsByVariant[variant] ?? []" :key="func.id">
              <SiFuncListItem
                :func="func"
                color="#921ed6"
                context="workspace-lab-functions"
              />
            </li>
          </template>
        </Collapsible>
      </ul>
    </ScrollArea>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import * as _ from "lodash-es";
import {
  Collapsible,
  ErrorMessage,
  RequestStatusMessage,
  ScrollArea,
} from "@si/vue-lib/design-system";
import SiFuncListItem from "@/components/SiFuncListItem.vue";
import SiSearch from "@/components/SiSearch.vue";
import { CUSTOMIZABLE_FUNC_TYPES, FuncVariant } from "@/api/sdf/dal/func";
import NewFuncDropdown from "@/components/NewFuncDropdown.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import FuncSkeleton from "@/components/FuncSkeleton.vue";

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

const funcsByVariant = computed(() =>
  _.groupBy(filteredList.value, (f) => f.variant),
);

const createFuncReqStatus = funcStore.getRequestStatus("CREATE_FUNC");

async function createNewFunc(variant: FuncVariant) {
  const createReq = await funcStore.CREATE_FUNC({ variant });
  if (createReq.result.success) {
    routeToFunc(createReq.result.data.id);
  }
}
</script>
