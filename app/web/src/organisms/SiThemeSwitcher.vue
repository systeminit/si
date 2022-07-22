<template>
  <!-- FIXME(nick,theo): dropdown-classes needs to be removed in favor of the dropdown knowing whether or not it is offscreen. -->
  <SiNavbarButton tooltip-text="Change theme" dropdown-classes="-right-12">
    <div v-if="lightmode">
      <MoonIcon class="w-6" />
    </div>
    <div v-else>
      <SunIcon class="w-6" />
    </div>

    <template #dropdownContent>
      <SiDropdownItem
        :checked="theme?.source === 'system'"
        @select="ThemeService.resetToSystems"
      >
        System theme
      </SiDropdownItem>
      <SiDropdownItem
        :checked="theme?.source === 'user' && theme?.value === 'light'"
        @select="ThemeService.setTo('light')"
      >
        Light theme
      </SiDropdownItem>
      <SiDropdownItem
        :checked="theme?.source === 'user' && theme?.value === 'dark'"
        @select="ThemeService.setTo('dark')"
      >
        Dark theme
      </SiDropdownItem>
    </template>
  </SiNavbarButton>
</template>

<script setup lang="ts">
import { ThemeService } from "@/service/theme";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import MoonIcon from "@/atoms/CustomIcons/MoonIcon.vue";
import SunIcon from "@/atoms/CustomIcons/SunIcon.vue";
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";
import { Theme } from "@/observable/theme";
import { computed } from "vue";
import { refFrom } from "vuse-rx/src";

const theme = refFrom<Theme>(ThemeService.currentTheme());
const lightmode = computed(() => {
  return theme.value?.value == "light";
});
</script>
