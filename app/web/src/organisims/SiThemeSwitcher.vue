<template>
  <SiNavbarButton
    tooltip-text="Change theme"
    :options="themeOptions"
    dropdown-classes="right-0 text-center"
  >
    <div v-if="lightmode">
      <MoonIcon class="w-6" />
    </div>
    <div v-else><SunIcon class="w-6" /></div>
  </SiNavbarButton>
</template>

<script setup lang="ts">
import { ThemeService } from "@/service/theme";
import { SiSelectOption } from "@/atoms/SiSelect2/types";
import MoonIcon from "@/atoms/CustomIcons/MoonIcon.vue";
import SunIcon from "@/atoms/CustomIcons/SunIcon.vue";
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";
import { Theme } from "@/observable/theme";
import { computed } from "vue";
import { refFrom } from "vuse-rx/src";

const theme = refFrom<Theme>(ThemeService.currentTheme());
const lightmode = computed(() => {
  console.log("light mode", { theme: theme.value });
  if (theme.value) {
    if (theme.value.value == "light") {
      return true;
    }
    return false;
  }
  return true;
});

const themeOptions: SiSelectOption[] = [
  {
    text: "System theme",
    action: ThemeService.resetToSystems,
  },
  {
    text: "Light theme",
    action: () => ThemeService.setTo("light"),
  },
  {
    text: "Dark theme",
    action: () => ThemeService.setTo("dark"),
  },
];
</script>
