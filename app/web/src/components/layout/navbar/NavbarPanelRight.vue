<template>
  <div class="flex items-center h-full">
    <NavbarButton tooltip-text="Copy link" @click="copyURL">
      <Icon name="link" />
    </NavbarButton>

    <SiThemeSwitcher />

    <NavbarButton tooltip-text="Profile">
      <template #default="{ open, hovered }">
        <div class="flex-row flex text-white items-center">
          <img
            class="h-8 w-8 rounded-full bg-white border-black border-2"
            src=""
          />
          <SiArrow :nudge="open || hovered" class="ml-1" />
        </div>
      </template>

      <template #dropdownContent>
        <DropdownMenuItem
          link-to-named-route="logout"
          icon="logout"
          label="Logout"
        />
        <DropdownMenuItem
          v-if="isDevMode"
          link-to-named-route="workspace-dev-dashboard"
          icon="cat"
          label="Dev Dashboard"
        />
      </template>
    </NavbarButton>
  </div>
</template>

<script lang="ts" setup>
import { Icon, DropdownMenuItem } from "@si/vue-lib/design-system";
import SiArrow from "@/components/SiArrow.vue";
import SiThemeSwitcher from "./NavbarThemeSwitcher.vue";
import NavbarButton from "./NavbarButton.vue";

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

// Cannot use inside the template directly.
const isDevMode = import.meta.env.DEV;
</script>
