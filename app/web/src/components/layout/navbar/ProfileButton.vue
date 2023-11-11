<template>
  <NavbarButton tooltipText="Profile" class="flex-none h-full overflow-hidden">
    <template #default="{ open, hovered }">
      <div class="flex-row flex text-white items-center">
        <img
          v-if="authStore.user?.picture_url"
          class="h-8 w-8 rounded-full border-2 border-shade-100 bg-shade-100 overflow-hidden block"
          :src="authStore.user?.picture_url"
          referrerpolicy="no-referrer"
        />
        <Icon v-else name="user-circle" />
        <SiArrow :nudge="open || hovered" class="ml-1" />
      </div>
    </template>

    <template #dropdownContent>
      <DropdownMenuItem
        linkToNamedRoute="logout"
        icon="logout"
        label="Logout"
      />
      <DropdownMenuItem
        v-if="isDevMode"
        linkToNamedRoute="workspace-dev-dashboard"
        icon="cat"
        label="Dev Dashboard"
      />
    </template>
  </NavbarButton>
</template>

<script lang="ts" setup>
import { DropdownMenuItem, Icon } from "@si/vue-lib/design-system";
import SiArrow from "@/components/SiArrow.vue";
import { usePresenceStore } from "@/store/presence.store";
import { useAuthStore } from "@/store/auth.store";
import NavbarButton from "./NavbarButton.vue";
import UserIcon from "./UserIcon.vue";

const authStore = useAuthStore();

const isDevMode = import.meta.env.DEV;
</script>
