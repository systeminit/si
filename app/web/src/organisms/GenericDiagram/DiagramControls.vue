<template>
  <div :class="clsx('absolute flex left-4 bottom-4 z-20 h-8')">
    <div
      :class="getButtonClasses(zoomLevel <= MIN_ZOOM)"
      @click="adjustZoom('down')"
    >
      <Icon name="minus" size="full" />
    </div>

    <Menu>
      <MenuButton
        as="div"
        :class="
          clsx(
            'bg-white border-neutral-300 border text-black text-center w-20 cursor-pointer mx-2 flex flex-col justify-center',
            'dark:bg-black dark:text-white dark:border-black',
          )
        "
        title="set zoom level"
      >
        {{ roundedZoomPercent }}%
      </MenuButton>
      <MenuItems as="div" class="absolute mt-[-100%] -top-10 left-6 bg-red">
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
      :class="getButtonClasses(zoomLevel >= MAX_ZOOM)"
      @click="adjustZoom('up')"
    >
      <Icon name="plus" size="full" />
    </div>

    <div
      class="ml-4"
      :class="getButtonClasses(false)"
      @click="emit('open:help')"
    >
      <Icon name="help-circle" size="full" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import _ from "lodash";
import { Menu, MenuButton, MenuItems } from "@headlessui/vue";
import clsx from "clsx";
import SiDropdown from "@/molecules/SiDropdown.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import { tw } from "@/utils/style_helpers";
import { MAX_ZOOM, MIN_ZOOM } from "./diagram_constants";

const ZOOM_LEVEL_OPTIONS = [25, 50, 100, 150, 200];

const props = defineProps({
  zoomLevel: { type: Number, required: true },
});

const emit = defineEmits<{
  (e: "update:zoom", newZoom: number): void;
  (e: "open:help"): void;
}>();

function adjustZoom(direction: "up" | "down") {
  const mult = direction === "down" ? -1 : 1;
  emit("update:zoom", props.zoomLevel + (10 / 100) * mult);
}

const roundedZoomPercent = computed(() => Math.round(props.zoomLevel * 100));

function getButtonClasses(isDisabled: boolean) {
  return clsx(
    tw`rounded-full p-1 bg-neutral-600 text-white`,
    tw`dark:bg-gray-200 dark:text-black`,
    isDisabled
      ? tw`cursor-not-allowed opacity-50`
      : tw`cursor-pointer hover:scale-110`,
  );
}
</script>
