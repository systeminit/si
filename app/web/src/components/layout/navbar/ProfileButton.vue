<template>
  <NavbarButton tooltipText="Profile" class="flex-none py-xs">
    <template #default="{ open, hovered }">
      <div class="flex-row flex text-white items-center">
        <UserIcon
          :user="{
            name: authStore.user?.name ?? '',
            pictureUrl: authStore.user?.picture_url ?? null,
          }"
        />
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
import { DropdownMenuItem } from "@si/vue-lib/design-system";
import SiArrow from "@/components/SiArrow.vue";
import { useAuthStore } from "@/store/auth.store";
import { isDevMode } from "@/utils/debug";
import NavbarButton from "./NavbarButton.vue";
import UserIcon from "./UserIcon.vue";

const authStore = useAuthStore();
</script>
