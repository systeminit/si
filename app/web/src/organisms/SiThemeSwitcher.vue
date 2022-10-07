<template>
  <!-- FIXME(nick,theo): dropdown-classes needs to be removed in favor of the dropdown knowing whether or not it is offscreen. -->
  <SiBarButton tooltip-text="Change theme" dropdown-classes="-right-12">
    <Icon :name="currentTheme === 'light' ? 'sun' : 'moon'" />

    <template #dropdownContent>
      <SiDropdownItem
        class="text-sm"
        :checked="!userOverrideTheme"
        @select="userOverrideTheme = null"
      >
        System theme
      </SiDropdownItem>
      <SiDropdownItem
        class="text-sm"
        :checked="userOverrideTheme === 'light'"
        @select="userOverrideTheme = 'light'"
      >
        Light theme
      </SiDropdownItem>
      <SiDropdownItem
        class="text-sm"
        :checked="userOverrideTheme === 'dark'"
        @select="userOverrideTheme = 'dark'"
      >
        Dark theme
      </SiDropdownItem>
    </template>
  </SiBarButton>
</template>

<script setup lang="ts">
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import Icon from "@/ui-lib/Icon.vue";
import { userOverrideTheme, useTheme } from "@/ui-lib/theme_tools";

const { theme: currentTheme } = useTheme();
</script>
