<template>
  <div class="absolute flex left-4 bottom-4 z-20 h-8">
    <div
      class="rounded-full w-8 h-8 bg-neutral-600 text-white dark:bg-gray-200 dark:text-black p-2 cursor-pointer"
      @click="adjustZoom('down')"
    >
      <Icon icon="minus" />
    </div>

    <Menu>
      <MenuButton
        as="div"
        class="bg-white border-neutral-300 border text-black dark:bg-black dark:text-white dark:border-black text-center w-20 cursor-pointer mx-2 h-8 flex flex-col justify-center"
        title="set zoom level"
      >
        {{ roundedZoomPercent }}%
      </MenuButton>
      <MenuItems as="div" class="absolute mt-[-100%] -top-10 left-6">
        <SiDropdown>
          <SiDropdownItem
            v-for="zoomOptionAmount in ZOOM_LEVEL_OPTIONS"
            :key="zoomOptionAmount"
            class="text-sm text-white"
            @select="emit('update:zoom', zoomOptionAmount / 100)"
          >
            {{ zoomOptionAmount }}%
          </SiDropdownItem>
        </SiDropdown>
      </MenuItems>
    </Menu>

    <div
      class="rounded-full w-8 h-8 bg-neutral-600 text-white dark:bg-gray-200 dark:text-black p-2 cursor-pointer"
      @click="adjustZoom('up')"
    >
      <Icon icon="plus" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import _ from "lodash";
import { Menu, MenuButton, MenuItems } from "@headlessui/vue";
import Icon from "@/molecules/VButton/Icon.vue";
import SiDropdown from "@/molecules/SiDropdown.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";

const ZOOM_LEVEL_OPTIONS = [25, 50, 100, 150, 200];

const props = defineProps({
  zoomLevel: { type: Number, required: true },
});

const emit = defineEmits<{
  (e: "update:zoom", newZoom: number): void;
}>();

function adjustZoom(direction: "up" | "down") {
  const mult = direction === "down" ? -1 : 1;
  emit("update:zoom", props.zoomLevel + (10 / 100) * mult);
}

const roundedZoomPercent = computed(() => Math.round(props.zoomLevel * 100));
</script>
