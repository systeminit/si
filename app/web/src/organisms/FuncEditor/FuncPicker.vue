<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">FUNCTIONS</SiTabHeader>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto flex flex-col">
        <div
          class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
        >
          <NewFuncDropdown
            label="Function"
            :fn-types="CREATE_OPTIONS"
            @selected-func-kind="createFunc"
          />

          <NewFuncDropdown
            v-if="isDevMode"
            label="Builtin"
            :fn-types="BUILTIN_CREATE_OPTIONS"
            @selected-func-kind="openFuncNameModal"
          />
        </div>
        <SiSearch
          auto-search
          placeholder="search functions"
          @search="onSearch"
        />
        <div
          class="w-full text-neutral-400 dark:text-neutral-300 text-sm p-2 border-b dark:border-neutral-600"
        >
          Select a function from the lists below to view or edit it.
        </div>
        <ul class="overflow-y-auto">
          <SiCollapsible
            v-for="(fnTypeInfo, kind) in CUSTOMIZABLE_FUNC_TYPES"
            :key="kind"
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
              <li v-for="func in funcsByKind[kind] ?? []" :key="func.id">
                <SiFuncSprite
                  :class="
                    selectedFuncId === func.id
                      ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
                      : ''
                  "
                  :is-builtin="func.isBuiltin"
                  :name="func.name"
                  class="border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
                  color="#921ed6"
                  @click="routeToFunc(func.id)"
                />
              </li>
            </template>
          </SiCollapsible>
        </ul>
      </TabPanel>
    </template>
  </SiTabGroup>

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
</template>

<script lang="ts" setup>
import { computed, ref, Ref } from "vue";
import { TabPanel } from "@headlessui/vue";
import validator from "validator";
import { storeToRefs } from "pinia";
import _ from "lodash";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiFuncSprite from "@/molecules/SiFuncSprite.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import FuncSkeleton from "@/atoms/FuncSkeleton.vue";
import { CUSTOMIZABLE_FUNC_TYPES, FuncBackendKind } from "@/api/sdf/dal/func";
import NewFuncDropdown from "@/organisms/NewFuncDropdown.vue";
import Modal from "@/ui-lib/Modal.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";

import { useFuncStore } from "@/store/func/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";

const routeToFunc = useRouteToFunc();
const funcStore = useFuncStore();
const { funcList, selectedFuncId } = storeToRefs(funcStore);

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

const funcsByKind = computed(() =>
  filteredList.value.reduce(
    (funcMap, func) =>
      typeof funcMap[func.kind] === "undefined"
        ? { ...funcMap, [func.kind]: [func] }
        : { ...funcMap, [func.kind]: [...funcMap[func.kind], func] },
    {} as { [key in FuncBackendKind]-?: typeof filteredList.value },
  ),
);

const emits = defineEmits<{
  (
    e: "createFunc",
    v: { kind: FuncBackendKind; isBuiltin: boolean; name?: string },
  ): void;
}>();

const createFunc = (kind: FuncBackendKind) => {
  console.log(kind);
  emits("createFunc", { kind, isBuiltin: false });
};

const funcNameModalOpen = ref(false);
const newBuiltinFuncName = ref("");
const newBuiltinFuncKind = ref<FuncBackendKind>();

const openFuncNameModal = (kind: FuncBackendKind) => {
  newBuiltinFuncName.value = "";
  funcNameModalOpen.value = true;
  newBuiltinFuncKind.value = kind;
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
    kind: FuncBackendKind.JsQualification,
    isBuiltin: true,
    name: `si:${newBuiltinFuncName.value}`,
  });
  closeFuncNameModal();
};
</script>
