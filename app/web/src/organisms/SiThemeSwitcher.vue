<template>
  <!-- FIXME(nick,theo): dropdown-classes needs to be removed in favor of the dropdown knowing whether or not it is offscreen. -->
  <SiBarButton tooltip-text="Change theme" dropdown-classes="-right-12">
    <Icon :name="lightmode ? 'sun' : 'moon'" />

    <template #dropdownContent>
      <SiDropdownItem
        class="text-sm"
        :checked="theme?.source === 'system'"
        @select="ThemeService.resetToSystems"
      >
        System theme
      </SiDropdownItem>
      <SiDropdownItem
        class="text-sm"
        :checked="theme?.source === 'user' && theme?.value === 'light'"
        @select="ThemeService.setTo('light')"
      >
        Light theme
      </SiDropdownItem>
      <SiDropdownItem
        class="text-sm"
        :checked="theme?.source === 'user' && theme?.value === 'dark'"
        @select="ThemeService.setTo('dark')"
      >
        Dark theme
      </SiDropdownItem>
    </template>
  </SiBarButton>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { refFrom } from "vuse-rx/src";
import { ThemeService } from "@/service/theme";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import { Theme } from "@/observable/theme";
import Icon from "@/ui-lib/Icon.vue";

const theme = refFrom<Theme>(ThemeService.currentTheme());
const lightmode = computed(() => {
  return theme.value?.value === "light";
});
</script>
