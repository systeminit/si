<template>
  <Menu>
    <div class="relative w-fit">
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
        class="z-30 absolute right-0 mt-2 rounded bg-white dark:bg-black shadow-lg border focus:outline-none overflow-hidden"
      >
        <MenuItem
          v-for="(item, key) in menuItems"
          :key="item"
          as="a"
          class="flex flex-row relative items-center whitespace-nowrap py-2 px-4 cursor-pointer gap-2 hover:bg-action-500 hover:text-white"
          @click="emits('selectedFuncKind', key as FuncBackendKind)"
        >
          <FuncSkeleton />
          {{ item }}
        </MenuItem>
      </MenuItems>
    </div>
  </Menu>
</template>

<script setup lang="ts">
import { MenuButton, MenuItem, MenuItems, Menu } from "@headlessui/vue";
import { PropType } from "vue";
import VButton from "@/molecules/VButton.vue";
import FuncSkeleton from "@/atoms/FuncSkeleton.vue";
import { FuncBackendKind } from "@/api/sdf/dal/func";

const props = defineProps({
  label: { type: String, required: true },
  menuItems: {
    type: Object as PropType<{ [key: string]: string }>,
    required: true,
  },
});

const emits = defineEmits<{
  (e: "selectedFuncKind", kind: FuncBackendKind): void;
}>();
</script>
