<template>
  <!-- FIXME(nick,theo): dropdown-classes needs to be removed in favor of the dropdown knowing whether or not it is offscreen. -->
  <NavbarButton ref="buttonRef" tooltip-text="Change theme">
    <Icon :name="currentTheme === 'light' ? 'sun' : 'moon'" />

    <template #dropdownContent>
      <DropdownMenuItem
        :checked="!userOverrideTheme"
        icon="bolt"
        @select="userOverrideTheme = null"
      >
        System theme
      </DropdownMenuItem>
      <DropdownMenuItem
        :checked="userOverrideTheme === 'light'"
        icon="sun"
        @select="userOverrideTheme = 'light'"
      >
        Light theme
      </DropdownMenuItem>
      <DropdownMenuItem
        :checked="userOverrideTheme === 'dark'"
        icon="moon"
        @select="userOverrideTheme = 'dark'"
      >
        Dark theme
      </DropdownMenuItem>
    </template>
  </NavbarButton>
</template>

<script setup lang="ts">
import Icon from "@/ui-lib/icons/Icon.vue";
import { userOverrideTheme, useTheme } from "@/ui-lib/theme_tools";
import DropdownMenuItem from "@/ui-lib/menus/DropdownMenuItem.vue";
import NavbarButton from "./NavbarButton.vue";

const { theme: currentTheme } = useTheme();
</script>
