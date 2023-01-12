<template>
  <SiTabGroup @change="onTabChange">
    <template #tabs>
      <SiTabHeader :key="0">FUNCTIONS</SiTabHeader>
      <SiTabHeader :key="1">PACKAGES</SiTabHeader>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto flex flex-col">
        <RequestStatusMessage
          :request-status="loadFuncsReqStatus"
          loading-message="Loading functions..."
        />
        <template v-if="loadFuncsReqStatus.isSuccess">
          <div
            class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
          >
            <NewFuncDropdown
              label="Function"
              :fn-types="CREATE_OPTIONS"
              @selected-func-variant="createNewFunc"
            />

            <NewFuncDropdown
              v-if="isDevMode"
              label="Builtin"
              :fn-types="BUILTIN_CREATE_OPTIONS"
              @selected-func-variant="openCreateBuiltinModal"
            />
          </div>
          <SiSearch
            auto-search
            placeholder="search functions"
            @search="onSearch"
          />
          <div
            class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-2 border-b dark:border-neutral-600"
          >
            Select a function to view or edit it.
          </div>
          <ul class="overflow-y-auto min-h-[200px]">
            <SiCollapsible
              v-for="(fnTypeInfo, variant) in CUSTOMIZABLE_FUNC_TYPES"
              :key="variant"
              as="li"
              class="w-full"
              content-as="ul"
              default-open
            >
              <template #label>
                <div class="flex items-center gap-2">
                  <FuncSkeleton />
                  <span> {{ fnTypeInfo.pluralLabel }} </span>
                </div>
              </template>
              <template #default>
                <li
                  v-for="func in funcsByVariant[variant] ?? []"
                  :key="func.id"
                >
                  <SiFuncListItem
                    :func="func"
                    color="#921ed6"
                    @click="routeToFunc(func.id)"
                  />
                </li>
              </template>
            </SiCollapsible>
          </ul>
        </template>
      </TabPanel>
      <TabPanel />
    </template>
    <Modal
      ref="createBuiltinModalRef"
      size="sm"
      title="Create Builtin Function"
    >
      <Stack>
        <VormInput
          v-model="newBuiltinFuncName"
          label="New function name"
          placeholder="ex: myCoolFunc"
          :regex="VALID_FUNC_NAME_REGEX"
          regex-message="Letters and numbers only"
          required
        />
        <VButton2
          :disabled="validationState.isError"
          :request-status="createBuiltinFuncReqStatus"
          icon="plus-circle"
          label="Create"
          tone="success"
          @click="tryCreateBuiltinFunc"
        />
      </Stack>
    </Modal>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { computed, reactive, ref, Ref } from "vue";
import { TabPanel } from "@headlessui/vue";
import { storeToRefs } from "pinia";
import _ from "lodash";
import { useRouter } from "vue-router";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiFuncListItem from "@/molecules/SiFuncListItem.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import FuncSkeleton from "@/atoms/FuncSkeleton.vue";
import { CUSTOMIZABLE_FUNC_TYPES, FuncVariant } from "@/api/sdf/dal/func";
import NewFuncDropdown from "@/organisms/NewFuncDropdown.vue";
import Modal from "@/ui-lib/modals/Modal.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import { useValidatedInputGroup } from "@/ui-lib/forms/helpers/form-validation";

const router = useRouter();
const routeToFunc = useRouteToFunc();
const funcStore = useFuncStore();
const loadFuncsReqStatus = funcStore.getRequestStatus("FETCH_FUNC_LIST");
const { funcList } = storeToRefs(funcStore);

const isDevMode = import.meta.env.DEV;

const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const CREATE_OPTIONS = _.mapValues(CUSTOMIZABLE_FUNC_TYPES, "singularLabel");
const BUILTIN_CREATE_OPTIONS = _.mapValues(
  _.pickBy(CUSTOMIZABLE_FUNC_TYPES, { enableBuiltIn: true }),
  "singularLabel",
);

const filteredList = computed(() => {
  if (!searchString.value) return funcList.value;
  return _.filter(funcList.value, (f) =>
    f.name.toLocaleLowerCase().includes(searchString.value),
  );
});

const funcsByVariant = computed(() =>
  _.groupBy(filteredList.value, (f) => f.variant),
);

// creating new regular function ////////////
// TODO: show spinner and error message!
// const createFuncReqStatus = funcStore.getRequestStatus("CREATE_FUNC");
async function createNewFunc(variant: FuncVariant) {
  const createReq = await funcStore.CREATE_FUNC({ variant });
  if (createReq.result.success) {
    routeToFunc(createReq.result.data.id);
  }
}

// creating new builtin function ////////

const createBuiltinModalRef = ref<InstanceType<typeof Modal>>();

const newBuiltinFuncName = ref("");
const newBuiltinFuncVariant = ref<FuncVariant>();

const openCreateBuiltinModal = (variant: FuncVariant) => {
  newBuiltinFuncName.value = "";
  newBuiltinFuncVariant.value = variant;
  createBuiltinModalRef.value?.open();
};

const VALID_FUNC_NAME_REGEX = /^[a-z0-9]+$/i;

const createBuiltinFuncReqStatus = funcStore.getRequestStatus(
  "CREATE_BUILTIN_FUNC",
);

const { validationState, validationMethods } = useValidatedInputGroup();
async function tryCreateBuiltinFunc() {
  if (import.meta.env.DEV) {
    if (validationMethods.hasError()) return;
    const funcReq = await funcStore.CREATE_BUILTIN_FUNC({
      name: `si:${newBuiltinFuncName.value}`,
      variant: FuncVariant.Attribute,
    });
    if (funcReq.result.success) {
      createBuiltinModalRef.value?.close();
      routeToFunc(funcReq.result.data.id);
    }
  } else {
    throw new Error("Cannot create builtin funcs outside of dev mode");
  }
}

const onTabChange = () => {
  router.push({ name: "workspace-lab-packages" });
};
</script>
