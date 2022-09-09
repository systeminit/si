<template>
  <Menu>
    <div class="block w-fit">
      <MenuButton>
        <VButton
          button-rank="primary"
          button-type="success"
          icon="plus"
          icon-right="chevron--down"
          :label="label"
          size="sm"
        />
      </MenuButton>

      <MenuItems
        class="z-30 absolute mt-2 rounded bg-white dark:bg-black shadow-lg border focus:outline-none overflow-hidden"
      >
        <MenuItem
          as="a"
          class="flex flex-row relative items-center whitespace-nowrap py-2 px-4 cursor-pointer gap-2 hover:bg-action-500 hover:text-white"
          @click="emits('selectedFuncKind', FuncBackendKind.JsQualification)"
        >
          <FuncSkeleton />
          Qualification
        </MenuItem>
      </MenuItems>
    </div>
  </Menu>
</template>

<script setup lang="ts">
import { MenuButton, MenuItem, MenuItems, Menu } from "@headlessui/vue";
import VButton from "@/molecules/VButton.vue";
import FuncSkeleton from "@/atoms/FuncSkeleton.vue";
import { FuncBackendKind } from "@/api/sdf/dal/func";

defineProps<{
  label: string;
}>();

const emits = defineEmits<{
  (e: "selectedFuncKind", kind: FuncBackendKind): void;
}>();
</script>
