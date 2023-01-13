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
              @selected-func-variant="createFunc"
            />

            <NewFuncDropdown
              v-if="isDevMode"
              label="Builtin"
              :fn-types="BUILTIN_CREATE_OPTIONS"
              @selected-func-variant="openFuncNameModal"
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
      size="sm"
      :open="funcNameModalOpen"
      type="save"
      @close="closeFuncNameModal"
      @save="createBuiltinFunc"
    >
      <template #title>Name your Builtin Function</template>
      <template #content>
        <SiTextBox
          id="name"
          v-model="newBuiltinFuncName"
          class="pb-2"
          autofocus
          title=""
          required
          placeholder="Type the name of this function here..."
          :validations="[
            {
              id: 'name',
              message: 'Alphanumeric characters only.',
              check: validator.isAlphanumeric,
            },
          ]"
          @keyup.enter="createBuiltinFunc"
          @error="updateFuncNameError"
        />
      </template>
    </Modal>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { computed, ref, Ref } from "vue";
import { TabPanel } from "@headlessui/vue";
import validator from "validator";
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
import Modal from "@/ui-lib/Modal.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import NewFuncDropdown from "../NewFuncDropdown.vue";

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

const filteredList = computed(() =>
  searchString.value.length > 0
    ? funcList.value.filter((f) =>
        f.name.toLocaleLowerCase().includes(searchString.value),
      )
    : funcList.value,
);

const funcsByVariant = computed(() =>
  filteredList.value.reduce(
    (funcMap, func) =>
      typeof funcMap[func.variant] === "undefined"
        ? { ...funcMap, [func.variant]: [func] }
        : { ...funcMap, [func.variant]: [...funcMap[func.variant], func] },
    {} as { [key in FuncVariant]-?: typeof filteredList.value },
  ),
);

const emits = defineEmits<{
  (
    e: "createFunc",
    v: { variant: FuncVariant; isBuiltin: boolean; name?: string },
  ): void;
}>();

const createFunc = (variant: FuncVariant) => {
  emits("createFunc", { variant, isBuiltin: false });
};

const funcNameModalOpen = ref(false);
const newBuiltinFuncName = ref("");
const newBuiltinFuncVariant = ref<FuncVariant>();

const openFuncNameModal = (variant: FuncVariant) => {
  newBuiltinFuncName.value = "";
  funcNameModalOpen.value = true;
  newBuiltinFuncVariant.value = variant;
};

const closeFuncNameModal = () => {
  funcNameModalOpen.value = false;
};

const funcNameHasError = ref(false);
const updateFuncNameError = ({ value }: Ref<boolean>) => {
  funcNameHasError.value = value;
};

const createBuiltinFunc = () => {
  if (newBuiltinFuncName.value === "") return;
  if (funcNameHasError.value) return;

  emits("createFunc", {
    variant: FuncVariant.Attribute,
    isBuiltin: true,
    name: `si:${newBuiltinFuncName.value}`,
  });
  closeFuncNameModal();
};

const onTabChange = () => {
  router.push({ name: "workspace-lab-packages" });
};
</script>
