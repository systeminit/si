<template>
  <MenuItems
    class="origin-top-right absolute right-0 mt-2 w-48 rounded-md shadow-lg py-1 bg-white ring-1 ring-black ring-opacity-5 focus:outline-none"
  >
    <div v-if="props.enableOldAppSwitch">
      <MenuItem v-slot="{ active }">
        <a
          href="#"
          :class="[
            active ? 'bg-gray-100' : '',
            'block px-4 py-2 text-sm text-gray-700',
          ]"
          @click="onOldAppSwitch"
          >Switch to "old" app</a
        >
      </MenuItem>
    </div>

    <MenuItem v-slot="{ active }">
      <a
        :class="[
          active ? 'bg-gray-100' : '',
          'block px-4 py-2 text-sm text-gray-700',
        ]"
        @click="onLogout"
        >Logout</a
      >
    </MenuItem>
  </MenuItems>
</template>

<script setup lang="ts">
import { SessionService } from "@/service/session";
import { MenuItem, MenuItems } from "@headlessui/vue";
import { useRouter } from "vue-router";
import { defineProps } from "vue";

const props = defineProps<{
  enableOldAppSwitch?: boolean;
}>();

const router = useRouter();
const onLogout = async () => {
  await SessionService.logout();
  await router.push({ name: "login" });
};
const onOldAppSwitch = async () => {
  await router.push({ name: "home" });
};
</script>
