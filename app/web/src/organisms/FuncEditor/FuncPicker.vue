<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">FUNCTIONS</SiTabHeader>
    </template>
    <template #dropdownitems>
      <SiDropdownItem>FUNCTIONS</SiDropdownItem>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto flex flex-col">
        <div
          class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
        >
          <Menu>
            <div class="block w-fit">
              <MenuButton>
                <VButton
                  button-rank="primary"
                  button-type="success"
                  icon="plus"
                  icon-right="chevron--down"
                  label="Function"
                  size="sm"
                />
              </MenuButton>

              <MenuItems
                class="z-30 absolute mt-2 rounded bg-white dark:bg-black shadow-lg border focus:outline-none overflow-hidden"
              >
                <MenuItem
                  v-for="(kindName, kind) in funcCreateTypes"
                  :key="kind"
                  as="a"
                  class="flex flex-row relative items-center whitespace-nowrap py-2 px-4 cursor-pointer gap-2 hover:bg-action-500 hover:text-white"
                  @click="createFunc(kind)"
                >
                  <FuncSkeleton />

                  {{ kindName }}
                </MenuItem>
              </MenuItems>
            </div>
          </Menu>

          <NewFuncDropdown
            v-if="isDevMode"
            label="Builtin"
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
            v-for="(kindName, kind) in funcTypes"
            :key="kind"
            as="li"
            class="w-full"
            content-as="ul"
            default-open
          >
            <template #label>
              <div class="flex items-center gap-2">
                <FuncSkeleton />
                <span> {{ kindName }} </span>
              </div>
            </template>
            <template #default>
              <li
                v-for="func in filteredList.filter((f) => f && f.kind === kind)"
                :key="func.id"
              >
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
import {
  Menu,
  MenuButton,
  MenuItem,
  MenuItems,
  TabPanel,
} from "@headlessui/vue";
import validator from "validator";
import { storeToRefs } from "pinia";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiFuncSprite from "@/molecules/SiFuncSprite.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import VButton from "@/molecules/VButton.vue";
import FuncSkeleton from "@/atoms/FuncSkeleton.vue";
import { FuncBackendKind } from "@/api/sdf/dal/func";
import NewFuncDropdown from "@/organisms/NewFuncDropdown.vue";
import Modal from "@/ui-lib/Modal.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";

import { useFuncStore } from "@/store/funcs.store";
import { useRouteToFunc } from "@/utils/useRouteToFunc";

const routeToFunc = useRouteToFunc();
const funcStore = useFuncStore();
const { funcList, selectedFuncId } = storeToRefs(funcStore);

const isDevMode = import.meta.env.DEV;

const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const funcTypes = {
  [FuncBackendKind.JsQualification]: "Qualifications",
  [FuncBackendKind.JsAttribute]: "Attributes",
  [FuncBackendKind.JsCodeGeneration]: "Code Generation",
  [FuncBackendKind.JsConfirmation]: "Confirmation",
};

const funcCreateTypes = {
  [FuncBackendKind.JsQualification]: "Qualification",
  [FuncBackendKind.JsAttribute]: "Attribute",
  [FuncBackendKind.JsCodeGeneration]: "Code Generation",
  [FuncBackendKind.JsConfirmation]: "Confirmation",
};

const filteredList = computed(() =>
  searchString.value.length > 0
    ? funcList.value.filter((f) =>
        f.name.toLocaleLowerCase().includes(searchString.value),
      )
    : funcList.value,
);

const emits = defineEmits<{
  (
    e: "createFunc",
    v: { kind: FuncBackendKind; isBuiltin: boolean; name?: string },
  ): void;
}>();

const createFunc = (kind: FuncBackendKind) => {
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
