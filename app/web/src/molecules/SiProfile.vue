<template>
  <Menu as="div" class="ml-3 relative">
    <div>
      <SiProfileButton />
    </div>

    <transition
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <SiIconDropdown :options="options" />
    </transition>
  </Menu>
</template>

<script setup lang="ts">
import { SessionService } from "@/service/session";
import { useRouter } from "vue-router";
import { Menu } from "@headlessui/vue";
import SiIconDropdown from "@/atoms/SiIconDropdown.vue";
import { SiIconDropdownOption } from "@/atoms/SiIconDropdown/types";
import SiProfileButton from "@/atoms/SiProfileButton.vue";

const router = useRouter();
const options: SiIconDropdownOption[] = [
  {
    text: "Switch to old app",
    action: async () => {
      await router.push({ name: "home" });
    },
  },
  {
    text: "Logout",
    action: async () => {
      await SessionService.logout();
      await router.push({ name: "login" });
    },
  },
];
</script>
