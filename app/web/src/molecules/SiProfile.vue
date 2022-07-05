<template>
  <Menu as="div" class="ml-3 relative">
    <div>
      <SiProfileIcon />
    </div>

    <transition
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <SiProfileDropdown :enable-old-app-switch="props.enableOldAppSwitch" />
    </transition>
  </Menu>
</template>

<script setup lang="ts">
import { Menu } from "@headlessui/vue";
import SiProfileIcon from "@/atoms/SiProfileIcon.vue";
import SiProfileDropdown from "@/atoms/SiProfileDropdown.vue";

const props = defineProps<{
  enableOldAppSwitch?: boolean;
}>();
</script>
